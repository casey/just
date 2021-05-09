pub fn unescape(s: &str) -> Result<String, char> {
  let mut cooked = String::new();
  let mut escape = false;
  for c in s.chars() {
    if escape {
      match c {
        'n' => cooked.push('\n'),
        'r' => cooked.push('\r'),
        't' => cooked.push('\t'),
        '\\' => cooked.push('\\'),
        '\n' => {},
        '"' => cooked.push('"'),
        other => {
          return Err(other);
        },
      }
      escape = false;
    } else if c == '\\' {
      escape = true;
    } else {
      cooked.push(c);
    }
  }
  Ok(cooked)
}
