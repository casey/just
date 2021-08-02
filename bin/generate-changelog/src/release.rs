use crate::common::*;

pub(crate) struct Release {
  version:             Version,
  tag:                 String,
  pub(crate) sections: BTreeMap<CommitType, Section>,
  time:                Time,
}

impl Release {
  pub(crate) fn new(repo: &Repository, version: &Version) -> Self {
    let tag = repo
      .find_reference(&format!(
        "refs/tags/{}{}",
        if version <= &Version::new(0, 9, 4) {
          "v"
        } else {
          ""
        },
        version
      ))
      .unwrap()
      .peel_to_tag()
      .unwrap();

    Self {
      sections: BTreeMap::new(),
      version:  version.clone(),
      tag:      tag.name().unwrap().to_owned(),
      time:     tag.target().unwrap().peel_to_commit().unwrap().time(),
    }
  }

  pub(crate) fn add(&mut self, commit: Commit) {
    self
      .sections
      .entry(commit.ty)
      .or_insert_with(Section::default)
      .add(commit);
  }

  pub(crate) fn header(&self, format: &Format) -> String {
    let mut header = match format {
      Format::Markdown => format!(
        "[{}](https://github.com/casey/just/releases/tag/{}) - {}",
        self.version(),
        self.tag(),
        self.date()
      ),
      Format::Text => format!("{} - {}", self.version(), self.date()),
    };

    header.push('\n');
    header.push_str(&"-".repeat(header.len() - 1));
    header
  }

  fn date(&self) -> String {
    NaiveDateTime::from_timestamp(self.time.seconds(), 0)
      .format("%Y-%m-%d")
      .to_string()
  }

  fn version(&self) -> &Version {
    &self.version
  }

  fn tag(&self) -> &str {
    &self.tag
  }

  pub(crate) fn uncategorized_commit_count(&self) -> usize {
    self
      .sections
      .values()
      .map(Section::uncategorized_commit_count)
      .sum()
  }
}
