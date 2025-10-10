use super::*;
use git2::Repository;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Manages Git repository operations for remote imports
pub struct GitRepositoryManager {
  cache_dir: PathBuf,
}

impl GitRepositoryManager {
  pub fn new() -> RunResult<'static, Self> {
    let cache_dir = Self::get_cache_dir()?;
    
    // Ensure cache directory exists
    std::fs::create_dir_all(&cache_dir).map_err(|io_error| Error::TempdirIo {
      recipe: "git_cache_setup",
      io_error,
    })?;

    Ok(Self { cache_dir })
  }

  /// Get the cache directory for Git repositories
  fn get_cache_dir() -> RunResult<'static, PathBuf> {
    // Try to use system cache directory, fall back to current directory
    // on Linux this will default to ~/.cache/just/git-repos
    // on Mac this will default to ~/Library/Caches/just/git-repos
    // on Windows this will default to %USERPROFILE%\AppData\Local\just\git-repos
    let cache_dir = if let Some(cache_dir) = dirs::cache_dir() {
      cache_dir.join("just").join("git-repos")
    } else {
      std::env::current_dir()
        .map_err(|io_error| Error::TempdirIo {
          recipe: "cache_dir_fallback",
          io_error,
        })?
        .join(".just-cache")
        .join("git-repos")
    };

    Ok(cache_dir)
  }

  /// Creates a hash-based directory name for the given URL and SHA
  fn get_repo_cache_key(git_url: &str, sha: Option<&str>) -> String {
    let mut hasher = DefaultHasher::new();
    git_url.hash(&mut hasher);
    if let Some(sha) = sha {
      sha.hash(&mut hasher);
    }
    format!("{:x}", hasher.finish())
  }

  /// Determines if a path looks like a Git repository URL
  pub fn is_git_url(path: &str) -> bool {
    // Check for common Git URL patterns
    path.starts_with("git://")
      || path.starts_with("git@")
      || path.starts_with("https://")
      || path.starts_with("http://")
      || path.starts_with("ssh://")
      || (path.contains("github.com") && (path.contains(".git") || !path.contains('.')))
      || (path.contains("gitlab.com") && (path.contains(".git") || !path.contains('.')))
      || (path.contains("bitbucket.org") && (path.contains(".git") || !path.contains('.')))
      || path.ends_with(".git")
  }

  /// Clones a Git repository to a cache directory and returns the path to the specified file
  pub fn clone_and_get_path<'src>(
    &self,
    git_url: &str,
    sha: Option<&str>,
    file_path: Option<&str>,
    force_reclone: bool,
  ) -> RunResult<'src, PathBuf> {
    let cache_key = Self::get_repo_cache_key(git_url, sha);
    let repo_cache_dir = self.cache_dir.join(&cache_key);

    // Check if repository already exists and we're not forcing a re-clone
    if !force_reclone && repo_cache_dir.exists() && self.is_valid_repo(&repo_cache_dir) {
      return self.resolve_file_path(&repo_cache_dir, file_path);
    }

    // Remove existing repository if it exists (for force re-clone or invalid repo)
    if repo_cache_dir.exists() {
      std::fs::remove_dir_all(&repo_cache_dir).map_err(|io_error| Error::TempdirIo {
        recipe: "git_cache_cleanup",
        io_error,
      })?;
    }

    // Clone the repository
    let repo = self.clone_repository(git_url, &repo_cache_dir)?;

    // If a SHA is specified, checkout that specific commit
    if let Some(sha) = sha {
      self.checkout_commit(&repo, sha)?;
    }

    self.resolve_file_path(&repo_cache_dir, file_path)
  }

  /// Check if a directory contains a valid Git repository
  fn is_valid_repo(&self, path: &Path) -> bool {
    Repository::open(path).is_ok()
  }

  /// Resolves the file path within the repository directory
  fn resolve_file_path<'src>(
    &self,
    repo_path: &Path,
    file_path: Option<&str>,
  ) -> RunResult<'src, PathBuf> {
    let target_path = if let Some(file_path) = file_path {
      repo_path.join(file_path)
    } else {
      // Default to looking for justfile or .justfile
      let justfile_path = repo_path.join("justfile");
      let dot_justfile_path = repo_path.join(".justfile");
      
      if justfile_path.exists() {
        justfile_path
      } else if dot_justfile_path.exists() {
        dot_justfile_path
      } else {
        return Err(Error::MissingImportFile {
          path: Token {
            src: "",
            offset: 0,
            length: 0,
            line: 0,
            column: 0,
            path: Path::new(""),
            kind: TokenKind::StringToken,
          },
        });
      }
    };

    if target_path.exists() {
      Ok(target_path)
    } else {
      Err(Error::MissingImportFile {
        path: Token {
          src: "",
          offset: 0,
          length: 0,
          line: 0,
          column: 0,
          path: Path::new(""),
          kind: TokenKind::StringToken,
        },
      })
    }
  }

  /// Checkout a specific commit in the repository
  fn checkout_commit<'src>(&self, repo: &Repository, sha: &str) -> RunResult<'src, ()> {
    let oid = git2::Oid::from_str(sha).map_err(|git_error| {
      self.categorize_git_error("", git_error)
    })?;

    let commit = repo.find_commit(oid).map_err(|git_error| {
      self.categorize_git_error("", git_error)
    })?;

    // Checkout the specific commit
    repo
      .checkout_tree(commit.as_object(), None)
      .map_err(|git_error| self.categorize_git_error("", git_error))?;

    // Update HEAD to point to the commit
    repo
      .set_head_detached(oid)
      .map_err(|git_error| self.categorize_git_error("", git_error))?;

    Ok(())
  }

  /// Clones a repository to the specified target path
  fn clone_repository<'src>(&self, url: &str, target_path: &Path) -> RunResult<'src, Repository> {
    let mut builder = git2::build::RepoBuilder::new();

    // Set up credential callback for authentication
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| {
      git2::Cred::ssh_key_from_agent(_username_from_url.unwrap_or("git"))
    });

    // Accept all certificates for now (you might want to be more strict in production)
    callbacks.certificate_check(|_, _| Ok(git2::CertificateCheckStatus::CertificateOk));

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    builder.fetch_options(fetch_options);

    builder
      .clone(url, target_path)
      .map_err(|git_error| self.categorize_git_error(url, git_error))
  }

  /// Maps git2 errors to our custom error types
  fn categorize_git_error<'src>(&self, url: &str, git_error: git2::Error) -> Error<'src> {
    match git_error.class() {
      git2::ErrorClass::Net => Error::GitNetwork {
        url: url.to_string(),
        message: git_error.message().to_string(),
      },
      git2::ErrorClass::Ssh => Error::GitAuth {
        url: url.to_string(),
        message: git_error.message().to_string(),
      },
      git2::ErrorClass::Http => {
        if git_error.code() == git2::ErrorCode::Auth {
          Error::GitAuth {
            url: url.to_string(),
            message: git_error.message().to_string(),
          }
        } else {
          Error::GitNetwork {
            url: url.to_string(),
            message: git_error.message().to_string(),
          }
        }
      }
      _ => Error::GitRepository {
        url: url.to_string(),
        message: git_error.message().to_string(),
      },
    }
  }

  /// Parses a Git URL into its components: base URL, optional SHA, and optional file path
  /// Format: git_url[#sha][@file_path] or git_url[@file_path]
  /// Examples:
  /// - https://github.com/user/repo.git
  /// - https://github.com/user/repo.git#abc123
  /// - https://github.com/user/repo.git@path/to/file.just
  /// - https://github.com/user/repo.git#abc123@path/to/file.just
  pub fn parse_git_url(url: &str) -> (String, Option<String>, Option<String>) {
    // First, check for SHA (after #)
    let (base_url, sha) = if let Some(hash_pos) = url.find('#') {
      let base = &url[..hash_pos];
      let remaining = &url[hash_pos + 1..];

      // Check if there's a file path after the SHA (using @)
      if let Some(at_pos) = remaining.find('@') {
        let sha = &remaining[..at_pos];
        let file_path = &remaining[at_pos + 1..];
        return (base.to_string(), Some(sha.to_string()), Some(file_path.to_string()));
      } else {
        (base, Some(remaining.to_string()))
      }
    } else {
      (url, None)
    };

    // Then check for file path (after @) if no SHA was found
    if sha.is_none() {
      if let Some(at_pos) = base_url.rfind('@') {
        let before_at = &base_url[..at_pos];
        let after_at = &base_url[at_pos + 1..];

        // If what comes after @ looks like a path (contains / or .), treat it as a file path
        if after_at.contains('/') || after_at.contains('.') {
          // But make sure this isn't a user@host pattern (like git@github.com:user/repo.git)
          // Check if this is an SSH URL pattern by looking for the absence of :// protocol
          if before_at.contains("://") {
            return (before_at.to_string(), None, Some(after_at.to_string()));
          }
        }
      }
    }

    (base_url.to_string(), sha, None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_git_url() {
    assert!(GitRepositoryManager::is_git_url("https://github.com/user/repo.git"));
    assert!(GitRepositoryManager::is_git_url("git@github.com:user/repo.git"));
    assert!(GitRepositoryManager::is_git_url("git://example.com/repo.git"));
    assert!(!GitRepositoryManager::is_git_url("./local/path"));
    assert!(!GitRepositoryManager::is_git_url("/absolute/path"));
  }

  #[test]
  fn test_parse_git_url() {
    // Test basic URL
    let (url, sha, path) = GitRepositoryManager::parse_git_url("https://github.com/user/repo.git");
    assert_eq!(url, "https://github.com/user/repo.git");
    assert_eq!(sha, None);
    assert_eq!(path, None);

    // Test URL with file path
    let (url, sha, path) = GitRepositoryManager::parse_git_url("https://github.com/user/repo.git@path/to/file.just");
    assert_eq!(url, "https://github.com/user/repo.git");
    assert_eq!(sha, None);
    assert_eq!(path, Some("path/to/file.just".to_string()));

    // Test URL with SHA
    let (url, sha, path) = GitRepositoryManager::parse_git_url("https://github.com/user/repo.git#abc123");
    assert_eq!(url, "https://github.com/user/repo.git");
    assert_eq!(sha, Some("abc123".to_string()));
    assert_eq!(path, None);

    // Test URL with SHA and file path
    let (url, sha, path) = GitRepositoryManager::parse_git_url("https://github.com/user/repo.git#abc123@path/to/file.just");
    assert_eq!(url, "https://github.com/user/repo.git");
    assert_eq!(sha, Some("abc123".to_string()));
    assert_eq!(path, Some("path/to/file.just".to_string()));
  }
} 