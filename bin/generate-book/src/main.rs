use {
  pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag},
  pulldown_cmark_to_cmark::cmark,
  std::{error::Error, fs, ops::Deref},
};

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

struct Chapter<'a> {
  level: HeadingLevel,
  events: Vec<Event<'a>>,
}

impl<'a> Chapter<'a> {
  fn title(&self) -> String {
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

  fn slug(&self) -> String {
    let mut slug = String::new();
    for c in self.title().chars() {
      match c {
        'A'..='Z' => slug.extend(c.to_lowercase()),
        ' ' => slug.push('-'),
        '.' => {}
        _ => slug.push(c),
      }
    }
    slug
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  for language in [Language::English, Language::Chinese] {
    let src = format!("book/{}/src", language.code());
    fs::remove_dir_all(&src).ok();
    fs::create_dir(&src)?;

    let txt = fs::read_to_string(format!("README{}.md", language.suffix()))?;

    let mut chapters = vec![Chapter {
      level: HeadingLevel::H1,
      events: Vec::new(),
    }];

    for event in Parser::new_ext(&txt, Options::all()) {
      if let Event::Start(Tag::Heading(level @ (HeadingLevel::H2 | HeadingLevel::H3), ..)) = event {
        chapters.push(Chapter {
          level,
          events: Vec::new(),
        });
      }
      chapters.last_mut().unwrap().events.push(event);
    }

    let mut summary = String::new();

    for (i, chapter) in chapters.into_iter().enumerate() {
      eprintln!("{} - {}", chapter.title(), chapter.slug());
      let mut txt = String::new();
      cmark(chapter.events.iter(), &mut txt)?;
      let title = if i == 0 {
        txt = txt.split_inclusive('\n').skip(1).collect::<String>();
        language.introduction()
      } else {
        txt.lines().next().unwrap().split_once(' ').unwrap().1
      };

      let path = format!("{}/chapter_{}.md", src, i + 1);
      fs::write(&path, &txt)?;
      let indent = match chapter.level {
        HeadingLevel::H1 => 0,
        HeadingLevel::H2 => 1,
        HeadingLevel::H3 => 2,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 4,
        HeadingLevel::H6 => 5,
      };
      summary.push_str(&format!(
        "{}- [{}](chapter_{}.md)\n",
        " ".repeat(indent * 4),
        title,
        i + 1
      ));
    }

    fs::write(format!("{}/SUMMARY.md", src), summary).unwrap();
  }

  Ok(())
}
