use super::*;

const CONSTANTS: &[(&str, &str, Option<&str>, &str)] = &[
  ("HEX", "0123456789abcdef", None, "1.27.0"),
  ("HEXLOWER", "0123456789abcdef", None, "1.27.0"),
  ("HEXUPPER", "0123456789ABCDEF", None, "1.27.0"),
  ("PATH_SEP", "/", Some("\\"), "1.41.0"),
  ("PATH_VAR_SEP", ":", Some(";"), "1.41.0"),
  ("CLEAR", "\x1bc", None, "1.37.0"),
  ("NORMAL", "\x1b[0m", None, "1.37.0"),
  ("BOLD", "\x1b[1m", None, "1.37.0"),
  ("ITALIC", "\x1b[3m", None, "1.37.0"),
  ("UNDERLINE", "\x1b[4m", None, "1.37.0"),
  ("INVERT", "\x1b[7m", None, "1.37.0"),
  ("HIDE", "\x1b[8m", None, "1.37.0"),
  ("STRIKETHROUGH", "\x1b[9m", None, "1.37.0"),
  ("BLACK", "\x1b[30m", None, "1.37.0"),
  ("RED", "\x1b[31m", None, "1.37.0"),
  ("GREEN", "\x1b[32m", None, "1.37.0"),
  ("YELLOW", "\x1b[33m", None, "1.37.0"),
  ("BLUE", "\x1b[34m", None, "1.37.0"),
  ("MAGENTA", "\x1b[35m", None, "1.37.0"),
  ("CYAN", "\x1b[36m", None, "1.37.0"),
  ("WHITE", "\x1b[37m", None, "1.37.0"),
  ("BG_BLACK", "\x1b[40m", None, "1.37.0"),
  ("BG_RED", "\x1b[41m", None, "1.37.0"),
  ("BG_GREEN", "\x1b[42m", None, "1.37.0"),
  ("BG_YELLOW", "\x1b[43m", None, "1.37.0"),
  ("BG_BLUE", "\x1b[44m", None, "1.37.0"),
  ("BG_MAGENTA", "\x1b[45m", None, "1.37.0"),
  ("BG_CYAN", "\x1b[46m", None, "1.37.0"),
  ("BG_WHITE", "\x1b[47m", None, "1.37.0"),
];

pub(crate) fn constants() -> &'static HashMap<&'static str, &'static str> {
  static MAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
  MAP.get_or_init(|| {
    CONSTANTS
      .iter()
      .copied()
      .map(|(name, unix, windows, _version)| {
        (
          name,
          if cfg!(windows) {
            windows.unwrap_or(unix)
          } else {
            unix
          },
        )
      })
      .collect()
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn readme_table() {
    let mut table = Vec::<String>::new();
    table.push("| Name | Value | Value on Windows |".into());
    table.push("|---|---|---|".into());
    for (name, unix, windows, version) in CONSTANTS {
      table.push(format!(
        "| `{name}`<sup>{version}</sup> | `\"{}\"` | {} |",
        unix.replace('\x1b', "\\e"),
        windows
          .map(|value| format!("\"{value}\""))
          .unwrap_or_default(),
      ));
    }

    let table = table.join("\n");

    let readme = fs::read_to_string("README.md").unwrap();

    assert!(
      readme.contains(&table),
      "could not find table in readme:\n{table}",
    );
  }
}
