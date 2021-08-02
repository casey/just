use crate::common::*;

#[derive(Default)]
pub(crate) struct Section {
  pub(crate) commits: Vec<Commit>,
}

impl Section {
  pub(crate) fn add(&mut self, commit: Commit) {
    self.commits.push(commit);
  }

  pub(crate) fn uncategorized_commit_count(&self) -> usize {
    self
      .commits
      .iter()
      .filter(|commit| commit.ty == CommitType::Uncategorized)
      .count()
  }
}
