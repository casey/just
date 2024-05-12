use {
  pulldown_cmark::{CowStr, Event, HeadingLevel, Options, Parser, Tag},
  pulldown_cmark_to_cmark::cmark,
  std::{collections::BTreeMap, error::Error, fmt::Write, fs, ops::Deref},
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, Debug)]
enum Language {
  English,
  Chinese,
}

impl Language {
  fn code(&self) -> &'static str {
    match self {
      Self::English => "en",
      Self::Chinese => "zh",
    }
  }

  fn suffix(&self) -> &'static str {
    match self {
      Self::English => "",
      Self::Chinese => ".中文",
    }
  }

  fn introduction(&self) -> &'static str {
    match self {
      Self::Chinese => "说明",
      Self::English => "Introduction",
    }
  }
}

#[derive(Debug)]
struct Chapter<'a> {
  level: HeadingLevel,
  events: Vec<Event<'a>>,
  index: usize,
  language: Language,
}

impl<'a> Chapter<'a> {
  fn title(&self) -> String {
    if self.index == 0 {
      return self.language.introduction().into();
    }

    self
      .events
      .iter()
      .skip_while(|event| !matches!(event, Event::Start(Tag::Heading(..))))
      .skip(1)
      .take_while(|event| !matches!(event, Event::End(Tag::Heading(..))))
      .filter_map(|event| match event {
        Event::Code(content) | Event::Text(content) => Some(content.deref()),
        _ => None,
      })
      .collect()
  }

  fn number(&self) -> usize {
    self.index + 1
  }

  fn markdown(&self) -> Result<String> {
    let mut markdown = String::new();
    cmark(self.events.iter(), &mut markdown)?;
    if self.index == 0 {
      markdown = markdown.split_inclusive('\n').skip(1).collect::<String>();
    }
    Ok(markdown)
  }
}

fn slug(s: &str) -> String {
  let mut slug = String::new();
  for c in s.chars() {
    match c {
      'A'..='Z' => slug.extend(c.to_lowercase()),
      ' ' => slug.push('-'),
      '?' | '.' | '？' => {}
      _ => slug.push(c),
    }
  }
  slug
}

fn main() -> Result {
  for language in [Language::English, Language::Chinese] {
    let src = format!("book/{}/src", language.code());
    fs::remove_dir_all(&src).ok();
    fs::create_dir(&src)?;

    let txt = fs::read_to_string(format!("README{}.md", language.suffix()))?;

    let mut chapters = vec![Chapter {
      level: HeadingLevel::H1,
      events: vec![],
      index: 0,
      language,
    }];

    for event in Parser::new_ext(&txt, Options::all()) {
      if let Event::Start(Tag::Heading(level @ (HeadingLevel::H2 | HeadingLevel::H3), ..)) = event {
        let index = chapters.last().unwrap().index + 1;
        chapters.push(Chapter {
          level,
          events: vec![],
          index,
          language,
        });
      }
      chapters.last_mut().unwrap().events.push(event);
    }

    let mut links = BTreeMap::new();

    for chapter in &chapters {
      let mut current = None;
      for event in &chapter.events {
        match event {
          Event::Start(Tag::Heading(..)) => current = Some(vec![]),
          Event::End(Tag::Heading(level, ..)) => {
            let events = current.unwrap();
            let title = events
              .iter()
              .filter_map(|event| match event {
                Event::Code(content) | Event::Text(content) => Some(content.deref()),
                _ => None,
              })
              .collect::<String>();
            let slug = slug(&title);
            let link = if let HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3 = level {
              format!("chapter_{}.html", chapter.number())
            } else {
              format!("chapter_{}.html#{}", chapter.number(), slug)
            };
            links.insert(slug, link);
            current = None;
          }
          _ => {
            if let Some(events) = &mut current {
              events.push(event.clone());
            }
          }
        }
      }
    }

    for chapter in &mut chapters {
      for event in &mut chapter.events {
        if let Event::Start(Tag::Link(_, dest, _)) | Event::End(Tag::Link(_, dest, _)) = event {
          if let Some(anchor) = dest.clone().strip_prefix('#') {
            *dest = CowStr::Borrowed(&links[anchor]);
          }
        }
      }
    }

    let mut summary = String::new();

    for chapter in chapters {
      let path = format!("{}/chapter_{}.md", src, chapter.number());
      fs::write(path, chapter.markdown()?)?;
      let indent = match chapter.level {
        HeadingLevel::H1 => 0,
        HeadingLevel::H2 => 1,
        HeadingLevel::H3 => 2,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 4,
        HeadingLevel::H6 => 5,
      };
      writeln!(
        summary,
        "{}- [{}](chapter_{}.md)",
        " ".repeat(indent * 4),
        chapter.title(),
        chapter.number()
      )?;
    }

    fs::write(format!("{src}/SUMMARY.md"), summary).unwrap();
  }

  Ok(())
}
