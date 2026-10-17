use super::*;

#[test]
fn long_help_prints_usage() {
  Test::new()
    .justfile(
      "
        [arg('bar', long, help='hello')]
        foo bar baz:
          echo bar={{ bar }} baz={{ baz }}
      ",
    )
    .args(["foo", "--help"])
    .stdout(
      "
        Usage: just foo [OPTIONS] baz

        Arguments:
          baz

        Options:
              --bar bar hello
      ",
    )
    .success();
}

#[test]
fn short_help_prints_usage() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        foo bar:
          echo bar={{ bar }}
      ",
    )
    .args(["foo", "-h"])
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
              --bar bar
      ",
    )
    .success();
}

#[test]
fn user_defined_help_options_take_precedence() {
  Test::new()
    .justfile(
      "
        [arg('bar', long='help')]
        [arg('baz', short='h')]
        @foo bar baz:
          echo bar={{ bar }} baz={{ baz }}
      ",
    )
    .args(["foo", "--help", "a", "-h", "b"])
    .stdout("bar=a baz=b\n")
    .success();
}

#[test]
fn help_is_positional_if_recipe_does_not_take_options() {
  Test::new()
    .justfile(
      "
        @foo bar:
          echo bar={{ bar }}
      ",
    )
    .args(["foo", "--help"])
    .stdout("bar=--help\n")
    .success();
}

#[test]
fn help_prevents_all_recipes_from_running() {
  Test::new()
    .justfile(
      "
        build:
          echo build

        [arg('bar', long)]
        test bar:
          echo test
      ",
    )
    .args(["build", "test", "--help"])
    .stdout(
      "
        Usage: just test [OPTIONS]

        Options:
              --bar bar
      ",
    )
    .success();
}

#[test]
fn first_help_wins() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        foo bar='baz':

        qux:
      ",
    )
    .args(["foo", "-h", "qux", "--bogus"])
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
              --bar bar [default: 'baz']
      ",
    )
    .success();
}

#[test]
fn help_takes_precedence_over_missing_arguments() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        foo bar baz:
      ",
    )
    .args(["foo", "-h"])
    .stdout(
      "
        Usage: just foo [OPTIONS] baz

        Arguments:
          baz

        Options:
              --bar bar
      ",
    )
    .success();
}

#[test]
fn combined_short_options_may_include_help() {
  Test::new()
    .justfile(
      "
        [arg('bar', short, value='baz')]
        foo bar:
      ",
    )
    .args(["foo", "-bh"])
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
          -b
      ",
    )
    .success();
}

#[test]
fn help_with_value_is_an_error() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        foo bar:
      ",
    )
    .args(["foo", "--help=baz"])
    .stderr("error: recipe `foo` flag `--help` does not take value\n")
    .failure();
}

#[test]
fn help_for_recipe_in_module() {
  Test::new()
    .justfile("mod bar")
    .write(
      "bar.just",
      "
        [arg('baz', long)]
        foo baz:
      ",
    )
    .args(["bar", "foo", "--help"])
    .stdout(
      "
        Usage: just bar foo [OPTIONS]

        Options:
              --baz baz
      ",
    )
    .success();
}

#[test]
fn help_is_printed_when_quiet() {
  Test::new()
    .justfile(
      "
        [arg('bar', long)]
        foo bar:
      ",
    )
    .args(["--quiet", "foo", "--help"])
    .stdout(
      "
        Usage: just foo [OPTIONS]

        Options:
              --bar bar
      ",
    )
    .success();
}
