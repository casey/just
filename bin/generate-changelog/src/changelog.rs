use crate::common::*;

#[derive(Template)]
#[template(path = "changelog.md")]
pub(crate) struct Changelog {
  pub(crate) format:   Format,
  pub(crate) releases: Vec<Release>,
}

impl Changelog {
  pub(crate) fn new(format: Format) -> Self {
    Self {
      releases: Vec::new(),
      format,
    }
  }

  pub(crate) fn add(&mut self, repo: &Repository, commit: Commit) {
    if commit.ty == CommitType::Release {
      self.releases.push(Release::new(
        repo,
        commit
          .version
          .as_ref()
          .unwrap_or_else(|| panic!("No version for release {}", commit.oid())),
      ));
    }

    if let Some(release) = self.releases.last_mut() {
      release.add(commit);
    }
  }

  pub(crate) fn uncategorized_commit_count(&self) -> usize {
    self
      .releases
      .iter()
      .map(Release::uncategorized_commit_count)
      .sum()
  }
}
