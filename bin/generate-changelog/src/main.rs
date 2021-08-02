use std::{collections::BTreeMap, convert::TryInto, fs, path::Path, str};

use ::{
  askama::Template,
  chrono::NaiveDateTime,
  git2::{Oid, Repository, Time},
  regex::Regex,
  semver::Version,
  serde::{Deserialize, Serialize},
};

enum Format {
  Markdown,
  Text,
}

#[derive(Template)]
#[template(path = "changelog.md")]
struct Changelog {
  format:   Format,
  releases: Vec<Release>,
}

impl Changelog {
  fn new(format: Format) -> Self {
    Self {
      releases: Vec::new(),
      format,
    }
  }

  fn add(&mut self, repo: &Repository, commit: Commit) {
    if commit.ty == CommitType::Release {
      self.releases.push(Release::new(
        repo,
        commit
          .version
          .as_ref()
          .expect(&format!("No version for release {}", commit.oid())),
      ));
    }

    if let Some(release) = self.releases.last_mut() {
      release.add(commit);
    }
  }
}

struct Release {
  version:  Version,
  tag:      String,
  sections: BTreeMap<CommitType, Section>,
  time:     Time,
}

impl Release {
  fn new(repo: &Repository, version: &Version) -> Self {
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
      .expect(&format!(""))
      .peel_to_tag()
      .unwrap();

    Self {
      sections: BTreeMap::new(),
      version:  version.clone(),
      tag:      tag.name().unwrap().to_owned(),
      time:     tag.target().unwrap().peel_to_commit().unwrap().time(),
    }
  }

  fn add(&mut self, commit: Commit) {
    self
      .sections
      .entry(commit.ty)
      .or_insert(Section::default())
      .add(commit);
  }

  fn header(&self, format: &Format) -> String {
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
}

#[derive(Default)]
struct Section {
  commits: Vec<Commit>,
}

impl Section {
  fn add(&mut self, commit: Commit) {
    self.commits.push(commit);
  }
}

#[derive(Default, Deserialize, Serialize)]
struct Metadata {
  commits: Vec<Commit>,
}

impl Metadata {
  fn read(path: &Path) -> Metadata {
    let yaml = fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&yaml).unwrap()
  }

  fn write(&self, path: impl AsRef<Path>) {
    fs::write(
      path,
      serde_yaml::to_string(self)
        .unwrap()
        .remove_prefix("---\n")
        .unwrap(),
    )
    .unwrap();
  }

  fn commit_metadata(&self, oid: Oid) -> Option<&Commit> {
    for commit in &self.commits {
      if commit.hash == oid.as_bytes() {
        return Some(commit);
      }
    }

    None
  }

  fn add_uncategorized_commit(&mut self, oid: Oid, summary: &str) {
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

#[derive(Deserialize, Serialize)]
struct Commit {
  #[serde(with = "hex")]
  hash:    Vec<u8>,
  summary: String,
  #[serde(rename = "type")]
  ty:      CommitType,
  version: Option<Version>,
}

impl Commit {
  fn oid(&self) -> Oid {
    Oid::from_bytes(&self.hash).unwrap()
  }

  fn entry(&self, format: &Format) -> String {
    match format {
      Format::Markdown => {
        let (summary, pr) = if let Some(captures) = Regex::new(r"^(.*) \(#([1-9][0-9]*)\)$")
          .unwrap()
          .captures(&self.summary)
        {
          (
            captures.get(1).unwrap().as_str(),
            Some(captures[2].parse::<u64>().unwrap()),
          )
        } else {
          (self.summary.as_str(), None)
        };

        let mut entry = format!(
          "[{}](https://github.com/casey/just/commit/{})",
          summary,
          hex::encode(&self.hash)
        );

        if let Some(pr) = pr {
          entry.push_str(&format!(
            " [(#{})](https://github.com/casey/just/pull/{})",
            pr, pr
          ));
        }

        entry
      },
      Format::Text => self.summary.clone(),
    }
  }
}

#[derive(
  Deserialize, PartialEq, Serialize, Ord, Eq, PartialOrd, Copy, Clone, strum_macros::Display,
)]
#[serde(rename_all = "kebab-case")]
enum CommitType {
  Release,
  Breaking,
  Fixed,
  Changed,
  Added,
  Misc,
  Merge,
  Uncategorized,
}

fn main() {
  let repo = Repository::open(".").unwrap();

  let metadata_path = Path::new("metadata.yaml");

  let mut metadata = Metadata::read(metadata_path);

  let mut revwalker = repo.revwalk().unwrap();

  revwalker.push_head().unwrap();

  for result in revwalker {
    let oid = result.unwrap();
    let summary = repo.find_commit(oid).unwrap().summary().unwrap().to_owned();
    if metadata.commit_metadata(oid).is_none() {
      metadata.add_uncategorized_commit(oid, &summary);
    }
  }

  metadata.write(metadata_path);

  let uncategorized = metadata
    .commits
    .iter()
    .filter(|commit| commit.ty == CommitType::Uncategorized)
    .count();

  if uncategorized > 0 {
    panic!("{} uncategorized commits, aborting…", uncategorized);
  }

  let mut changelog = Changelog::new(Format::Markdown);

  for commit in metadata.commits {
    if commit.ty != CommitType::Merge {
      changelog.add(&repo, commit);
    }
  }

  fs::write("CHANGELOG.md", changelog.render().unwrap()).unwrap();

  let changelog = Changelog {
    format: Format::Text,
    ..changelog
  };

  fs::write("CHANGELOG.txt", changelog.render().unwrap()).unwrap();
}
