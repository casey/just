use {
  super::*,
  pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd},
};

#[test]
fn heading_levels_are_not_skipped() {
  let markdown = fs::read_to_string("README.md").unwrap();

  let mut headings = Vec::new();
  let mut current = None;

  for event in Parser::new(&markdown) {
    match event {
      Event::Start(Tag::Heading { level, .. }) => current = Some((level, String::new())),
      Event::Text(text) => {
        if let Some((_, title)) = &mut current {
          title.push_str(&text);
        }
      }
      Event::End(TagEnd::Heading(_)) => headings.push(current.take().unwrap()),
      _ => {}
    }
  }

  let mut previous = HeadingLevel::H1;

  for (level, title) in headings {
    assert!(
      level as usize <= previous as usize + 1,
      "heading `{title}` skips from level {} to level {}",
      previous as usize,
      level as usize,
    );
    previous = level;
  }
}

#[test]
fn h1_and_h2_headings_are_setext() {
  let markdown = fs::read_to_string("README.md").unwrap();

  for (event, range) in Parser::new(&markdown).into_offset_iter() {
    if let Event::Start(Tag::Heading { level, .. }) = event {
      if matches!(level, HeadingLevel::H1 | HeadingLevel::H2) {
        let source = &markdown[range];
        assert!(
          !source.starts_with('#'),
          "{level:?} heading is not setext: {}",
          source.lines().next().unwrap(),
        );
      }
    }
  }
}

#[test]
fn readme() {
  let mut justfiles = Vec::new();
  let mut current = None;

  for line in fs::read_to_string("README.md").unwrap().lines() {
    if let Some(mut justfile) = current {
      if line == "```" {
        justfiles.push(justfile);
        current = None;
      } else {
        justfile += line;
        justfile += "\n";
        current = Some(justfile);
      }
    } else if line == "```just" {
      current = Some(String::new());
    }
  }

  for justfile in justfiles {
    let tmp = tempdir();

    let path = tmp.path().join("justfile");

    fs::write(path, justfile).unwrap();

    let output = Command::new(JUST)
      .current_dir(tmp.path())
      .arg("--dump")
      .output()
      .unwrap();

    assert_success(&output);
  }
}
