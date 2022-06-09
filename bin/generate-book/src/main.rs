use {
  pulldown_cmark::{
    Event,
    HeadingLevel::{H2, H3},
    Options, Parser, Tag,
  },
  pulldown_cmark_to_cmark::cmark,
  std::{error::Error, fs},
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
}

fn main() -> Result<(), Box<dyn Error>> {
  for language in [Language::English, Language::Chinese] {
    let src = format!("book/{}/src", language.code());
    fs::remove_dir_all(&src).ok();
    fs::create_dir(&src)?;

    let txt = fs::read_to_string(format!("README{}.md", language.suffix()))?;

    let mut chapters = vec![(1usize, Vec::new())];

    for event in Parser::new_ext(&txt, Options::all()) {
      if let Event::Start(Tag::Heading(level @ (H2 | H3), ..)) = event {
        chapters.push((if level == H2 { 2 } else { 3 }, Vec::new()));
      }
      chapters.last_mut().unwrap().1.push(event);
    }

    let mut summary = String::new();

    for (i, (level, chapter)) in chapters.into_iter().enumerate() {
      let mut txt = String::new();
      cmark(chapter.iter(), &mut txt)?;
      let title = if i == 0 {
        txt = txt.split_inclusive('\n').skip(1).collect::<String>();
        "Introduction"
      } else {
        txt.lines().next().unwrap().split_once(' ').unwrap().1
      };

      let path = format!("{}/chapter_{}.md", src, i + 1);
      fs::write(&path, &txt)?;
      summary.push_str(&format!(
        "{}- [{}](chapter_{}.md)\n",
        " ".repeat((level.saturating_sub(1)) * 4),
        title,
        i + 1
      ));
    }

    fs::write(format!("{}/SUMMARY.md", src), summary).unwrap();
  }

  Ok(())
}
