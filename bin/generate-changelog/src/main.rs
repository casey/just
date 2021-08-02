use std::{
  collections::BTreeMap,
  convert::TryInto,
  fmt::{self, Display, Formatter},
  fs, io,
  path::Path,
  process, str,
};

use ::{
  askama::Template,
  git2::{Oid, Repository, Time},
  regex::Regex,
  serde::{Deserialize, Serialize},
};

macro_rules! die {
  {$($input:tt)+} => {
    {
      eprint!("error: ");
      eprintln!($($input)*);
      process::exit(1);
    }
  }
}

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
    let mut commit_tag = None;

    repo.tag_foreach(|oid, tag| {
      if repo.find_tag(oid).unwrap().target_id() == commit.oid() {
        let tag = str::from_utf8(tag).unwrap();

        if let Some(captures) = Regex::new(r"^refs/tags/(v?[0-9]+\.[0-9]+\.[0-9]+)$")
          .unwrap()
          .captures(tag)
        {
          if commit_tag.is_some() {
            panic!("Duplicate tag for commit {}", commit.oid());
          }
          commit_tag = Some(captures[1].to_owned());
        } else {
          eprintln!("Tag is not release tag: {}", tag);
        }
      }

      true
    });

    if let Some(commit_tag) = commit_tag {
      self.releases.push(Release::new(Some(commit_tag)));
    } else if self.releases.is_empty() {
      self.releases.push(Release::new(None));
    }

    self.releases.last_mut().unwrap().add(commit);
  }
}

struct Release {
  tag:      Option<String>,
  sections: BTreeMap<CommitType, Section>,
}

impl Release {
  fn new(tag: Option<String>) -> Self {
    Self {
      sections: BTreeMap::new(),
      tag,
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
    let mut header = if self.tag.is_some() {
      match format {
        Format::Markdown => format!(
          "[{}](https://github.com/casey/just/releases/tag/{}) - {}",
          self.name(),
          self.tag(),
          self.date()
        ),
        Format::Text => format!("{} - {}", self.tag(), self.date()),
      }
    } else {
      String::from("Unreleased")
    };

    header.push('\n');
    header.push_str(&"-".repeat(header.len() - 1));
    header
  }

  fn date(&self) -> String {
    // TODO: fix
    "1970-1-1".into()
  }

  fn name(&self) -> &str {
    self.tag().strip_prefix("v").unwrap_or(self.tag())
  }

  fn tag(&self) -> &str {
    self.tag.as_ref().unwrap()
  }
}

#[derive(Default)]
struct Section {
  commits: Vec<Commit>,
}

impl Section {
  fn name(&self) -> String {
    // TODO: fix
    "name".into()
  }

  fn add(&mut self, commit: Commit) {
    self.commits.push(commit);
  }

  fn commits(&self) -> impl Iterator<Item = &Commit> {
    self.commits.iter()
  }
}

// TODO:
// - include release names and commits
// - remove `---` from yaml
// - hide merge prs
// - render .txt changelog
// - reenalbe writing metadata metadata.write("metadata.yaml");
// - print text changelog?

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
    fs::write(path, serde_yaml::to_string(self).unwrap());
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

// TODO: document commit types
// - added = user can do something new that they might like to know about
// - changed = something that user was doing changed in a way that they would
//   like to know about
// - breaking = something that used to work now no longer worked
// - fixed = somethign that didn't work now works
// - release = release commit
// - misc = anything that users either don't care about (development stuff,
//   refactors, testing infrastructure) or will discover without needing to look
//   for specifically (documentation fixes, error message fixes, etc)
// - merge = merge commit that shouldn't show up in changelog
// - require linear history after a point

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
  Uncategorized,
  Merge,
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
    changelog.add(&repo, commit);
  }

  fs::write("CHANGELOG.md", changelog.render().unwrap()).unwrap();

  let changelog = Changelog {
    format: Format::Text,
    ..changelog
  };

  fs::write("CHANGELOG.txt", changelog.render().unwrap()).unwrap();
}
