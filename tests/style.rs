use super::*;

#[test]
fn style_command_default() {
  Test::new()
    .justfile(
      "
        foo:
          @echo '{{ style('command') }}foo{{NORMAL}}'
      ",
    )
    .stdout("\x1b[1mfoo\x1b[0m\n")
    .success();
}

#[test]
fn style_command_non_default() {
  Test::new()
    .justfile(
      "
        foo:
          @echo '{{ style('command') }}foo{{NORMAL}}'
      ",
    )
    .args(["--command-color", "red"])
    .stdout("\x1b[1;31mfoo\x1b[0m\n")
    .success();
}

#[test]
fn style_error() {
  Test::new()
    .justfile(
      "
        foo:
          @echo '{{ style('error') }}foo{{NORMAL}}'
      ",
    )
    .stdout("\x1b[1;31mfoo\x1b[0m\n")
    .success();
}

#[test]
fn style_warning() {
  Test::new()
    .justfile(
      "
        foo:
          @echo '{{ style('warning') }}foo{{NORMAL}}'
      ",
    )
    .stdout("\x1b[1;33mfoo\x1b[0m\n")
    .success();
}

#[test]
fn style_unknown() {
  Test::new()
    .justfile(
      "
        foo:
          @echo '{{ style('hippo') }}foo{{NORMAL}}'
      ",
    )
    .stderr(
      "
        error: call to function `style` failed: invalid style: `hippo`
         ——▶ justfile:2:13
          │
        2 │   @echo '{{ style('hippo') }}foo{{NORMAL}}'
          │             ^^^^^
      ",
    )
    .failure();
}

#[test]
fn style_single() {
  #[track_caller]
  fn case(name: &str, code: u8) {
    assert_eval_eq(
      &format!("style('{name}', 'foo')"),
      &format!("\x1b[{code}mfoo\x1b[0m"),
    );
  }

  case("black", 30);
  case("blue", 34);
  case("cyan", 36);
  case("green", 32);
  case("magenta", 35);
  case("red", 31);
  case("white", 37);
  case("yellow", 33);

  case("fg:black", 30);
  case("fg:blue", 34);
  case("fg:cyan", 36);
  case("fg:green", 32);
  case("fg:magenta", 35);
  case("fg:red", 31);
  case("fg:white", 37);
  case("fg:yellow", 33);

  case("bg:black", 40);
  case("bg:blue", 44);
  case("bg:cyan", 46);
  case("bg:green", 42);
  case("bg:magenta", 45);
  case("bg:red", 41);
  case("bg:white", 47);
  case("bg:yellow", 43);

  case("bold", 1);
  case("dim", 2);
  case("italic", 3);
  case("underline", 4);
  case("blink", 5);
  case("reverse", 7);
  case("hidden", 8);
  case("strikethrough", 9);
}

#[test]
fn style_list() {
  Test::new()
    .justfile(
      "
        set lists

        x := style(['bold', 'bg:blue', 'red'], 'foo')
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stdout("\x1b[1;44;31mfoo\x1b[0m")
    .unindent_stdout(false)
    .success();
}

#[test]
fn style_last_wins() {
  Test::new()
    .justfile(
      "
        set lists

        x := style(['red', 'green'], 'foo')
      ",
    )
    .env("JUST_UNSTABLE", "1")
    .args(["--evaluate", "x"])
    .stdout("\x1b[32mfoo\x1b[0m")
    .unindent_stdout(false)
    .success();
}

#[test]
fn style_prefix_without_text() {
  assert_eval_eq("style('red')", "\x1b[31m");
}

#[test]
fn style_with_text() {
  assert_eval_eq("style('error', 'foo')", "\x1b[1;31mfoo\x1b[0m");
}
