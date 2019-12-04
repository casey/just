pub(crate) struct Shebang<'line> {
  pub(crate) interpreter: &'line str,
  pub(crate) argument: Option<&'line str>,
}

impl<'line> Shebang<'line> {
  pub(crate) fn new(line: &'line str) -> Option<Shebang<'line>> {
    if !line.starts_with("#!") {
      return None;
    }

    let mut pieces = line[2..]
      .lines()
      .nth(0)
      .unwrap_or("")
      .trim()
      .splitn(2, |c| c == ' ' || c == '\t');

    let interpreter = pieces.next().unwrap_or("");
    let argument = pieces.next();

    if interpreter == "" {
      return None;
    }

    Some(Shebang {
      interpreter,
      argument,
    })
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
}
