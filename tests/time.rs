use super::*;

fn duration_regex(recipe_name: &str) -> String {
  format!(r"---> {recipe_name} \(Duration: \d+\.\d+s\)\s")
}

#[test]
fn time_reports_time_when_specified() {
  Test::new()
    .justfile(
      "
        foo:
          @echo FOO
      ",
    )
    .arg("--time")
    .stdout("FOO\n")
    .stderr_regex(duration_regex("foo"))
    .success();
}

#[test]
fn time_reports_time_for_multiple_when_specified() {
  let foo_regex = duration_regex("foo");
  let bar_regex = duration_regex("bar");
  let stderr_regex = format!("{foo_regex}{bar_regex}");

  Test::new()
    .justfile(
      "
        foo:
          @echo FOO

        bar:
          @echo BAR
      ",
    )
    .args(["--time", "foo", "bar"])
    .stdout("FOO\nBAR\n")
    .stderr_regex(stderr_regex)
    .success();
}

#[test]
fn time_recursive_just_invocation_reports_time_if_specified() {
  let foo_regex = duration_regex("foo");
  let bar_regex = duration_regex("bar");
  let stderr_regex = format!("{bar_regex}{foo_regex}");

  Test::new()
    .justfile(format!(
      "
        foo:
          @{JUST} --time --justfile {{{{ justfile() }}}} bar

        bar:
          @echo BAR
      ",
    ))
    .args(["--time", "foo"])
    .stdout("BAR\n")
    .stderr_regex(stderr_regex)
    .success();
}

#[test]
fn time_recursive_just_invocation_does_not_report_time_if_not_specified() {
  Test::new()
    .justfile(format!(
      "
        foo:
          @{JUST} --justfile {{{{ justfile() }}}} bar

        bar:
          @echo BAR
      ",
    ))
    .args(["--time", "foo"])
    .stdout("BAR\n")
    .stderr_regex(duration_regex("foo"))
    .success();
}
