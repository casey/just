use super::*;

const CONSTANTS: [(&str, &str, &str); 27] = [
  ("HEX", "0123456789abcdef", "1.27.0"),
  ("HEXLOWER", "0123456789abcdef", "1.27.0"),
  ("HEXUPPER", "0123456789ABCDEF", "1.27.0"),
  ("CLEAR", "\x1bc", "master"),
  ("NORMAL", "\x1b[0m", "master"),
  ("BOLD", "\x1b[1m", "master"),
  ("ITALIC", "\x1b[3m", "master"),
  ("UNDERLINE", "\x1b[4m", "master"),
  ("INVERT", "\x1b[7m", "master"),
  ("HIDE", "\x1b[8m", "master"),
  ("STRIKETHROUGH", "\x1b[9m", "master"),
  ("BLACK", "\x1b[30m", "master"),
  ("RED", "\x1b[31m", "master"),
  ("GREEN", "\x1b[32m", "master"),
  ("YELLOW", "\x1b[33m", "master"),
  ("BLUE", "\x1b[34m", "master"),
  ("MAGENTA", "\x1b[35m", "master"),
  ("CYAN", "\x1b[36m", "master"),
  ("WHITE", "\x1b[37m", "master"),
  ("BG_BLACK", "\x1b[40m", "master"),
  ("BG_RED", "\x1b[41m", "master"),
  ("BG_GREEN", "\x1b[42m", "master"),
  ("BG_YELLOW", "\x1b[43m", "master"),
  ("BG_BLUE", "\x1b[44m", "master"),
  ("BG_MAGENTA", "\x1b[45m", "master"),
  ("BG_CYAN", "\x1b[46m", "master"),
  ("BG_WHITE", "\x1b[47m", "master"),
];

pub(crate) fn constants() -> &'static HashMap<&'static str, &'static str> {
  static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
  MAP.get_or_init(|| {
    CONSTANTS
      .into_iter()
      .map(|(name, value, _version)| (name, value))
      .collect()
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn readme_table() {
    println!("| Name | Value |");
    println!("|------|-------------|");
    for (name, value, version) in CONSTANTS {
      println!(
        "| `{name}`<sup>{version}</sup> | `\"{}\"` |",
        value.replace('\x1b', "\\e")
      );
    }
  }
}
