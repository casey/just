extern crate tempdir;
extern crate brev;

use ::prelude::*;
use tempdir::TempDir;
use std::str;

/// Instantiate integration tests for a given test case using
/// sh, dash, and bash.
///
/// Although `sh` is likely to be dash or bash, we include it
/// in case it's a different version or a different shell entirely.
///
/// For example, on FreeBSD, `sh` is ash.
macro_rules! integration_test {
  (
    name:     $name:ident,
    justfile: $text:tt,
    args:     ($($arg:tt)*),
    stdout:   $stdout:expr,
    stderr:   $stderr:expr,
    status:   $status:expr,
  ) => {
    mod $name {
      use ::prelude::*;
      use super::integration_test;

      // silence unused import warnings
      const __: i32 = EXIT_SUCCESS;

      #[test] fn sh()   { integration_test("sh",   $text, &[$($arg)*], $stdout, $stderr, $status); }
      #[test] fn dash() { integration_test("dash", $text, &[$($arg)*], $stdout, $stderr, $status); }
      #[test] fn bash() { integration_test("bash", $text, &[$($arg)*], $stdout, $stderr, $status); }
    }
  }
}

fn integration_test(
  shell:           &str,
  justfile:        &str,
  args:            &[&str],
  expected_stdout: &str,
  expected_stderr: &str,
  expected_status: i32,
) {
  let tmp = TempDir::new("just-integration")
    .unwrap_or_else(
      |err| panic!("integration test: failed to create temporary directory: {}", err));
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(path, justfile);

  let output = process::Command::new(&super::test_utils::just_binary_path())
    .current_dir(tmp.path())
    .args(&["--shell", shell])
    .args(args)
    .output()
    .expect("just invocation failed");

  let mut failure = false;

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout:\ngot:\n{}\n\nexpected:\n{}", stdout, expected_stdout);
    failure = true;
  }

  let stderr = str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr:\ngot:\n{}\n\nexpected:\n{}", stderr, expected_stderr);
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}

integration_test! {
  name:     default,
  justfile: "default:\n echo hello\nother: \n echo bar",
  args:     (),
  stdout:   "hello\n",
  stderr:   "echo hello\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet,
  justfile: "default:\n @echo hello",
  args:     (),
  stdout:   "hello\n",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     verbose,
  justfile: "default:\n @echo hello",
  args:     ("--verbose"),
  stdout:   "hello\n",
  stderr:   "===> Running recipe `default`...\necho hello\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
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
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     summary,
  justfile: "b: a
a:
d: c
c: b",
  args:     ("--summary"),
  stdout:   "a b c d\n",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
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
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
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
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     show,
  justfile: r#"hello = "foo"
bar = hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#,
  args:     ("--show", "recipe"),
  stdout:   r#"recipe:
    echo {{hello + "bar" + bar}}
"#,
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     status_passthrough,
  justfile: "

hello:

recipe:
  @exit 100",
  args:     ("recipe"),
  stdout:   "",
  stderr:   "error: Recipe `recipe` failed on line 6 with exit code 100\n",
  status:   100,
}

integration_test! {
  name:     unknown_dependency,
  justfile: "bar:\nhello:\nfoo: bar baaaaaaaz hello",
  args:     (),
  stdout:   "",
  stderr:   "error: Recipe `foo` has unknown dependency `baaaaaaaz`
  |
3 | foo: bar baaaaaaaz hello
  |          ^^^^^^^^^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     backtick_success,
  justfile: "a = `printf Hello,`\nbar:\n printf '{{a + `printf ' world!'`}}'",
  args:     (),
  stdout:   "Hello, world!",
  stderr:   "printf 'Hello, world!'\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     backtick_trimming,
  justfile: "a = `echo Hello,`\nbar:\n echo '{{a + `echo ' world!'`}}'",
  args:     (),
  stdout:   "Hello, world!\n",
  stderr:   "echo 'Hello, world!'\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     backtick_code_assignment,
  justfile: "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  status:   100,
}

integration_test! {
  name:     backtick_code_interpolation,
  justfile: "b = a\na = `echo hello`\nbar:\n echo '{{`exit 200`}}'",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 200
  |
4 |  echo '{{`exit 200`}}'
  |          ^^^^^^^^^^
",
  status:   200,
}

integration_test! {
  name:     backtick_code_interpolation_tab,
  justfile: "
backtick-fail:
\techo {{`exit 1`}}
",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 1
  |
3 |     echo {{`exit 1`}}
  |            ^^^^^^^^
",
  status:   1,
}

integration_test! {
  name:     backtick_code_interpolation_tabs,
  justfile: "
backtick-fail:
\techo {{\t`exit 1`}}
",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 1
  |
3 |     echo {{    `exit 1`}}
  |                ^^^^^^^^
",
  status:   1,
}

integration_test! {
  name:     backtick_code_interpolation_inner_tab,
  justfile: "
backtick-fail:
\techo {{\t`exit\t\t1`}}
",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 1
  |
3 |     echo {{    `exit        1`}}
  |                ^^^^^^^^^^^^^^^
",
  status:   1,
}

integration_test! {
  name:     backtick_code_interpolation_leading_emoji,
  justfile: "
backtick-fail:
\techo 😬{{`exit 1`}}
",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 1
  |
3 |     echo 😬{{`exit 1`}}
  |              ^^^^^^^^
",
  status:   1,
}

integration_test! {
  name:     backtick_code_interpolation_unicode_hell,
  justfile: "
backtick-fail:
\techo \t\t\t😬鎌鼬{{\t\t`exit 1 # \t\t\tabc`}}\t\t\t😬鎌鼬
",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 1
  |
3 |     echo             😬鎌鼬{{        `exit 1 #             abc`}}            😬鎌鼬
  |                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^
",
  status:   1,
}

integration_test! {
  name:     backtick_code_long,
  justfile: "\n\n\n\n\n\nb = a\na = `echo hello`\nbar:\n echo '{{`exit 200`}}'",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 200
   |
10 |  echo '{{`exit 200`}}'
   |          ^^^^^^^^^^
",
  status:   200,
}

integration_test! {
  name:     shebang_backtick_failure,
  justfile: "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 123
  |
4 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  status:   123,
}

integration_test! {
  name:     command_backtick_failure,
  justfile: "foo:
 echo hello
 echo {{`exit 123`}}",
  args:     (),
  stdout:   "hello\n",
  stderr:   "echo hello\nerror: Backtick failed with exit code 123
  |
3 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  status:   123,
}

integration_test! {
  name:     assignment_backtick_failure,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
  args:     (),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 222
  |
4 | a = `exit 222`
  |     ^^^^^^^^^^
",
  status:   222,
}

integration_test! {
  name:     unknown_override_options,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
  args:     ("--set", "foo", "bar", "--set", "baz", "bob", "--set", "a", "b", "a", "b"),
  stdout:   "",
  stderr:   "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unknown_override_args,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
  args:     ("foo=bar", "baz=bob", "a=b", "a", "b"),
  stdout:   "",
  stderr:   "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unknown_override_arg,
  justfile: "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
  args:     ("foo=bar", "a=b", "a", "b"),
  stdout:   "",
  stderr:   "error: Variable `foo` overridden on the command line but not present in justfile\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     overrides_first,
  justfile: r#"
foo = "foo"
a = "a"
baz = "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
  args:     ("foo=bar", "a=b", "recipe", "baz=bar"),
  stdout:   "arg=baz=bar\nbarbbaz\n",
  stderr:   "echo arg=baz=bar\necho barbbaz\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     overrides_not_evaluated,
  justfile: r#"
foo = `exit 1`
a = "a"
baz = "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
  args:     ("foo=bar", "a=b", "recipe", "baz=bar"),
  stdout:   "arg=baz=bar\nbarbbaz\n",
  stderr:   "echo arg=baz=bar\necho barbbaz\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     dry_run,
  justfile: r#"
var = `echo stderr 1>&2; echo backtick`

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
  stderr:   "stderr
#!/bin/sh
touch /this/is/not/a/file
backtick
echo shebang interpolation
touch /this/is/not/a/file
backtick
echo command interpolation
",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     evaluate,
  justfile: r#"
foo = "a\t"
hello = "c"
bar = "b\t"
ab = foo + bar + hello

wut:
  touch /this/is/not/a/file
"#,
  args:     ("--evaluate"),
  stdout:   r#"ab    = "a	b	c"
bar   = "b	"
foo   = "a	"
hello = "c"
"#,
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     export_success,
  justfile: r#"
export FOO = "a"
baz = "c"
export BAR = "b"
export ABC = FOO + BAR + baz

wut:
  echo $FOO $BAR $ABC
"#,
  args:     (),
  stdout:   "a b abc\n",
  stderr:   "echo $FOO $BAR $ABC\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     export_override,
  justfile: r#"
export FOO = "a"
baz = "c"
export BAR = "b"
export ABC = FOO + "-" + BAR + "-" + baz

wut:
  echo $FOO $BAR $ABC
"#,
  args:     ("--set", "BAR", "bye", "FOO=hello"),
  stdout:   "hello bye hello-bye-c\n",
  stderr:   "echo $FOO $BAR $ABC\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     outer_shebang,
  justfile: r#"#!/lol/wut
export FOO = "a"
baz = "c"
export BAR = "b"
export ABC = FOO + BAR + baz

wut:
  #!/bin/sh
  echo $FOO $BAR $ABC
"#,
  args:     (),
  stdout:   "",
  stderr:   "error: `#!` is reserved syntax outside of recipes
  |
1 | #!/lol/wut
  | ^
",
  status:   EXIT_FAILURE,
}


integration_test! {
  name:     export_shebang,
  justfile: r#"
export FOO = "a"
baz = "c"
export BAR = "b"
export ABC = FOO + BAR + baz

wut:
  #!/bin/sh
  echo $FOO $BAR $ABC
"#,
  args:     (),
  stdout:   "a b abc\n",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     export_recipe_backtick,
  justfile: r#"
export EXPORTED_VARIABLE = "A-IS-A"

recipe:
  echo {{`echo recipe $EXPORTED_VARIABLE`}}
"#,
  args:     (),
  stdout:   "recipe A-IS-A\n",
  stderr:   "echo recipe A-IS-A\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     raw_string,
  justfile: r#"
export EXPORTED_VARIABLE = '\z'

recipe:
  printf "$EXPORTED_VARIABLE"
"#,
  args:     (),
  stdout:   "\\z",
  stderr:   "printf \"$EXPORTED_VARIABLE\"\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     line_error_spacing,
  justfile: r#"








???
"#,
  args:     (),
  stdout:   "",
  stderr:   "error: Unknown start of token:
   |
10 | ???
   | ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     quiet_flag_no_stdout,
  justfile: r#"
default:
  @echo hello
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet_flag_no_stderr,
  justfile: r#"
default:
  @echo hello 1>&2
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet_flag_no_command_echoing,
  justfile: r#"
default:
  exit
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet_flag_no_error_messages,
  justfile: r#"
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   100,
}

integration_test! {
  name:     quiet_flag_no_assignment_backtick_stderr,
  justfile: r#"
a = `echo hello 1>&2`
default:
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   100,
}

integration_test! {
  name:     quiet_flag_no_interpolation_backtick_stderr,
  justfile: r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
  args:     ("--quiet"),
  stdout:   "",
  stderr:   "",
  status:   100,
}

integration_test! {
  name:     quiet_flag_or_dry_run_flag,
  justfile: "",
  args:     ("--quiet", "--dry-run"),
  stdout:   "",
  stderr:   &format!("error: The argument '--dry-run' cannot be used with '--quiet'

USAGE:
    just{} --color <COLOR> --quiet --shell <SHELL>

For more information try --help\n", env::consts::EXE_SUFFIX),
  status:   1,
}

integration_test! {
  name:     argument_single,
  justfile: "
foo A:
  echo {{A}}
    ",
  args:     ("foo", "ARGUMENT"),
  stdout:   "ARGUMENT\n",
  stderr:   "echo ARGUMENT\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     argument_multiple,
  justfile: "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE", "TWO"),
  stdout:   "A:ONE B:TWO\n",
  stderr:   "echo A:ONE B:TWO\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
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

integration_test! {
  name:     argument_mismatch_fewer,
  justfile: "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
  args:     ("foo", "ONE"),
  stdout:   "",
  stderr:   "error: Recipe `foo` got 1 argument but takes 2\n",
  status:   EXIT_FAILURE,
}

integration_test! {
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

integration_test! {
  name:     argument_mismatch_fewer_with_default,
  justfile: "
foo A B C='C':
  echo A:{{A}} B:{{B}} C:{{C}}
    ",
  args:     ("foo", "bar"),
  stdout:   "",
  stderr:   "error: Recipe `foo` got 1 argument but takes at least 2\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unknown_recipe,
  justfile: "hello:",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipe `foo`.\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unknown_recipes,
  justfile: "hello:",
  args:     ("foo", "bar"),
  stdout:   "",
  stderr:   "error: Justfile does not contain recipes `foo` or `bar`.\n",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     color_always,
  justfile: "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "always"),
  stdout:   "",
  stderr:   "\u{1b}[1;31merror:\u{1b}[0m \u{1b}[1mBacktick failed with exit code 100
\u{1b}[0m  |\n2 | a = `exit 100`\n  |     \u{1b}[1;31m^^^^^^^^^^\u{1b}[0m\n",
  status:   100,
}

integration_test! {
  name:     color_never,
  justfile: "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "never"),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  status:   100,
}

integration_test! {
  name:     color_auto,
  justfile: "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
  args:     ("--color", "auto"),
  stdout:   "",
  stderr:   "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  status:   100,
}

integration_test! {
  name:     colors_no_context,
  justfile: "
recipe:
  @exit 100",
  args:     ("--color=always"),
  stdout:   "",
  stderr:   "\u{1b}[1;31merror:\u{1b}[0m \u{1b}[1m\
Recipe `recipe` failed on line 3 with exit code 100\u{1b}[0m\n",
  status:   100,
}

integration_test! {
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
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     mixed_whitespace,
  justfile: "bar:\n\t echo hello",
  args:     (),
  stdout:   "",
  stderr:   "error: Found a mix of tabs and spaces in leading whitespace: `␉␠`
Leading whitespace may consist of tabs or spaces, but not both
  |
2 |      echo hello
  | ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     extra_leading_whitespace,
  justfile: "bar:\n\t\techo hello\n\t\t\techo goodbye",
  args:     (),
  stdout:   "",
  stderr:   "error: Recipe line has extra leading whitespace
  |
3 |             echo goodbye
  |         ^^^^^^^^^^^^^^^^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     inconsistent_leading_whitespace,
  justfile: "bar:\n\t\techo hello\n\t echo goodbye",
  args:     (),
  stdout:   "",
  stderr:   "error: Recipe line has inconsistent leading whitespace. \
            Recipe started with `␉␉` but found line with `␉␠`
  |
3 |      echo goodbye
  | ^
",
  status:   EXIT_FAILURE,
}


integration_test! {
  name:     required_after_default,
  justfile: "bar:\nhello baz arg='foo' bar:",
  args:     (),
  stdout:   "",
  stderr:   "error: Non-default parameter `bar` follows default parameter
  |
2 | hello baz arg='foo' bar:
  |                     ^^^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     required_after_variadic,
  justfile: "bar:\nhello baz +arg bar:",
  args:     (),
  stdout:   "",
  stderr:   "error: Parameter `bar` follows variadic parameter
  |
2 | hello baz +arg bar:
  |                ^^^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     use_string_default,
  justfile: r#"
bar:
hello baz arg="XYZ\t\"	":
  echo '{{baz}}...{{arg}}'
"#,
  args:     ("hello", "ABC"),
  stdout:   "ABC...XYZ\t\"\t\n",
  stderr:   "echo 'ABC...XYZ\t\"\t'\n",
  status:   EXIT_SUCCESS,
}


integration_test! {
  name:     use_raw_string_default,
  justfile: r#"
bar:
hello baz arg='XYZ\t"	':
  printf '{{baz}}...{{arg}}'
"#,
  args:     ("hello", "ABC"),
  stdout:   "ABC...XYZ\t\"\t",
  stderr:   "printf 'ABC...XYZ\\t\"\t'\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     supply_use_default,
  justfile: r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
  args:     ("hello", "0", "1"),
  stdout:   "0 1 C\n",
  stderr:   "echo 0 1 C\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     supply_defaults,
  justfile: r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
  args:     ("hello", "0", "1", "2"),
  stdout:   "0 1 2\n",
  stderr:   "echo 0 1 2\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     list,
  justfile: r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this comment will be ignored

a Z="\t z":
"#,
  args:     ("--list"),
  stdout:   r"Available recipes:
    a Z='\t z'
    hello a b='B\t' c='C' # this does a thing
",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
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

integration_test! {
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

integration_test! {
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

integration_test! {
  name:     line_continuation_with_space,
  justfile: r#"
foo:
  echo a\
         b  \
             c
"#,
  args:     (),
  stdout:   "a b c\n",
  stderr:   "echo a       b             c\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     line_continuation_with_quoted_space,
  justfile: r#"
foo:
  echo 'a\
         b  \
             c'
"#,
  args:     (),
  stdout:   "a       b             c\n",
  stderr:   "echo 'a       b             c'\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     line_continuation_no_space,
  justfile: r#"
foo:
  echo a\
  b\
  c
"#,
  args:     (),
  stdout:   "abc\n",
  stderr:   "echo abc\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet_recipe,
  justfile: r#"
@quiet:
  # a
  # b
  @echo c
"#,
  args:     (),
  stdout:   "c\n",
  stderr:   "echo c\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     quiet_shebang_recipe,
  justfile: r#"
@quiet:
  #!/bin/sh
  echo hello
"#,
  args:     (),
  stdout:   "hello\n",
  stderr:   "#!/bin/sh\necho hello\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     shebang_line_numbers,
  justfile: r#"
quiet:
  #!/usr/bin/env cat

  a

  b


  c


"#,
  args:     (),
  stdout:   "#!/usr/bin/env cat



a

b


c


",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     complex_dependencies,
  justfile: r#"
a: b
b:
c: b a
"#,
  args:     ("b"),
  stdout:   "",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     parameter_shadows_variable,
  justfile: "FOO = 'hello'\na FOO:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Parameter `FOO` shadows variable of the same name
  |
2 | a FOO:
  |   ^^^
",
  status:   EXIT_FAILURE,
}


integration_test! {
  name:     dependency_takes_arguments,
  justfile: "b: a\na FOO:",
  args:     ("b"),
  stdout:   "",
  stderr:   "error: Recipe `b` depends on `a` which requires arguments. \
             Dependencies may not require arguments
  |
1 | b: a
  |    ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
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

integration_test! {
  name:     duplicate_dependency,
  justfile: "b:\na: b b",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Recipe `a` has duplicate dependency `b`
  |
2 | a: b b
  |      ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
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

integration_test! {
  name:     duplicate_variable,
  justfile: "a = 'hello'\na = 'hello'\nfoo:",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Variable `a` has multiple definitions
  |
2 | a = 'hello'
  | ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unexpected_token,
  justfile: "foo: 'bar'",
  args:     ("foo"),
  stdout:   "",
  stderr:   "error: Expected name, end of line, or end of file, but found raw string
  |
1 | foo: 'bar'
  |      ^^^^^
",
  status:   EXIT_FAILURE,
}


integration_test! {
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

integration_test! {
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

integration_test! {
  name:     variable_self_dependency,
  justfile: "z = z\na:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `z` is defined in terms of itself
  |
1 | z = z
  | ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     variable_circular_dependency,
  justfile: "x = y\ny = z\nz = x\na:",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Variable `x` depends on its own value: `x -> y -> z -> x`
  |
1 | x = y
  | ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     invalid_escape_sequence,
  justfile: r#"x = "\q"
a:"#,
  args:     ("a"),
  stdout:   "",
  stderr:   "error: `\\q` is not a valid escape sequence
  |
1 | x = \"\\q\"
  |     ^^^^
",
  status:   EXIT_FAILURE,
}


integration_test! {
  name:     multiline_raw_string,
  justfile: "
string = 'hello
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
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     error_line_after_multiline_raw_string,
  justfile: "
string = 'hello

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

integration_test! {
  name:     error_column_after_multiline_raw_string,
  justfile: "
string = 'hello

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

integration_test! {
  name:     multiline_raw_string_in_interpolation,
  justfile: r#"
a:
  echo '{{"a" + '
  ' + "b"}}'
"#,
  args:     ("a"),
  stdout:   "a
  b
",
  stderr:   "echo 'a
  b'
",
  status:   EXIT_SUCCESS,
}

integration_test! {
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

integration_test! {
  name:     unterminated_raw_string,
  justfile: "
a b=':
",
  args:     ("a"),
  stdout:   "",
  stderr:   "error: Unterminated string
  |
2 | a b=':
  |     ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     unterminated_string,
  justfile: r#"
a b=":
"#,
  args:     ("a"),
  stdout:   "",
  stderr:   r#"error: Unterminated string
  |
2 | a b=":
  |     ^
"#,
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     variadic_recipe,
  justfile: "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     variadic_ignore_default,
  justfile: "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1", "2", "3", " 4 "),
  stdout:   "0 1 2 3 4\n",
  stderr:   "echo 0 1 2 3  4 \n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     variadic_use_default,
  justfile: "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "0 1 HELLO\n",
  stderr:   "echo 0 1 HELLO\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     variadic_too_few,
  justfile: "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
  args:     ("a", "0", "1"),
  stdout:   "",
  stderr:   "error: Recipe `a` got 2 arguments but takes at least 3\n",
  status:   EXIT_FAILURE,
}

integration_test! {
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
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     missing_second_dependency,
  justfile: "
x:

a: x y
",
  args:     (),
  stdout:   "",
  stderr:   "error: Recipe `a` has unknown dependency `y`
  |
4 | a: x y
  |      ^
",
  status:   EXIT_FAILURE,
}

integration_test! {
  name:     list_colors,
  justfile: "
# comment
a B C +D='hello':
  echo {{B}} {{C}} {{D}}
",
  args:     ("--color", "always", "--list"),
  stdout:   "Available recipes:\n    a \
    \u{1b}[36mB\u{1b}[0m \u{1b}[36mC\u{1b}[0m \u{1b}[35m+\
    \u{1b}[0m\u{1b}[36mD\u{1b}[0m=\'\u{1b}[32mhello\u{1b}[0m\
    \' \u{1b}[34m#\u{1b}[0m \u{1b}[34mcomment\u{1b}[0m\n",
  stderr:   "",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     run_colors,
  justfile: "
# comment
a:
  echo hi
",
  args:     ("--color", "always", "--highlight", "--verbose"),
  stdout:   "hi\n",
  stderr:   "\u{1b}[1;36m===> Running recipe `a`...\u{1b}[0m\n\u{1b}[1mecho hi\u{1b}[0m\n",
  status:   EXIT_SUCCESS,
}

integration_test! {
  name:     trailing_flags,
  justfile: "
echo A B C:
  echo {{A}} {{B}} {{C}}
",
  args:     ("echo", "--some", "--awesome", "--flags"),
  stdout:   "--some --awesome --flags\n",
  stderr:   "echo --some --awesome --flags\n",
  status:   EXIT_SUCCESS,
}
