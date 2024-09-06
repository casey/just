#[derive(Copy, Clone)]
pub(crate) struct Shebang<'line> {
  pub(crate) interpreter: &'line str,
  pub(crate) argument: Option<&'line str>,
}

impl<'line> Shebang<'line> {
  pub(crate) fn new(line: &'line str) -> Option<Self> {
    if !line.starts_with("#!") {
      return None;
    }

    let mut pieces = line[2..]
      .lines()
      .next()
      .unwrap_or("")
      .trim()
      .splitn(2, [' ', '\t']);

    let interpreter = pieces.next().unwrap_or("");
    let argument = pieces.next();

    if interpreter.is_empty() {
      return None;
    }

    Some(Self {
      interpreter,
      argument,
    })
  }

  pub fn interpreter_filename(&self) -> &str {
    self
      .interpreter
      .split(['/', '\\'])
      .last()
      .unwrap_or(self.interpreter)
  }

  pub(crate) fn include_shebang_line(&self) -> bool {
    !(cfg!(windows) || matches!(self.interpreter_filename(), "cmd" | "cmd.exe"))
  }
}

#[cfg(test)]
mod tests {
  use super::Shebang;

  #[test]
  fn split_shebang() {
    fn check(text: &str, expected_split: Option<(&str, Option<&str>)>) {
      let shebang = Shebang::new(text);
      assert_eq!(
        shebang.map(|shebang| (shebang.interpreter, shebang.argument)),
        expected_split
      );
    }

    check("#!    ", None);
    check("#!", None);
    check("#!/bin/bash", Some(("/bin/bash", None)));
    check("#!/bin/bash    ", Some(("/bin/bash", None)));
    check(
      "#!/usr/bin/env python",
      Some(("/usr/bin/env", Some("python"))),
    );
    check(
      "#!/usr/bin/env python   ",
      Some(("/usr/bin/env", Some("python"))),
    );
    check(
      "#!/usr/bin/env python -x",
      Some(("/usr/bin/env", Some("python -x"))),
    );
    check(
      "#!/usr/bin/env python   -x",
      Some(("/usr/bin/env", Some("python   -x"))),
    );
    check(
      "#!/usr/bin/env python \t-x\t",
      Some(("/usr/bin/env", Some("python \t-x"))),
    );
    check("#/usr/bin/env python \t-x\t", None);
    check("#!  /bin/bash", Some(("/bin/bash", None)));
    check("#!\t\t/bin/bash    ", Some(("/bin/bash", None)));
    check(
      "#!  \t\t/usr/bin/env python",
      Some(("/usr/bin/env", Some("python"))),
    );
    check(
      "#!  /usr/bin/env python   ",
      Some(("/usr/bin/env", Some("python"))),
    );
    check(
      "#!  /usr/bin/env python -x",
      Some(("/usr/bin/env", Some("python -x"))),
    );
    check(
      "#!  /usr/bin/env python   -x",
      Some(("/usr/bin/env", Some("python   -x"))),
    );
    check(
      "#!  /usr/bin/env python \t-x\t",
      Some(("/usr/bin/env", Some("python \t-x"))),
    );
    check("#  /usr/bin/env python \t-x\t", None);
  }

  #[test]
  fn interpreter_filename_with_forward_slash() {
    assert_eq!(
      Shebang::new("#!/foo/bar/baz")
        .unwrap()
        .interpreter_filename(),
      "baz"
    );
  }

  #[test]
  fn interpreter_filename_with_backslash() {
    assert_eq!(
      Shebang::new("#!\\foo\\bar\\baz")
        .unwrap()
        .interpreter_filename(),
      "baz"
    );
  }

  #[test]
  fn dont_include_shebang_line_cmd() {
    assert!(!Shebang::new("#!cmd").unwrap().include_shebang_line());
  }

  #[test]
  fn dont_include_shebang_line_cmd_exe() {
    assert!(!Shebang::new("#!cmd.exe /C").unwrap().include_shebang_line());
  }

  #[test]
  #[cfg(not(windows))]
  fn include_shebang_line_other_not_windows() {
    assert!(Shebang::new("#!foo -c").unwrap().include_shebang_line());
  }

  #[test]
  #[cfg(windows)]
  fn include_shebang_line_other_windows() {
    assert!(!Shebang::new("#!foo -c").unwrap().include_shebang_line());
  }
}
