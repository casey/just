use crate::common::*;

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Metadata {
  pub(crate) commits: Vec<Commit>,
}

impl Metadata {
  pub(crate) fn read(path: &Path) -> Metadata {
    let yaml = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
  }

  pub(crate) fn write(&self, path: impl AsRef<Path>) {
    fs::write(
      path,
      serde_yaml::to_string(self)
        .unwrap()
        .strip_prefix("---\n")
        .unwrap(),
    )
    .unwrap();
  }

  pub(crate) fn commit_metadata(&self, oid: Oid) -> Option<&Commit> {
    for commit in &self.commits {
      if commit.hash == oid.as_bytes() {
        return Some(commit);
      }
    }

    None
  }

  pub(crate) fn add_uncategorized_commit(&mut self, oid: Oid, summary: &str) {
    if self.commit_metadata(oid).is_some() {
      panic!("Commit {} already has metadata", oid);
    }

    self.commits.push(Commit {
      hash:    oid.as_bytes().try_into().unwrap(),
      summary: summary.to_owned(),
      ty:      CommitType::Uncategorized,
      version: None,
    });
  }
}
