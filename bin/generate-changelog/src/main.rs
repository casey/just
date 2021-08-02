use crate::common::*;

mod changelog;
mod commit;
mod commit_type;
mod common;
mod format;
mod metadata;
mod release;
mod section;

fn main() {
  let repo = Repository::open(".").unwrap();

  let metadata_path = Path::new("metadata.yaml");

  let mut metadata = Metadata::read(metadata_path);

  let mut revwalker = repo.revwalk().unwrap();

  revwalker.push_head().unwrap();

  let mut seen_categorized = false;

  for result in revwalker {
    let oid = result.unwrap();
    let summary = repo.find_commit(oid).unwrap().summary().unwrap().to_owned();

    match metadata.commit(oid) {
      Some(commit) => seen_categorized = commit.ty != CommitType::Uncategorized,
      None =>
        if seen_categorized {
          metadata.add_uncategorized_commit(oid, &summary);
        },
    }
  }

  metadata.write(metadata_path);

  let mut changelog = Changelog::new(Format::Markdown);

  for commit in metadata.commits {
    if commit.ty != CommitType::Merge {
      changelog.add(&repo, commit);
    }
  }

  let uncategorized = changelog.uncategorized_commit_count();

  if uncategorized > 0 {
    panic!("{} uncategorized commits, aborting…", uncategorized);
  }

  fs::write("CHANGELOG.md", changelog.render().unwrap()).unwrap();

  let changelog = Changelog {
    format: Format::Text,
    ..changelog
  };

  fs::write("CHANGELOG.txt", changelog.render().unwrap()).unwrap();
}
