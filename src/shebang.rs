pub struct Shebang<'a> {
  pub interpreter: &'a str,
  pub argument:    Option<&'a str>,
}

impl<'a> Shebang<'a> {
  pub fn new(text: &'a str) -> Option<Shebang<'a>> {
    if !text.starts_with("#!") {
      return None;
    }

    let mut pieces = text[2..]
      .lines()
      .nth(0)
      .unwrap_or("")
      .trim()
      .splitn(2, |c| c == ' ' || c == '\t');

    let interpreter = pieces.next().unwrap_or("");
    let argument    = pieces.next();

    if interpreter == "" {
      return None;
    }

    Some(Shebang{interpreter, argument})
  }
}
