use super::*;

#[test]
fn cmd_shell_receives_command_verbatim() {
  #[track_caller]
  fn case(line: &str, stdout: &str, stderr: &str) {
    Test::new()
      .justfile(format!(
        "set shell := ['cmd.exe', '/c']\n\nfoo:\n @{line}\n"
      ))
      .shell(false)
      .stdout(stdout)
      .stderr(stderr)
      .success();
  }

  case(r#"echo "foo bar""#, "\"foo bar\"\r\n", "");
  case(r#"echo {{ 'foo "bar" baz' }}"#, "foo \"bar\" baz\r\n", "");
  case(r#"echo "foo&bar""#, "\"foo&bar\"\r\n", "");
  case(r#"echo "foo"#, "\"foo\r\n", "");
  case(r#"echo "^""#, "\"^\"\r\n", "");
  case("echo foo&echo bar", "foo\r\nbar\r\n", "");
  case("echo foo&&echo bar", "foo\r\nbar\r\n", "");
  case("cmd /c exit 1||echo foo", "foo\r\n", "");
  case("echo ^&", "&\r\n", "");
  case("(echo foo)", "foo\r\n", "");
  case(r#"if "foo"=="foo" echo bar"#, "bar\r\n", "");
  case("for %i in (foo bar) do echo %i", "foo\r\nbar\r\n", "");
  case("echo foo!", "foo!\r\n", "");
  case("echo %qwerty%", "%qwerty%\r\n", "");
  case("echo foo|findstr foo", "foo\r\n", "");
  case("echo foo>&2", "", "foo\r\n");
}

#[test]
fn cmd_shell_expands_environment_variables() {
  Test::new()
    .justfile(
      "
        set shell := ['cmd.exe', '/c']

        foo:
          @echo %BAR%
      ",
    )
    .shell(false)
    .env("BAR", "baz")
    .stdout("baz\r\n")
    .success();
}

#[test]
fn cmd_shell_redirection() {
  Test::new()
    .justfile(
      "
        set shell := ['cmd.exe', '/c']

        foo:
          @echo bar>baz
          @type baz
      ",
    )
    .shell(false)
    .stdout("bar\r\n")
    .expect_file("baz", "bar\r\n")
    .success();
}

#[test]
fn bare_bash_in_shebang() {
  Test::new()
    .justfile(
      "
        default:
            #!bash
            echo FOO
      ",
    )
    .stdout("FOO\n")
    .success();
}
