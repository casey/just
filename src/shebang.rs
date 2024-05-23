use std::io::stdin;



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
    
    let pieces = Some(&line[2..]);
    let mut commands: Vec<Option<&str>> = stdin().lines()
    .filter(|l| l.unwrap().starts_with("#!"))
    .map(|pieces| pieces
        .unwrap_or("".to_string())
        .lines()
        .next()
        .unwrap()
        .trim()
        .splitn(2, |c| c == ' ' || c == '\t')
        .collect()
     );
      
      // collect into vector or concatenate together into single string
      
    let interpreter = pieces.unwrap_or("");
    let argument = pieces;

    if interpreter.is_empty() {
      return None;
    }

    Some(Self {
      interpreter,
      argument,
    })
  }

  fn interpreter_filename(&self) -> &str {
    self
      .interpreter
      .split(|c| matches!(c, '/' | '\\'))
      .last()
      .unwrap_or(self.interpreter)
  }

  pub(crate) fn script_filename(&self, recipe: &str) -> String {
    match self.interpreter_filename() {
      "cmd" | "cmd.exe" => format!("{recipe}.bat"),
      "powershell" | "powershell.exe" | "pwsh" | "pwsh.exe" => format!("{recipe}.ps1"),
      _ => recipe.to_owned(),
    }
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
  fn powershell_script_filename() {
    assert_eq!(
      Shebang::new("#!powershell").unwrap().script_filename("foo"),
      "foo.ps1"
    );
  }

  #[test]
  fn pwsh_script_filename() {
    assert_eq!(
      Shebang::new("#!pwsh").unwrap().script_filename("foo"),
      "foo.ps1"
    );
  }

  #[test]
  fn powershell_exe_script_filename() {
    assert_eq!(
      Shebang::new("#!powershell.exe")
        .unwrap()
        .script_filename("foo"),
      "foo.ps1"
    );
  }

  #[test]
  fn pwsh_exe_script_filename() {
    assert_eq!(
      Shebang::new("#!pwsh.exe").unwrap().script_filename("foo"),
      "foo.ps1"
    );
  }

  #[test]
  fn cmd_script_filename() {
    assert_eq!(
      Shebang::new("#!cmd").unwrap().script_filename("foo"),
      "foo.bat"
    );
  }

  #[test]
  fn cmd_exe_script_filename() {
    assert_eq!(
      Shebang::new("#!cmd.exe").unwrap().script_filename("foo"),
      "foo.bat"
    );
  }

  #[test]
  fn plain_script_filename() {
    assert_eq!(Shebang::new("#!bar").unwrap().script_filename("foo"), "foo");
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
