use std::{
  collections::BTreeMap,
  env, fs,
  io::Write,
  path::Path,
  process::{Command, Stdio},
  str,
};

use executable_path::executable_path;
use libc::{EXIT_FAILURE, EXIT_SUCCESS};
use pretty_assertions::assert_eq;
use test_utilities::{tempdir, unindent};

macro_rules! test {
  (
    name:     $name:ident,
    justfile: $justfile:expr,
    $(args:     ($($arg:tt)*),)?
    $(env:      {
      $($env_key:literal : $env_value:literal,)*
    },)?
    $(stdin:    $stdin:expr,)?
    $(stdout:   $stdout:expr,)?
    $(stderr:   $stderr:expr,)?
    $(status:   $status:expr,)?
    $(shell:    $shell:expr,)?
  ) => {
    #[test]
    fn $name() {
      #[allow(unused_mut)]
      let mut env = BTreeMap::new();

      $($(env.insert($env_key.to_string(), $env_value.to_string());)*)?

      Test {
        justfile: $justfile,
        $(args: &[$($arg)*],)?
        $(stdin: $stdin,)?
        $(stdout: $stdout,)?
        $(stderr: $stderr,)?
        $(status: $status,)?
        $(shell: $shell,)?
        env,
        ..Test::default()
      }.run();
    }
  }
}

struct Test<'a> {
  justfile: &'a str,
  args:     &'a [&'a str],
  env:      BTreeMap<String, String>,
  stdin:    &'a str,
  stdout:   &'a str,
  stderr:   &'a str,
  status:   i32,
  shell:    bool,
}

impl<'a> Default for Test<'a> {
  fn default() -> Test<'a> {
    Test {
      justfile: "",
      args:     &[],
      env:      BTreeMap::new(),
      stdin:    "",
      stdout:   "",
      stderr:   "",
      status:   EXIT_SUCCESS,
      shell:    true,
    }
  }
}

impl<'a> Test<'a> {
  fn run(self) {
    let tmp = tempdir();

    let justfile = unindent(self.justfile);
    let stdout = unindent(self.stdout);
    let stderr = unindent(self.stderr);

    let mut justfile_path = tmp.path().to_path_buf();
    justfile_path.push("justfile");
    fs::write(justfile_path, justfile).unwrap();

    let mut dotenv_path = tmp.path().to_path_buf();
    dotenv_path.push(".env");
    fs::write(dotenv_path, "DOTENV_KEY=dotenv-value").unwrap();

    let mut command = Command::new(&executable_path("just"));

    if self.shell {
      command.args(&["--shell", "bash"]);
    }

    let mut child = command
      .args(self.args)
      .envs(self.env)
      .current_dir(tmp.path())
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .expect("just invocation failed");

    {
      let mut stdin_handle = child.stdin.take().expect("failed to unwrap stdin handle");

      stdin_handle
        .write_all(self.stdin.as_bytes())
        .expect("failed to write stdin to just process");
    }

    let output = child
      .wait_with_output()
      .expect("failed to wait for just process");

    let have = Output {
      status: output.status.code().unwrap(),
      stdout: str::from_utf8(&output.stdout).unwrap(),
      stderr: str::from_utf8(&output.stderr).unwrap(),
    };

    let want = Output {
      status: self.status,
      stdout: &stdout,
      stderr: &stderr,
    };

    assert_eq!(have, want, "bad output");

    if self.status == EXIT_SUCCESS {
      test_round_trip(tmp.path());
    }
  }
}

#[derive(PartialEq, Debug)]
struct Output<'a> {
  stdout: &'a str,
  stderr: &'a str,
  status: i32,
}

fn test_round_trip(tmpdir: &Path) {
  println!("Reparsing...");

  let output = Command::new(&executable_path("just"))
    .current_dir(tmpdir)
    .arg("--dump")
    .output()
    .expect("just invocation failed");

  if !output.status.success() {
    panic!("dump failed: {}", output.status);
  }

  let dumped = String::from_utf8(output.stdout).unwrap();

  let reparsed_path = tmpdir.join("reparsed.just");

  fs::write(&reparsed_path, &dumped).unwrap();

  let output = Command::new(&executable_path("just"))
    .current_dir(tmpdir)
    .arg("--justfile")
    .arg(&reparsed_path)
    .arg("--dump")
    .output()
    .expect("just invocation failed");

  if !output.status.success() {
    panic!("reparse failed: {}", output.status);
  }

  let reparsed = String::from_utf8(output.stdout).unwrap();

  assert_eq!(reparsed, dumped, "reparse mismatch");
}

test! {
  name: alias_listing,
  justfile: "
    foo:
      echo foo

    alias f := foo
  ",
  args: ("--list"),
  stdout: "
    Available recipes:
        foo
        f   # alias for `foo`
  ",
}

test! {
  name: alias_listing_multiple_aliases,
  justfile: "foo:\n  echo foo\nalias f := foo\nalias fo := foo",
  args: ("--list"),
  stdout: "
    Available recipes:
        foo
        f   # alias for `foo`
        fo  # alias for `foo`
  ",
}

test! {
  name: alias_listing_parameters,
  justfile: "foo PARAM='foo':\n  echo {{PARAM}}\nalias f := foo",
  args: ("--list"),
  stdout: "
    Available recipes:
        foo PARAM='foo'
        f PARAM='foo'   # alias for `foo`
  ",
}

test! {
  name: alias_listing_private,
  justfile: "foo PARAM='foo':\n  echo {{PARAM}}\nalias _f := foo",
  args: ("--list"),
  stdout: "
    Available recipes:
        foo PARAM='foo'
  ",
}

test! {
  name: alias,
  justfile: "foo:\n  echo foo\nalias f := foo",
  args: ("f"),
  stdout: "foo\n",
  stderr: "echo foo\n",
}

test! {
  name: alias_with_parameters,
  justfile: "foo value='foo':\n  echo {{value}}\nalias f := foo",
  args: ("f", "bar"),
  stdout: "bar\n",
  stderr: "echo bar\n",
}

test! {
  name: alias_with_dependencies,
  justfile: "foo:\n  echo foo\nbar: foo\nalias b := bar",
  args: ("b"),
  stdout: "foo\n",
  stderr: "echo foo\n",
}

test! {
  name: duplicate_alias,
  justfile: "alias foo := bar\nalias foo := baz\n",
  stderr: "
    error: Alias `foo` first defined on line 1 is redefined on line 2
      |
    2 | alias foo := baz
      |       ^^^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: unknown_alias_target,
  justfile: "alias foo := bar\n",
  stderr: "
    error: Alias `foo` has an unknown target `bar`
      |
    1 | alias foo := bar
      |       ^^^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: alias_shadows_recipe,
  justfile: "bar:\n  echo bar\nalias foo := bar\nfoo:\n  echo foo",
  stderr: "
    error: Alias `foo` defined on line 3 shadows recipe `foo` defined on line 4
      |
    3 | alias foo := bar
      |       ^^^
  ",
  status: EXIT_FAILURE,
}

test! {
  name: alias_show,
  justfile: "foo:\n    bar\nalias f := foo",
  args: ("--show", "f"),
  stdout: "
    alias f := foo
    foo:
        bar
  ",
}

test! {
  name: alias_show_missing_target,
  justfile: "alias f := foo",
  args: ("--show", "f"),
  stderr: "
    error: Alias `f` has an unknown target `foo`
      |
    1 | alias f := foo
      |       ^
  ",
  status: EXIT_FAILURE,
}

test! {
  name:     default,
  justfile: "default:\n echo hello\nother: \n echo bar",
  stdout:   "hello\n",
  stderr:   "echo hello\n",
}

test! {
  name:     quiet,
  justfile: "default:\n @echo hello",
  stdout:   "hello\n",
}

test! {
  name:     verbose,
  justfile: "default:\n @echo hello",
  args:     ("--verbose"),
  stdout:   "hello\n",
  stderr:   "===> Running recipe `default`...\necho hello\n",
}

test! {
  name:     order,
  justfile: "
b: a
  echo b
  @mv a b

a:
  echo a
  @touch F
  @touch a

d: c
  echo d
  @rm c

c: b
  echo c
  @mv b c",
  args:     ("a", "d"),
  stdout:   "a\nb\nc\nd\n",
  stderr:   "echo a\necho b\necho c\necho d\n",
}

test! {
  name:     summary,
  justfile: "b: a
a:
d: c
c: b
_z: _y
_y:
",
  args:     ("--summary"),
  stdout:   "a b c d\n",
}

test! {
  name:     select,
  justfile: "b:
  @echo b
a:
  @echo a
d:
  @echo d
c:
  @echo c",
  args:     ("d", "c"),
  stdout:   "d\nc\n",
}

test! {
  name:     print,
  justfile: "b:
  echo b
a:
  echo a
d:
  echo d
c:
  echo c",
  args:     ("d", "c"),
  stdout:   "d\nc\n",
  stderr:   "echo d\necho c\n",
}

test! {
  name:     show,
  justfile: r#"hello := "foo"
bar := hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#,
  args:     ("--show", "recipe"),
  stdout:   r#"
    recipe:
        echo {{hello + "bar" + bar}}
  "#,
}

test! {
  name:     status_passthrough,
  justfile: "

hello:

recipe:
  @exit 100",
  args:     ("recipe"),
  stderr:   "error: Recipe `recipe` failed on line 6 with exit code 100\n",
  status:   100,
}

test! {
  name:     unknown_dependency,
  justfile: "bar:\nhello:\nfoo: bar baaaaaaaz hello",
  stderr:   "
    error: Recipe `foo` has unknown dependency `baaaaaaaz`
      |
    3 | foo: bar baaaaaaaz hello
      |          ^^^^^^^^^
  ",
  status:   EXIT_FAILURE,
}

test! {
  name:     backtick_success,
  justfile: "a := `printf Hello,`\nbar:\n printf '{{a + `printf ' world.'`}}'",
  stdout:   "Hello, world.",
  stderr:   "printf 'Hello, world.'\n",
}

test! {
  name:     backtick_trimming,
  justfile: "a := `echo Hello,`\nbar:\n echo '{{a + `echo ' world.'`}}'",
  stdout:   "Hello, world.\n",
  stderr:   "echo 'Hello, world.'\n",
}

test! {
  name:     backtick_code_assignment,
  justfile: "b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  stderr:   "
    error: Backtick failed with exit code 100
      |
    2 | a := `exit 100`
      |      ^^^^^^^^^^
  ",
  status:   100,
}

test! {
  name:     backtick_code_interpolation,
  justfile: "b := a\na := `echo hello`\nbar:\n echo '{{`exit 200`}}'",
  stderr:   "
    error: Backtick failed with exit code 200
      |
    4 |  echo '{{`exit 200`}}'
      |          ^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_mod,
  justfile: "f:\n 無{{`exit 200`}}",
  stderr:   "
    error: Backtick failed with exit code 200
      |
    2 |  無{{`exit 200`}}
      |      ^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_tab,
  justfile: "
backtick-fail:
\techo {{`exit 200`}}
",
  stderr:   "    error: Backtick failed with exit code 200
      |
    3 |     echo {{`exit 200`}}
      |            ^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_tabs,
  justfile: "
backtick-fail:
\techo {{\t`exit 200`}}
",
  stderr:   "error: Backtick failed with exit code 200
  |
3 |     echo {{    `exit 200`}}
  |                ^^^^^^^^^^
",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_inner_tab,
  justfile: "
backtick-fail:
\techo {{\t`exit\t\t200`}}
",
  stderr:   "
    error: Backtick failed with exit code 200
      |
    3 |     echo {{    `exit        200`}}
      |                ^^^^^^^^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_leading_emoji,
  justfile: "
backtick-fail:
\techo 😬{{`exit 200`}}
",
  stderr: "
    error: Backtick failed with exit code 200
      |
    3 |     echo 😬{{`exit 200`}}
      |              ^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_interpolation_unicode_hell,
  justfile: "
backtick-fail:
\techo \t\t\t😬鎌鼬{{\t\t`exit 200 # \t\t\tabc`}}\t\t\t😬鎌鼬
",
  stderr: "
    error: Backtick failed with exit code 200
      |
    3 |     echo             😬鎌鼬{{        `exit 200 #             abc`}}            😬鎌鼬
      |                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     backtick_code_long,
  justfile: "\n\n\n\n\n\nb := a\na := `echo hello`\nbar:\n echo '{{`exit 200`}}'",
  stderr:   "
    error: Backtick failed with exit code 200
       |
    10 |  echo '{{`exit 200`}}'
       |          ^^^^^^^^^^
  ",
  status:   200,
}

test! {
  name:     shebang_backtick_failure,
  justfile: "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
  stdout:   "",
  stderr:   "
    error: Backtick failed with exit code 123
      |
    4 |  echo {{`exit 123`}}
      |         ^^^^^^^^^^
  ",
  status:   123,
}

test! {
  name:     command_backtick_failure,
  justfile: "foo:
 echo hello
 echo {{`exit 123`}}",
  stdout:   "hello\n",
  stderr:   "
    echo hello
    error: Backtick failed with exit code 123
      |
    3 |  echo {{`exit 123`}}
      |         ^^^^^^^^^^
  ",
  status:   123,
}

test! {
  name:     assignment_backtick_failure,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
  stdout:   "",
  stderr:   "
    error: Backtick failed with exit code 222
      |
    4 | a := `exit 222`
      |      ^^^^^^^^^^
  ",
  status:   222,
}

test! {
  name:     unknown_override_options,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
  args:     ("--set", "foo", "bar", "--set", "baz", "bob", "--set", "a", "b", "a", "b"),
  stderr:   "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_override_args,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
  args:     ("foo=bar", "baz=bob", "a=b", "a", "b"),
  stderr:   "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_override_arg,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
  args:     ("foo=bar", "a=b", "a", "b"),
  stderr:   "error: Variable `foo` overridden on the command line but not present in justfile\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     overrides_first,
  justfile: r#"
foo := "foo"
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
  args:     ("foo=bar", "a=b", "recipe", "baz=bar"),
  stdout:   "arg=baz=bar\nbarbbaz\n",
  stderr:   "echo arg=baz=bar\necho barbbaz\n",
}

test! {
  name:     overrides_not_evaluated,
  justfile: r#"
foo := `exit 1`
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
  args:     ("foo=bar", "a=b", "recipe", "baz=bar"),
  stdout:   "arg=baz=bar\nbarbbaz\n",
  stderr:   "echo arg=baz=bar\necho barbbaz\n",
}

test! {
  name:     dry_run,
  justfile: r#"
var := `echo stderr 1>&2; echo backtick`

command:
  @touch /this/is/not/a/file
  {{var}}
  echo {{`echo command interpolation`}}

shebang:
  #!/bin/sh
  touch /this/is/not/a/file
  {{var}}
  echo {{`echo shebang interpolation`}}"#,
  args:     ("--dry-run", "shebang", "command"),
  stdout:   "",
  stderr:   "#!/bin/sh
touch /this/is/not/a/file
`echo stderr 1>&2; echo backtick`
echo `echo shebang interpolation`
touch /this/is/not/a/file
`echo stderr 1>&2; echo backtick`
echo `echo command interpolation`
",
}

test! {
  name:     evaluate,
  justfile: r#"
foo := "a\t"
hello := "c"
bar := "b\t"
ab := foo + bar + hello

wut:
  touch /this/is/not/a/file
"#,
  args:     ("--evaluate"),
  stdout:   r#"ab    := "a	b	c"
bar   := "b	"
foo   := "a	"
hello := "c"
"#,
}

test! {
  name:     export_success,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  echo $FOO $BAR $ABC
"#,
  stdout:   "a b abc\n",
  stderr:   "echo $FOO $BAR $ABC\n",
}

test! {
  name:     export_override,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + "-" + BAR + "-" + baz

wut:
  echo $FOO $BAR $ABC
"#,
  args:     ("--set", "BAR", "bye", "FOO=hello"),
  stdout:   "hello bye hello-bye-c\n",
  stderr:   "echo $FOO $BAR $ABC\n",
}

test! {
  name:     export_shebang,
  justfile: r#"
export FOO := "a"
baz := "c"
export BAR := "b"
export ABC := FOO + BAR + baz

wut:
  #!/bin/sh
  echo $FOO $BAR $ABC
"#,
  stdout:   "a b abc\n",
}

test! {
  name:     export_recipe_backtick,
  justfile: r#"
export EXPORTED_VARIABLE := "A-IS-A"

recipe:
  echo {{`echo recipe $EXPORTED_VARIABLE`}}
"#,
  stdout:   "recipe A-IS-A\n",
  stderr:   "echo recipe A-IS-A\n",
}

test! {
  name:     raw_string,
  justfile: r#"
export EXPORTED_VARIABLE := '\z'

recipe:
  printf "$EXPORTED_VARIABLE"
"#,
  stdout:   "\\z",
  stderr:   "printf \"$EXPORTED_VARIABLE\"\n",
}

test! {
  name:     line_error_spacing,
  justfile: r#"








???
"#,
  stdout:   "",
  stderr:   "error: Unknown start of token:
   |
10 | ???
   | ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     quiet_flag_no_stdout,
  justfile: r#"
default:
  @echo hello
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     quiet_flag_no_stderr,
  justfile: r#"
default:
  @echo hello 1>&2
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     quiet_flag_no_command_echoing,
  justfile: r#"
default:
  exit
"#,
  args:     ("--quiet"),
  stdout:   "",
}

test! {
  name:     quiet_flag_no_error_messages,
  justfile: r#"
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     quiet_flag_no_assignment_backtick_stderr,
  justfile: r#"
a := `echo hello 1>&2`
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     quiet_flag_no_interpolation_backtick_stderr,
  justfile: r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  status:   100,
}

test! {
  name:     argument_single,
  justfile: "
foo A:
  echo {{A}}
    ",
  args:     ("foo", "ARGUMENT"),
  stdout:   "ARGUMENT\n",
  stderr:   "echo ARGUMENT\n",
}

test! {
  name:     argument_multiple,
  justfile: "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE", "TWO"),
  stdout:   "A:ONE B:TWO\n",
  stderr:   "echo A:ONE B:TWO\n",
}

test! {
  name:     argument_mismatch_more,
  justfile: "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE", "TWO", "THREE"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `THREE`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     argument_mismatch_fewer,
  justfile: "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE"),
  stdout:   "",
  stderr:   "error: Recipe `foo` got 1 argument but takes 2\nusage:\n    just foo A B\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     argument_mismatch_more_with_default,
  justfile: "
foo A B='B':
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE", "TWO", "THREE"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `THREE`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     argument_mismatch_fewer_with_default,
  justfile: "
foo A B C='C':
  echo A:{{A}} B:{{B}} C:{{C}}
    ",
  args:     ("foo", "bar"),
  stdout:   "",
  stderr:   "
    error: Recipe `foo` got 1 argument but takes at least 2
    usage:
        just foo A B C='C'
  ",
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_recipe,
  justfile: "hello:",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `foo`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_recipes,
  justfile: "hello:",
  args:     ("foo", "bar"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipes `foo` or `bar`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     color_always,
  justfile: "b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "always"),
  stdout:   "",
  stderr:   "\u{1b}[1;31merror\u{1b}[0m: \u{1b}[1mBacktick failed with exit code 100
\u{1b}[0m  |\n2 | a := `exit 100`\n  |      \u{1b}[1;31m^^^^^^^^^^\u{1b}[0m\n",
  status:   100,
}

test! {
  name:     color_never,
  justfile: "b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "never"),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 100
  |
2 | a := `exit 100`
  |      ^^^^^^^^^^
",
  status:   100,
}

test! {
  name:     color_auto,
  justfile: "b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "auto"),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 100
  |
2 | a := `exit 100`
  |      ^^^^^^^^^^
",
  status:   100,
}

test! {
  name:     colors_no_context,
  justfile: "
recipe:
  @exit 100",
  args:     ("--color=always"),
  stdout:   "",
  stderr:   "\u{1b}[1;31merror\u{1b}[0m: \u{1b}[1m\
Recipe `recipe` failed on line 3 with exit code 100\u{1b}[0m\n",
  status:   100,
}

test! {
  name:     dump,
  justfile: r#"
# this recipe does something
recipe a b +d:
 @exit 100"#,
  args:     ("--dump"),
  stdout:   "# this recipe does something
recipe a b +d:
    @exit 100
",
}

test! {
  name:     mixed_whitespace,
  justfile: "bar:\n\t echo hello",
  stdout:   "",
  stderr:   "error: Found a mix of tabs and spaces in leading whitespace: `␉␠`
Leading whitespace may consist of tabs or spaces, but not both
  |
2 |      echo hello
  | ^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     extra_leading_whitespace,
  justfile: "bar:\n\t\techo hello\n\t\t\techo goodbye",
  stdout:   "",
  stderr:   "error: Recipe line has extra leading whitespace
  |
3 |             echo goodbye
  |         ^^^^^^^^^^^^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     inconsistent_leading_whitespace,
  justfile: "bar:\n\t\techo hello\n\t echo goodbye",
  stdout:   "",
  stderr:   "error: Recipe line has inconsistent leading whitespace. \
            Recipe started with `␉␉` but found line with `␉␠`
  |
3 |      echo goodbye
  | ^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     required_after_default,
  justfile: "bar:\nhello baz arg='foo' bar:",
  stdout:   "",
  stderr:   "error: Non-default parameter `bar` follows default parameter
  |
2 | hello baz arg='foo' bar:
  |                     ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     required_after_plus_variadic,
  justfile: "bar:\nhello baz +arg bar:",
  stdout:   "",
  stderr:   "error: Parameter `bar` follows variadic parameter
  |
2 | hello baz +arg bar:
  |                ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     required_after_star_variadic,
  justfile: "bar:\nhello baz *arg bar:",
  stdout:   "",
  stderr:   "error: Parameter `bar` follows variadic parameter
  |
2 | hello baz *arg bar:
  |                ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     use_string_default,
  justfile: r#"
bar:
hello baz arg="XYZ\t\"	":
  echo '{{baz}}...{{arg}}'
"#,
  args:     ("hello", "ABC"),
  stdout:   "ABC...XYZ\t\"\t\n",
  stderr:   "echo 'ABC...XYZ\t\"\t'\n",
}

test! {
  name:     use_raw_string_default,
  justfile: r#"
bar:
hello baz arg='XYZ"	':
  printf '{{baz}}...{{arg}}'
"#,
  args:     ("hello", "ABC"),
  stdout:   "ABC...XYZ\"\t",
  stderr:   "printf 'ABC...XYZ\"\t'\n",
}

test! {
  name:     supply_use_default,
  justfile: r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
  args:     ("hello", "0", "1"),
  stdout:   "0 1 C\n",
  stderr:   "echo 0 1 C\n",
}

test! {
  name:     supply_defaults,
  justfile: r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
  args:     ("hello", "0", "1", "2"),
  stdout:   "0 1 2\n",
  stderr:   "echo 0 1 2\n",
}

test! {
  name:     list,
  justfile: r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this comment will be ignored

a Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
  args:     ("--list"),
  stdout:   r#"
    Available recipes:
        a Z="\t z"
        hello a b='B	' c='C' # this does a thing
  "#,
}

test! {
  name:     list_alignment,
  justfile: r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# something else
a Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
  args:     ("--list"),
  stdout:   r#"
    Available recipes:
        a Z="\t z"          # something else
        hello a b='B	' c='C' # this does a thing
  "#,
}

test! {
  name:     list_alignment_long,
  justfile: r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this does another thing
x a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# something else
this-recipe-is-very-very-very-important Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
  args:     ("--list"),
  stdout:   r#"
    Available recipes:
        hello a b='B	' c='C' # this does a thing
        this-recipe-is-very-very-very-important Z="\t z" # something else
        x a b='B	' c='C'     # this does another thing
  "#,
}

test! {
  name:     show_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
  args:     ("--show", "hell"),
  stdout:   "",
  stderr:   "Justfile does not contain recipe `hell`.\nDid you mean `hello`?\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_alias_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
  args:     ("--show", "fo"),
  stdout:   "",
  stderr:   "Justfile does not contain recipe `fo`.\nDid you mean `foo`, an alias for `hello`?\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_no_suggestion,
  justfile: r#"
helloooooo a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
  args:     ("--show", "hell"),
  stdout:   "",
  stderr:   "Justfile does not contain recipe `hell`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     show_no_alias_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

alias foo := hello

a Z="\t z":
"#,
  args:     ("--show", "fooooooo"),
  stdout:   "",
  stderr:   "Justfile does not contain recipe `fooooooo`.\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     run_suggestion,
  justfile: r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
  args:     ("hell"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `hell`.\nDid you mean `hello`?\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     line_continuation_with_space,
  justfile: r#"
foo:
  echo a\
         b  \
             c
"#,
  stdout:   "ab c\n",
  stderr:   "echo ab  c\n",
}

test! {
  name:     line_continuation_with_quoted_space,
  justfile: r#"
foo:
  echo 'a\
         b  \
             c'
"#,
  stdout:   "ab  c\n",
  stderr:   "echo 'ab  c'\n",
}

test! {
  name:     line_continuation_no_space,
  justfile: r#"
foo:
  echo a\
  b\
  c
"#,
  stdout:   "abc\n",
  stderr:   "echo abc\n",
}

test! {
  name:     test_os_arch_functions_in_interpolation,
  justfile: r#"
foo:
  echo {{arch()}} {{os()}} {{os_family()}}
"#,
  stdout:   format!("{} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
  stderr:   format!("echo {} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
}

test! {
  name:     test_os_arch_functions_in_expression,
  justfile: r#"
a := arch()
o := os()
f := os_family()

foo:
  echo {{a}} {{o}} {{f}}
"#,
  stdout:   format!("{} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
  stderr:   format!("echo {} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
}

#[cfg(not(windows))]
test! {
  name:     env_var_functions,
  justfile: r#"
p := env_var('USER')
b := env_var_or_default('ZADDY', 'HTAP')
x := env_var_or_default('XYZ', 'ABC')

foo:
  /bin/echo '{{p}}' '{{b}}' '{{x}}'
"#,
  stdout:   format!("{} HTAP ABC\n", env::var("USER").unwrap()).as_str(),
  stderr:   format!("/bin/echo '{}' 'HTAP' 'ABC'\n", env::var("USER").unwrap()).as_str(),
}

#[cfg(windows)]
test! {
  name:     env_var_functions,
  justfile: r#"
p := env_var('USERNAME')
b := env_var_or_default('ZADDY', 'HTAP')
x := env_var_or_default('XYZ', 'ABC')

foo:
  /bin/echo '{{p}}' '{{b}}' '{{x}}'
"#,
  stdout:   format!("{} HTAP ABC\n", env::var("USERNAME").unwrap()).as_str(),
  stderr:   format!("/bin/echo '{}' 'HTAP' 'ABC'\n", env::var("USERNAME").unwrap()).as_str(),
}

test! {
  name:     env_var_failure,
  justfile: "a:\n  echo {{env_var('ZADDY')}}",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Call to function `env_var` failed: environment variable `ZADDY` not present
  |
2 |   echo {{env_var('ZADDY')}}
  |          ^^^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     quiet_recipe,
  justfile: r#"
@quiet:
  # a
  # b
  @echo c
"#,
  stdout:   "c\n",
  stderr:   "echo c\n",
}

test! {
  name:     quiet_shebang_recipe,
  justfile: r#"
@quiet:
  #!/bin/sh
  echo hello
"#,
  stdout:   "hello\n",
  stderr:   "#!/bin/sh\necho hello\n",
}

test! {
  name:     shebang_line_numbers,
  justfile: r#"
quiet:
  #!/usr/bin/env cat

  a

  b


  c


"#,
  stdout:   "#!/usr/bin/env cat



a

b


c
",
}

test! {
  name:     complex_dependencies,
  justfile: r#"
a: b
b:
c: b a
"#,
  args:     ("b"),
  stdout:   "",
}

test! {
  name:     parameter_shadows_variable,
  justfile: "FOO := 'hello'\na FOO:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Parameter `FOO` shadows variable of the same name
  |
2 | a FOO:
  |   ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_function_in_assignment,
  justfile: r#"foo := foo() + "hello"
bar:"#,
  args:     ("bar"),
  stdout:   "",
  stderr:   r#"error: Call to unknown function `foo`
  |
1 | foo := foo() + "hello"
  |        ^^^
"#,
  status:   EXIT_FAILURE,
}

test! {
  name:     dependency_takes_arguments_exact,
  justfile: "
    a FOO:
    b: a
  ",
  args:     ("b"),
  stdout:   "",
  stderr:   "error: Dependency `a` got 0 arguments but takes 1 argument
  |
2 | b: a
  |    ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     dependency_takes_arguments_at_least,
  justfile: "
    a FOO LUZ='hello':
    b: a
  ",
  args:     ("b"),
  stdout:   "",
  stderr:   "error: Dependency `a` got 0 arguments but takes at least 1 argument
  |
2 | b: a
  |    ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     dependency_takes_arguments_at_most,
  justfile: "
    a FOO LUZ='hello':
    b: (a '0' '1' '2')
  ",
  args:     ("b"),
  stdout:   "",
  stderr:   "error: Dependency `a` got 3 arguments but takes at most 2 arguments
  |
2 | b: (a '0' '1' '2')
  |     ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     duplicate_parameter,
  justfile: "a foo foo:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Recipe `a` has duplicate parameter `foo`
  |
1 | a foo foo:
  |       ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     duplicate_recipe,
  justfile: "b:\nb:",
  args:     ("b"),
  stdout:   "",
  stderr:   "error: Recipe `b` first defined on line 1 is redefined on line 2
  |
2 | b:
  | ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     duplicate_variable,
  justfile: "a := 'hello'\na := 'hello'\nfoo:",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Variable `a` has multiple definitions
  |
2 | a := 'hello'
  | ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unexpected_token_in_dependency_position,
  justfile: "foo: 'bar'",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Expected end of file, end of line, or identifier, but found raw string
  |
1 | foo: 'bar'
  |      ^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unexpected_token_after_name,
  justfile: "foo 'bar'",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Expected ':', ':=', identifier, or '+', but found raw string
  |
1 | foo 'bar'
  |     ^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     self_dependency,
  justfile: "a: a",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Recipe `a` depends on itself
  |
1 | a: a
  |    ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     long_circular_recipe_dependency,
  justfile: "a: b\nb: c\nc: d\nd: a",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Recipe `d` has circular dependency `a -> b -> c -> d -> a`
  |
4 | d: a
  |    ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     variable_self_dependency,
  justfile: "z := z\na:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `z` is defined in terms of itself
  |
1 | z := z
  | ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     variable_circular_dependency,
  justfile: "x := y\ny := z\nz := x\na:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `x` depends on its own value: `x -> y -> z -> x`
  |
1 | x := y
  | ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     invalid_escape_sequence,
  justfile: r#"x := "\q"
a:"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: `\\q` is not a valid escape sequence
  |
1 | x := \"\\q\"
  |      ^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     multiline_raw_string,
  justfile: "
string := 'hello
whatever'

a:
  echo '{{string}}'
",
  args:     ("a"),
  stdout:   "hello
whatever
",
  stderr:   "echo 'hello
whatever'
",
}

test! {
  name:     error_line_after_multiline_raw_string,
  justfile: "
string := 'hello

whatever' + 'yo'

a:
  echo '{{foo}}'
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `foo` not defined
  |
7 |   echo '{{foo}}'
  |           ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     error_column_after_multiline_raw_string,
  justfile: "
string := 'hello

whatever' + bar

a:
  echo '{{string}}'
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `bar` not defined
  |
4 | whatever' + bar
  |             ^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     multiline_raw_string_in_interpolation,
  justfile: r#"
a:
  echo '{{"a" + '
  ' + "b"}}'
"#,
  args:     ("a"),
  stdout:   "
    a
      b
  ",
  stderr:   "
    echo 'a
      b'
  ",
}

test! {
  name:     error_line_after_multiline_raw_string_in_interpolation,
  justfile: r#"
a:
  echo '{{"a" + '
  ' + "b"}}'

  echo {{b}}
"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `b` not defined
  |
6 |   echo {{b}}
  |          ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_raw_string,
  justfile: "
a b= ':
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Unterminated string
  |
2 | a b= ':
  |      ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_string,
  justfile: r#"
a b= ":
"#,
  args:     ("a"),
  stdout:   "",
  stderr:   r#"error: Unterminated string
  |
2 | a b= ":
  |      ^
"#,
  status:   EXIT_FAILURE,
}

test! {
  name:     plus_variadic_recipe,
  justfile: "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
}

test! {
  name:     plus_variadic_ignore_default,
  justfile: "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
}

test! {
  name:     plus_variadic_use_default,
  justfile: "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "0 1 HELLO\n",
  stderr:   "echo 0 1 HELLO\n",
}

test! {
  name:     plus_variadic_too_few,
  justfile: "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "",
  stderr:   "error: Recipe `a` got 2 arguments but takes at least 3\nusage:\n    just a x y +z\n",
  status:   EXIT_FAILURE,
}

test! {
  name:     star_variadic_recipe,
  justfile: "
a x y *z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
}

test! {
  name:     star_variadic_none,
  justfile: "
a x y *z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "0 1\n",
  stderr:   "echo 0 1 \n",
}

test! {
  name:     star_variadic_ignore_default,
  justfile: "
a x y *z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
}

test! {
  name:     star_variadic_use_default,
  justfile: "
a x y *z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "0 1 HELLO\n",
  stderr:   "echo 0 1 HELLO\n",
}

test! {
  name:     star_then_plus_variadic,
  justfile: "
foo *a +b:
  echo {{a}} {{b}}
",
  stdout:   "",
  stderr:   "error: Expected \':\' or \'=\', but found \'+\'
  |
2 | foo *a +b:
  |        ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     plus_then_star_variadic,
  justfile: "
foo +a *b:
  echo {{a}} {{b}}
",
  stdout:   "",
  stderr:   "error: Expected \':\' or \'=\', but found \'*\'
  |
2 | foo +a *b:
  |        ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     argument_grouping,
  justfile: "
FOO A B='blarg':
  echo foo: {{A}} {{B}}

BAR X:
  echo bar: {{X}}

BAZ +Z:
  echo baz: {{Z}}
",
  args:     ("BAR", "0", "FOO", "1", "2", "BAZ", "3", "4", "5"),
  stdout:   "bar: 0\nfoo: 1 2\nbaz: 3 4 5\n",
  stderr:   "echo bar: 0\necho foo: 1 2\necho baz: 3 4 5\n",
}

test! {
  name:     missing_second_dependency,
  justfile: "
x:

a: x y
",
  stdout:   "",
  stderr:   "error: Recipe `a` has unknown dependency `y`
  |
4 | a: x y
  |      ^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     list_colors,
  justfile: "
# comment
a B C +D='hello':
  echo {{B}} {{C}} {{D}}
",
  args:     ("--color", "always", "--list"),
  stdout:   "
    Available recipes:
        a \
    \u{1b}[36mB\u{1b}[0m \u{1b}[36mC\u{1b}[0m \u{1b}[35m+\
    \u{1b}[0m\u{1b}[36mD\u{1b}[0m=\u{1b}[32m'hello'\u{1b}[0m \
     \u{1b}[34m#\u{1b}[0m \u{1b}[34mcomment\u{1b}[0m
  ",
}

test! {
  name:     run_colors,
  justfile: "
# comment
a:
  echo hi
",
  args:     ("--color", "always", "--highlight", "--verbose"),
  stdout:   "hi\n",
  stderr:   "\u{1b}[1;36m===> Running recipe `a`...\u{1b}[0m\n\u{1b}[1mecho hi\u{1b}[0m\n",
}

test! {
  name:     no_highlight,
  justfile: "
# comment
a:
  echo hi
",
  args:     ("--color", "always", "--highlight", "--no-highlight", "--verbose"),
  stdout:   "hi\n",
  stderr:   "\u{1b}[1;36m===> Running recipe `a`...\u{1b}[0m\necho hi\n",
}

test! {
  name:     trailing_flags,
  justfile: "
echo A B C:
  echo {{A}} {{B}} {{C}}
",
  args:     ("echo", "--some", "--awesome", "--flags"),
  stdout:   "--some --awesome --flags\n",
  stderr:   "echo --some --awesome --flags\n",
}

test! {
   name:     comment_before_variable,
   justfile: "
#
A:='1'
echo:
  echo {{A}}
 ",
   args:     ("echo"),
   stdout:   "1\n",
   stderr:   "echo 1\n",
}

test! {
   name:     dotenv_variable_in_recipe,
   justfile: "
#
echo:
  echo $DOTENV_KEY
 ",
   stdout:   "dotenv-value\n",
   stderr:   "echo $DOTENV_KEY\n",
}

test! {
   name:     dotenv_variable_in_backtick,
   justfile: "
#
X:=`echo $DOTENV_KEY`
echo:
  echo {{X}}
 ",
   stdout:   "dotenv-value\n",
   stderr:   "echo dotenv-value\n",
}
test! {
   name:     dotenv_variable_in_function_in_recipe,
   justfile: "
#
echo:
  echo {{env_var_or_default('DOTENV_KEY', 'foo')}}
  echo {{env_var('DOTENV_KEY')}}
 ",
   stdout:   "dotenv-value\ndotenv-value\n",
   stderr:   "echo dotenv-value\necho dotenv-value\n",
}

test! {
   name:     dotenv_variable_in_function_in_backtick,
   justfile: "
#
X:=env_var_or_default('DOTENV_KEY', 'foo')
Y:=env_var('DOTENV_KEY')
echo:
  echo {{X}}
  echo {{Y}}
 ",
   stdout:   "dotenv-value\ndotenv-value\n",
   stderr:   "echo dotenv-value\necho dotenv-value\n",
}

test! {
   name:     no_dotenv,
   justfile: "
#
X:=env_var_or_default('DOTENV_KEY', 'DEFAULT')
echo:
  echo {{X}}
 ",
   args:     ("--no-dotenv"),
   stdout:   "DEFAULT\n",
   stderr:   "echo DEFAULT\n",
}

test! {
   name:     dotenv_env_var_override,
   justfile: "
#
echo:
  echo $DOTENV_KEY
 ",
   env:      {"DOTENV_KEY": "not-the-dotenv-value",},
   stdout:   "not-the-dotenv-value\n",
   stderr:   "echo $DOTENV_KEY\n",
}

test! {
   name:     invalid_escape_sequence_message,
   justfile: r#"
X := "\'"
"#,
   stdout:   "",
   stderr:   r#"error: `\'` is not a valid escape sequence
  |
2 | X := "\'"
  |      ^^^^
"#,
   status:   EXIT_FAILURE,
}

test! {
   name:     unknown_variable_in_default,
   justfile: "
foo x=bar:
",
   stdout:   "",
   stderr:   r#"error: Variable `bar` not defined
  |
2 | foo x=bar:
  |       ^^^
"#,
   status:   EXIT_FAILURE,
}

test! {
   name:     unknown_function_in_default,
   justfile: "
foo x=bar():
",
   stdout:   "",
   stderr:   r#"error: Call to unknown function `bar`
  |
2 | foo x=bar():
  |       ^^^
"#,
   status:   EXIT_FAILURE,
}

test! {
   name:     default_string,
   justfile: "
foo x='bar':
  echo {{x}}
",
   stdout:   "bar\n",
   stderr:   "echo bar\n",
}

test! {
   name:     default_concatination,
   justfile: "
foo x=(`echo foo` + 'bar'):
  echo {{x}}
",
   stdout:   "foobar\n",
   stderr:   "echo foobar\n",
}

test! {
   name:     default_backtick,
   justfile: "
foo x=`echo foo`:
  echo {{x}}
",
   stdout:   "foo\n",
   stderr:   "echo foo\n",
}

test! {
   name:     default_variable,
   justfile: "
y := 'foo'
foo x=y:
  echo {{x}}
",
   stdout:   "foo\n",
   stderr:   "echo foo\n",
}

test! {
  name:     test_os_arch_functions_in_default,
  justfile: r#"
foo a=arch() o=os() f=os_family():
  echo {{a}} {{o}} {{f}}
"#,
  stdout:   format!("{} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
  stderr:   format!("echo {} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
}

test! {
  name:     unterminated_interpolation_eol,
  justfile: "
foo:
  echo {{
",
  stderr:   r#"
    error: Unterminated interpolation
      |
    3 |   echo {{
      |        ^^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_interpolation_eof,
  justfile: "
foo:
  echo {{",
  stderr:   r#"
    error: Unterminated interpolation
      |
    3 |   echo {{
      |        ^^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unterminated_backtick,
  justfile: "
foo a=\t`echo blaaaaaah:
  echo {{a}}",
  stderr:   r#"
    error: Unterminated backtick
      |
    2 | foo a=    `echo blaaaaaah:
      |           ^
  "#,
  status:   EXIT_FAILURE,
}

test! {
  name:     unknown_start_of_token,
  justfile: "
assembly_source_files = $(wildcard src/arch/$(arch)/*.s)
",
  stderr:   r#"
    error: Unknown start of token:
      |
    2 | assembly_source_files = $(wildcard src/arch/$(arch)/*.s)
      |                         ^
  "#,
   status:   EXIT_FAILURE,
}

test! {
  name:     backtick_variable_cat,
  justfile: "
stdin := `cat`

default:
  echo {{stdin}}
",
  stdin:    "STDIN",
  stdout:   "STDIN\n",
  stderr:   "echo STDIN\n",
}

test! {
   name:     backtick_default_cat_stdin,
   justfile: "
default stdin = `cat`:
  echo {{stdin}}
",
   stdin:    "STDIN",
   stdout:   "STDIN\n",
   stderr:   "echo STDIN\n",
}

test! {
  name:     backtick_default_cat_justfile,
  justfile: "
default stdin = `cat justfile`:
  echo '{{stdin}}'
",
  stdout:   "

    default stdin = `cat justfile`:
      echo {{stdin}}
  ",
  stderr:   "
    echo '
    default stdin = `cat justfile`:
      echo '{{stdin}}''
  ",
}

test! {
   name:     backtick_variable_read_single,
   justfile: "
password := `read PW && echo $PW`

default:
  echo {{password}}
",
   stdin:    "foobar\n",
   stdout:   "foobar\n",
   stderr:   "echo foobar\n",
}

test! {
   name:     backtick_variable_read_multiple,
   justfile: "
a := `read A && echo $A`
b := `read B && echo $B`

default:
  echo {{a}}
  echo {{b}}
",
   stdin:    "foo\nbar\n",
   stdout:   "foo\nbar\n",
   stderr:   "echo foo\necho bar\n",
}

test! {
   name:     backtick_default_read_multiple,
   justfile: "

default a=`read A && echo $A` b=`read B && echo $B`:
  echo {{a}}
  echo {{b}}
",
   stdin:    "foo\nbar\n",
   stdout:   "foo\nbar\n",
   stderr:   "echo foo\necho bar\n",
}

test! {
  name: equals_deprecated_assignment,
  justfile: "
    foo = 'bar'

    default:
      echo {{foo}}
  ",
  stdout: "bar\n",
  stderr: "
    warning: `=` in assignments, exports, and aliases is being phased out on favor of `:=`
    Please see this issue for more details: https://github.com/casey/just/issues/379
      |
    1 | foo = 'bar'
      |     ^
    echo bar
  ",
}

test! {
  name: equals_deprecated_export,
  justfile: "
    export FOO = 'bar'

    default:
      echo $FOO
    ",
  stdout: "bar\n",
  stderr: "
    warning: `=` in assignments, exports, and aliases is being phased out on favor of `:=`
    Please see this issue for more details: https://github.com/casey/just/issues/379
      |
    1 | export FOO = 'bar'
      |            ^
    echo $FOO
  ",
}

test! {
  name: equals_deprecated_alias,
  justfile: "
    alias foo = default

    default:
      echo default
  ",
  args: ("foo"),
  stdout: "default\n",
  stderr: "
    warning: `=` in assignments, exports, and aliases is being phased out on favor of `:=`
    Please see this issue for more details: https://github.com/casey/just/issues/379
      |
    1 | alias foo = default
      |           ^
    echo default
  ",
}

test! {
  name: shell_args,
  justfile: "
    default:
      echo A${foo}A
  ",
  args: ("--shell-arg", "-c"),
  stdout: "AA\n",
  stderr: "echo A${foo}A\n",
  shell: false,
}

test! {
  name: shell_override,
  justfile: "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
  args: ("--shell", "bash"),
  stdout: "hello\n",
  stderr: "echo hello\n",
  shell: false,
}

test! {
  name: shell_arg_override,
  justfile: "
    set shell := ['foo-bar-baz']

    default:
      echo hello
  ",
  args: ("--shell-arg", "-cu"),
  stdout: "hello\n",
  stderr: "echo hello\n",
  shell: false,
}

#[cfg(unix)]
test! {
  name: set_shell,
  justfile: "
    set shell := ['echo', '-n']

    x := `bar`

    foo:
      echo {{x}}
      echo foo
  ",
  args: (),
  stdout: "echo barecho foo",
  stderr: "echo bar\necho foo\n",
  shell: false,
}

#[cfg(windows)]
test! {
  name: set_shell,
  justfile: "
    set shell := ['echo', '-n']

    x := `bar`

    foo:
      echo {{x}}
      echo foo
  ",
  args: (),
  stdout: "-n echo -n bar\r\r\n-n echo foo\r\n",
  stderr: "echo -n bar\r\necho foo\n",
  shell: false,
}

test! {
  name: dependency_argument_string,
  justfile: "
    release: (build 'foo') (build 'bar')

    build target:
      echo 'Building {{target}}...'
  ",
  args: (),
  stdout: "Building foo...\nBuilding bar...\n",
  stderr: "echo 'Building foo...'\necho 'Building bar...'\n",
  shell: false,
}

test! {
  name: dependency_argument_parameter,
  justfile: "
    default: (release '1.0')

    release version: (build 'foo' version) (build 'bar' version)

    build target version:
      echo 'Building {{target}}@{{version}}...'
  ",
  args: (),
  stdout: "Building foo@1.0...\nBuilding bar@1.0...\n",
  stderr: "echo 'Building foo@1.0...'\necho 'Building bar@1.0...'\n",
  shell: false,
}

test! {
  name: dependency_argument_function,
  justfile: "
    foo: (bar env_var_or_default('x', 'y'))

    bar arg:
      echo {{arg}}
  ",
  args: (),
  stdout: "y\n",
  stderr: "echo y\n",
  shell: false,
}

test! {
  name: dependency_argument_backtick,
  justfile: "
    export X := 'X'

    foo: (bar `echo $X`)

    bar arg:
      echo {{arg}}
      echo $X
  ",
  args: (),
  stdout: "X\nX\n",
  stderr: "echo X\necho $X\n",
  shell: false,
}

test! {
  name: dependency_argument_assignment,
  justfile: "
    v := '1.0'

    default: (release v)

    release version:
      echo Release {{version}}...
  ",
  args: (),
  stdout: "Release 1.0...\n",
  stderr: "echo Release 1.0...\n",
  shell: false,
}

test! {
  name: dependency_argument_plus_variadic,
  justfile: "
    foo: (bar 'A' 'B' 'C')

    bar +args:
      echo {{args}}
  ",
  args: (),
  stdout: "A B C\n",
  stderr: "echo A B C\n",
  shell: false,
}

test! {
  name: duplicate_dependency_no_args,
  justfile: "
    foo: bar bar bar bar

    bar:
      echo BAR
  ",
  args: (),
  stdout: "BAR\n",
  stderr: "echo BAR\n",
  shell: false,
}

test! {
  name: duplicate_dependency_argument,
  justfile: "
    foo: (bar 'BAR') (bar `echo BAR`)

    bar bar:
      echo {{bar}}
  ",
  args: (),
  stdout: "BAR\n",
  stderr: "echo BAR\n",
  shell: false,
}

test! {
  name: parameter_cross_reference_error,
  justfile: "
    foo:

    bar a b=a:
  ",
  args: (),
  stdout: "",
  stderr: "
    error: Variable `a` not defined
      |
    3 | bar a b=a:
      |         ^
  ",
  status: EXIT_FAILURE,
  shell: false,
}

#[cfg(windows)]
test! {
  name: pwsh_invocation_directory,
  justfile: r#"
    set shell := ["pwsh", "-NoProfile", "-c"]

    pwd:
      @Test-Path {{invocation_directory()}} > result.txt
  "#,
  args: (),
  stdout: "",
  stderr: "",
  status: EXIT_SUCCESS,
  shell: false,
}

test! {
  name: variables,
  justfile: "
    z := 'a'
    a := 'z'
  ",
  args: ("--variables"),
  stdout: "a z\n",
  stderr: "",
  shell: false,
}

test! {
  name: interpolation_evaluation_ignore_quiet,
  justfile: r#"
    foo:
      {{"@echo foo 2>/dev/null"}}
  "#,
  args: (),
  stdout: "",
  stderr: "
    @echo foo 2>/dev/null
    error: Recipe `foo` failed on line 2 with exit code 127
  ",
  status: 127,
  shell: false,
}

test! {
  name: interpolation_evaluation_ignore_quiet_continuation,
  justfile: r#"
    foo:
      {{""}}\
      @echo foo 2>/dev/null
  "#,
  args: (),
  stdout: "",
  stderr: "
    @echo foo 2>/dev/null
    error: Recipe `foo` failed on line 3 with exit code 127
  ",
  status: 127,
  shell: false,
}

#[cfg(windows)]
test! {
  name: windows_interpreter_path_no_base,
  justfile: r#"
    foo:
      #!powershell

      exit 0
  "#,
  args: (),
}
