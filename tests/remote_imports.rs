use super::*;

#[test]
fn detect_git_urls() {
  // Test Git URL detection
  assert!(just::git_repository::GitRepositoryManager::is_git_url(
    "https://github.com/user/repo.git"
  ));
  assert!(just::git_repository::GitRepositoryManager::is_git_url(
    "git@github.com:user/repo.git"
  ));
  assert!(just::git_repository::GitRepositoryManager::is_git_url(
    "https://github.com/user/repo"
  ));

  // Test non-Git paths
  assert!(!just::git_repository::GitRepositoryManager::is_git_url(
    "../local/file.just"
  ));
  assert!(!just::git_repository::GitRepositoryManager::is_git_url(
    "./relative/file.just"
  ));
  assert!(!just::git_repository::GitRepositoryManager::is_git_url(
    "/absolute/path/file.just"
  ));
}

#[test]
fn parse_git_url_with_path() {
  // Test parsing URLs with embedded file paths
  let (url, sha, path) = just::git_repository::GitRepositoryManager::parse_git_url(
    "https://github.com/user/repo@path/to/file.just",
  );
  assert_eq!(url, "https://github.com/user/repo");
  assert_eq!(sha, None);
  assert_eq!(path, Some("path/to/file.just".to_string()));

  // Test plain Git URLs
  let (url, sha, path) = just::git_repository::GitRepositoryManager::parse_git_url(
    "https://github.com/user/repo.git",
  );
  assert_eq!(url, "https://github.com/user/repo.git");
  assert_eq!(sha, None);
  assert_eq!(path, None);
}

#[test]
fn parse_git_url_with_sha() {
  // Test parsing URLs with SHA only
  let (url, sha, path) = just::git_repository::GitRepositoryManager::parse_git_url(
    "https://github.com/user/repo.git#abc123def456",
  );
  assert_eq!(url, "https://github.com/user/repo.git");
  assert_eq!(sha, Some("abc123def456".to_string()));
  assert_eq!(path, None);

  // Test parsing URLs with both SHA and file path
  let (url, sha, path) = just::git_repository::GitRepositoryManager::parse_git_url(
    "https://github.com/user/repo.git#abc123def456@path/to/file.just",
  );
  assert_eq!(url, "https://github.com/user/repo.git");
  assert_eq!(sha, Some("abc123def456".to_string()));
  assert_eq!(path, Some("path/to/file.just".to_string()));

  // Test parsing URLs with short SHA
  let (url, sha, path) = just::git_repository::GitRepositoryManager::parse_git_url(
    "https://github.com/user/repo.git#abc123",
  );
  assert_eq!(url, "https://github.com/user/repo.git");
  assert_eq!(sha, Some("abc123".to_string()));
  assert_eq!(path, None);
}

#[test]
fn import_missing_git_repo_fails() {
  Test::new()
    .justfile("import 'https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git'")
    .status(EXIT_FAILURE)
    .stderr_regex("error: .*Authentication failed.*git repository.*")
    .run();
}

#[test]
fn import_invalid_git_url_fails() {
  Test::new()
    .justfile("import 'not-a-real-git-url://invalid'")
    .status(EXIT_FAILURE)
    .stderr_regex("error: Could not find source file for import.*")
    .run();
}

#[test]
fn import_with_invalid_sha_fails() {
  Test::new()
    .justfile("import 'https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git#invalidsha123'")
    .status(EXIT_FAILURE)
    .stderr_regex("error: .*Authentication failed.*git repository.*")
    .run();
}

#[test] 
fn optional_import_missing_git_repo_succeeds() {
  Test::new()
    .justfile(
      "
      import? 'https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git'
      
      test:
        @echo 'test succeeded'
      ",
    )
    .arg("test")
    .stdout("test succeeded\n")
    .run();
} 