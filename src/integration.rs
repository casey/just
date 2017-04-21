extern crate tempdir;
extern crate brev;
extern crate regex;

use tempdir::TempDir;
use std::{env, fs, path, process, str};

fn integration_test(
  args:            &[&str],
  justfile:        &str,
  expected_status: i32,
  expected_stdout: &str,
  expected_stderr: &str,
) {
  let tmp = TempDir::new("just-integration")
    .unwrap_or_else(
      |err| panic!("integration test: failed to create temporary directory: {}", err));
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(path, justfile);
  let mut binary = env::current_dir().unwrap();
  binary.push("./target/debug/just");
  let output = process::Command::new(binary)
    .current_dir(tmp.path())
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

fn search_test<P: AsRef<path::Path>>(path: P, args: &[&str]) {
  let mut binary = env::current_dir().unwrap();
  binary.push("./target/debug/just");
  let output = process::Command::new(binary)
    .current_dir(path)
    .args(args)
    .output()
    .expect("just invocation failed");

  assert_eq!(output.status.code().unwrap(), 0);

  let stdout = str::from_utf8(&output.stdout).unwrap();
  assert_eq!(stdout, "ok\n");

  let stderr = str::from_utf8(&output.stderr).unwrap();
  assert_eq!(stderr, "echo ok\n");
}

#[test]
fn test_justfile_search() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_capitalized_justfile_search() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("Justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_capitalization_priority() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();
  path.push("Justfile");
  brev::dump(&path, "default:\n\techo fail");
  path.pop();

  // if we see "default\n\techo fail" in `justfile` then we're running
  // in a case insensitive filesystem, so just bail
  path.push("justfile");
  if brev::slurp(&path) == "default:\n\techo fail" {
    return;
  }
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("b");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("c");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");
  path.push("d");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  search_test(path, &[]);
}

#[test]
fn test_upwards_path_argument() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  path.push("justfile");
  brev::dump(&path, "default:\n\techo bad");
  path.pop();

  search_test(&path, &["../"]);
  search_test(&path, &["../default"]);
}

#[test]
fn test_downwards_path_argument() {
  let tmp = TempDir::new("just-test-justfile-search")
    .expect("test justfile search: failed to create temporary directory");
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(&path, "default:\n\techo bad");
  path.pop();

  path.push("a");
  fs::create_dir(&path).expect("test justfile search: failed to create intermediary directory");

  path.push("justfile");
  brev::dump(&path, "default:\n\techo ok");
  path.pop();
  path.pop();

  search_test(&path, &["a/"]);
  search_test(&path, &["a/default"]);
  search_test(&path, &["./a/"]);
  search_test(&path, &["./a/default"]);
  search_test(&path, &["./a/"]);
  search_test(&path, &["./a/default"]);
}

#[test]
fn default() {
  integration_test(
    &[],
    "default:\n echo hello\nother: \n echo bar",
    0,
    "hello\n",
    "echo hello\n",
  )
}

#[test]
fn quiet() {
  integration_test(
    &[],
    "default:\n @echo hello",
    0,
    "hello\n",
    "",
  )
}

#[test]
fn verbose() {
  integration_test(
    &["--verbose"],
    "default:\n @echo hello",
    0,
    "hello\n",
    "===> Running recipe `default`...\necho hello\n",
  )
}


#[test]
fn order() {
  let text = "
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
  @mv b c";
  integration_test(
    &["a", "d"],
    text,
    0,
    "a\nb\nc\nd\n",
    "echo a\necho b\necho c\necho d\n",
  );
}

#[test]
fn summary() {
  let text =
"b: a
a:
d: c
c: b";
  integration_test(
    &["--summary"],
    text,
    0,
    "a b c d\n",
    "",
  );
}

#[test]
fn select() {
  let text =
"b:
  @echo b
a:
  @echo a
d:
  @echo d
c:
  @echo c";
  integration_test(
    &["d", "c"],
    text,
    0,
    "d\nc\n",
    "",
  );
}

#[test]
fn print() {
  let text =
"b:
  echo b
a:
  echo a
d:
  echo d
c:
  echo c";
  integration_test(
    &["d", "c"],
    text,
    0,
    "d\nc\n",
    "echo d\necho c\n",
  );
}


#[test]
fn show() {
  let text =
r#"hello = "foo"
bar = hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#;
  integration_test(
    &["--show", "recipe"],
    text,
    0,
    r#"recipe:
    echo {{hello + "bar" + bar}}
"#,
    "",
  );
}

#[test]
fn status_passthrough() {
  let text =
"

hello:

recipe:
  @exit 100";
  integration_test(
    &["recipe"],
    text,
    100,
    "",
    "error: Recipe `recipe` failed on line 6 with exit code 100\n",
  );
}

#[test]
fn unknown_dependency() {
  integration_test(
    &[],
    "bar:\nhello:\nfoo: bar baaaaaaaz hello",
    255,
    "",
    "error: Recipe `foo` has unknown dependency `baaaaaaaz`
  |
3 | foo: bar baaaaaaaz hello
  |          ^^^^^^^^^
",
  );
}

#[test]
fn backtick_success() {
  integration_test(
    &[],
    "a = `printf Hello,`\nbar:\n printf '{{a + `printf ' world!'`}}'",
    0,
    "Hello, world!",
    "printf 'Hello, world!'\n",
  );
}

#[test]
fn backtick_trimming() {
  integration_test(
    &[],
    "a = `echo Hello,`\nbar:\n echo '{{a + `echo ' world!'`}}'",
    0,
    "Hello, world!\n",
    "echo 'Hello, world!'\n",
  );
}

#[test]
fn backtick_code_assignment() {
  integration_test(
    &[],
    "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
    100,
    "",
    "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation() {
  integration_test(
    &[],
    "b = a\na = `echo hello`\nbar:\n echo '{{`exit 200`}}'",
    200,
    "",
    "error: Backtick failed with exit code 200
  |
4 |  echo '{{`exit 200`}}'
  |          ^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation_tab() {
  integration_test(
    &[],
    "
backtick-fail:
\techo {{`exit 1`}}
",
    1,
    "",
    "error: Backtick failed with exit code 1
  |
3 |     echo {{`exit 1`}}
  |            ^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation_tabs() {
  integration_test(
    &[],
    "
backtick-fail:
\techo {{\t`exit 1`}}
",
    1,
    "",
    "error: Backtick failed with exit code 1
  |
3 |     echo {{    `exit 1`}}
  |                ^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation_inner_tab() {
  integration_test(
    &[],
    "
backtick-fail:
\techo {{\t`exit\t\t1`}}
",
    1,
    "",
    "error: Backtick failed with exit code 1
  |
3 |     echo {{    `exit        1`}}
  |                ^^^^^^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation_leading_emoji() {
  integration_test(
    &[],
    "
backtick-fail:
\techo 😬{{`exit 1`}}
",
    1,
    "",
    "error: Backtick failed with exit code 1
  |
3 |     echo 😬{{`exit 1`}}
  |              ^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation_unicode_hell() {
  integration_test(
    &[],
    "
backtick-fail:
\techo \t\t\t😬鎌鼬{{\t\t`exit 1 # \t\t\t😬鎌鼬`}}\t\t\t😬鎌鼬
",
    1,
    "",
    "error: Backtick failed with exit code 1
  |
3 |     echo             😬鎌鼬{{        `exit 1 #             😬鎌鼬`}}            😬鎌鼬
  |                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_long() {
  integration_test(
    &[],
    "\n\n\n\n\n\nb = a\na = `echo hello`\nbar:\n echo '{{`exit 200`}}'",
    200,
    "",
    "error: Backtick failed with exit code 200
   |
10 |  echo '{{`exit 200`}}'
   |          ^^^^^^^^^^
",
  );
}

#[test]
fn shebang_backtick_failure() {
  integration_test(
    &[],
    "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
    123,
    "",
    "error: Backtick failed with exit code 123
  |
4 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn command_backtick_failure() {
  integration_test(
    &[],
    "foo:
 echo hello
 echo {{`exit 123`}}",
    123,
    "hello\n",
    "echo hello\nerror: Backtick failed with exit code 123
  |
3 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn assignment_backtick_failure() {
  integration_test(
    &[],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    222,
    "",
    "error: Backtick failed with exit code 222
  |
4 | a = `exit 222`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn unknown_override_options() {
  integration_test(
    &["--set", "foo", "bar", "a", "b", "--set", "baz", "bob", "--set", "a", "b"],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    255,
    "",
    "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  );
}

#[test]
fn unknown_override_args() {
  integration_test(
    &["foo=bar", "baz=bob", "a=b", "a", "b"],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    255,
    "",
    "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
  );
}

#[test]
fn unknown_override_arg() {
  integration_test(
    &["foo=bar", "a=b", "a", "b"],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    255,
    "",
    "error: Variable `foo` overridden on the command line but not present in justfile\n",
  );
}

#[test]
fn overrides_first() {
  integration_test(
    &["foo=bar", "a=b", "recipe", "baz=bar"],
    r#"
foo = "foo"
a = "a"
baz = "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    0,
    "arg=baz=bar\nbarbbaz\n",
    "echo arg=baz=bar\necho barbbaz\n",
  );
}

#[test]
fn overrides_not_evaluated() {
  integration_test(
    &["foo=bar", "a=b", "recipe", "baz=bar"],
    r#"
foo = `exit 1`
a = "a"
baz = "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    0,
    "arg=baz=bar\nbarbbaz\n",
    "echo arg=baz=bar\necho barbbaz\n",
  );
}

#[test]
fn dry_run() {
  integration_test(
    &["--dry-run", "shebang", "command"],
    r#"
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
    0,
    "",
    "stderr
#!/bin/sh
touch /this/is/not/a/file
backtick
echo shebang interpolation
touch /this/is/not/a/file
backtick
echo command interpolation
",
  );
}

#[test]
fn evaluate() {
  integration_test(
    &["--evaluate"],
    r#"
foo = "a\t"
hello = "c"
bar = "b\t"
ab = foo + bar + hello

wut:
  touch /this/is/not/a/file
"#,
    0,
    r#"ab    = "a	b	c"
bar   = "b	"
foo   = "a	"
hello = "c"
"#,
    "",
  );
}

#[test]
fn export_success() {
  integration_test(
    &[],
    r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  echo $foo $bar $abc
"#,
    0,
    "a b abc\n",
    "echo $foo $bar $abc\n",
  );
}

#[test]
fn export_override() {
  integration_test(
    &["foo=hello", "--set", "bar", "bye"],
    r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + "-" + bar + "-" + baz

wut:
  echo $foo $bar $abc
"#,
    0,
    "hello bye hello-bye-c\n",
    "echo $foo $bar $abc\n",
  );
}

#[test]
fn outer_shebang() {
  integration_test(
    &[],
    r#"#!/lol/wut
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  #!/bin/sh
  echo $foo $bar $abc
"#,
    255,
    "",
    "error: `#!` is reserved syntax outside of recipes
  |
1 | #!/lol/wut
  | ^
",
  );
}


#[test]
fn export_shebang() {
  integration_test(
    &[],
    r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  #!/bin/sh
  echo $foo $bar $abc
"#,
    0,
    "a b abc\n",
    "",
  );
}

#[test]
fn export_recipe_backtick() {
  integration_test(
    &[],
    r#"
export exported_variable = "A-IS-A"

recipe:
  echo {{`echo recipe $exported_variable`}}
"#,
    0,
    "recipe A-IS-A\n",
    "echo recipe A-IS-A\n",
  );
}

#[test]
fn raw_string() {
  integration_test(
    &[],
    r#"
export exported_variable = '\\\\\\"'

recipe:
  echo {{`echo recipe $exported_variable`}}
"#,
    0,
    "recipe \\\"\n",
    "echo recipe \\\\\\\"\n",
  );
}

#[test]
fn line_error_spacing() {
  integration_test(
    &[],
    r#"








???
"#,
    255,
    "",
    "error: Unknown start of token:
   |
10 | ???
   | ^
",
  );
}

#[test]
fn quiet_flag_no_stdout() {
  integration_test(
    &["--quiet"],
    r#"
default:
  @echo hello
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_stderr() {
  integration_test(
    &["--quiet"],
    r#"
default:
  @echo hello 1>&2
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_command_echoing() {
  integration_test(
    &["--quiet"],
    r#"
default:
  exit
"#,
    0,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_error_messages() {
  integration_test(
    &["--quiet"],
    r#"
default:
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_assignment_backtick_stderr() {
  integration_test(
    &["--quiet"],
    r#"
a = `echo hello 1>&2`
default:
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_no_interpolation_backtick_stderr() {
  integration_test(
    &["--quiet"],
    r#"
default:
  echo `echo hello 1>&2`
  exit 100
"#,
    100,
    "",
    "",
  );
}

#[test]
fn quiet_flag_or_dry_run_flag() {
  integration_test(
    &["--quiet", "--dry-run"],
    "",
    1,
    "",
    "error: The argument '--dry-run' cannot be used with '--quiet'

USAGE:
    just --color <color> --quiet

For more information try --help\n",
  );
}

#[test]
fn argument_single() {
  integration_test(
    &["foo", "ARGUMENT"],
    "
foo A:
  echo {{A}}
    ",
    0,
    "ARGUMENT\n",
    "echo ARGUMENT\n",
  );
}

#[test]
fn argument_multiple() {
  integration_test(
    &["foo", "ONE", "TWO"],
    "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    0,
    "A:ONE B:TWO\n",
    "echo A:ONE B:TWO\n",
  );
}

#[test]
fn argument_mismatch_more() {
  integration_test(
    &["foo", "ONE", "TWO", "THREE"],
    "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    255,
    "",
    "error: Justfile does not contain recipe `THREE`.\n",
  );
}

#[test]
fn argument_mismatch_fewer() {
  integration_test(
    &["foo", "ONE"],
    "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    255,
    "",
    "error: Recipe `foo` got 1 argument but takes 2\n"
  );
}

#[test]
fn argument_mismatch_more_with_default() {
  integration_test(
    &["foo", "ONE", "TWO", "THREE"],
    "
foo A B='B':
  echo A:{{A}} B:{{B}}
    ",
    255,
    "",
    "error: Justfile does not contain recipe `THREE`.\n",
  );
}

#[test]
fn argument_mismatch_fewer_with_default() {
  integration_test(
    &["foo", "bar"],
    "
foo A B C='C':
  echo A:{{A}} B:{{B}} C:{{C}}
    ",
    255,
    "",
    "error: Recipe `foo` got 1 argument but takes at least 2\n",
  );
}

#[test]
fn unknown_recipe() {
  integration_test(
    &["foo"],
    "hello:",
    255,
    "",
    "error: Justfile does not contain recipe `foo`.\n",
  );
}

#[test]
fn unknown_recipes() {
  integration_test(
    &["foo", "bar"],
    "hello:",
    255,
    "",
    "error: Justfile does not contain recipes `foo` or `bar`.\n",
  );
}

#[test]
fn color_always() {
  integration_test(
    &["--color", "always"],
    "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
    100,
    "",
    "\u{1b}[1;31merror:\u{1b}[0m \u{1b}[1mBacktick failed with exit code 100
\u{1b}[0m  |\n2 | a = `exit 100`\n  |     \u{1b}[1;31m^^^^^^^^^^\u{1b}[0m\n",
  );
}

#[test]
fn color_never() {
  integration_test(
    &["--color", "never"],
    "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
    100,
    "",
    "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn color_auto() {
  integration_test(
    &["--color", "auto"],
    "b = a\na = `exit 100`\nbar:\n echo '{{`exit 200`}}'",
    100,
    "",
    "error: Backtick failed with exit code 100
  |
2 | a = `exit 100`
  |     ^^^^^^^^^^
",
  );
}

#[test]
fn colors_no_context() {
  let text ="
recipe:
  @exit 100";
  integration_test(
    &["--color=always"],
    text,
    100,
    "",
    "\u{1b}[1;31merror:\u{1b}[0m \u{1b}[1m\
Recipe `recipe` failed on line 3 with exit code 100\u{1b}[0m\n",
  );
}

#[test]
fn dump() {
  let text = r#"
# this recipe does something
recipe a b +d:
 @exit 100"#;
  integration_test(
    &["--dump"],
    text,
    0,
    "# this recipe does something
recipe a b +d:
    @exit 100
",
    "",
  );
}

#[test]
fn mixed_whitespace() {
  integration_test(
    &[],
    "bar:\n\t echo hello",
    255,
    "",
    "error: Found a mix of tabs and spaces in leading whitespace: `␉␠`
Leading whitespace may consist of tabs or spaces, but not both
  |
2 |      echo hello
  | ^
",
  );
}

#[test]
fn extra_leading_whitespace() {
  integration_test(
    &[],
    "bar:\n\t\techo hello\n\t\t\techo goodbye",
    255,
    "",
    "error: Recipe line has extra leading whitespace
  |
3 |             echo goodbye
  |         ^^^^^^^^^^^^^^^^
",
  );
}

#[test]
fn inconsistent_leading_whitespace() {
  integration_test(
    &[],
    "bar:\n\t\techo hello\n\t echo goodbye",
    255,
    "",
    "error: Recipe line has inconsistent leading whitespace. Recipe started with `␉␉` but found line with `␉␠`
  |
3 |      echo goodbye
  | ^
",
  );
}


#[test]
fn required_after_default() {
  integration_test(
    &[],
    "bar:\nhello baz arg='foo' bar:",
    255,
    "",
    "error: Non-default parameter `bar` follows default parameter
  |
2 | hello baz arg='foo' bar:
  |                     ^^^
",
  );
}

#[test]
fn required_after_variadic() {
  integration_test(
    &[],
    "bar:\nhello baz +arg bar:",
    255,
    "",
    "error: Parameter `bar` follows variadic parameter
  |
2 | hello baz +arg bar:
  |                ^^^
",
  );
}

#[test]
fn use_string_default() {
  integration_test(
    &["hello", "ABC"],
    r#"
bar:
hello baz arg="XYZ\t\"	":
  echo '{{baz}}...{{arg}}'
"#,
    0,
    "ABC...XYZ\t\"\t\n",
    "echo 'ABC...XYZ\t\"\t'\n",
  );
}


#[test]
fn use_raw_string_default() {
  integration_test(
    &["hello", "ABC"],
    r#"
bar:
hello baz arg='XYZ\t\"	':
  echo '{{baz}}...{{arg}}'
"#,
    0,
    "ABC...XYZ\t\\\"\t\n",
    "echo 'ABC...XYZ\\t\\\"\t'\n",
  );
}

#[test]
fn supply_use_default() {
  integration_test(
    &["hello", "0", "1"],
    r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
    0,
    "0 1 C\n",
    "echo 0 1 C\n",
  );
}

#[test]
fn supply_defaults() {
  integration_test(
    &["hello", "0", "1", "2"],
    r#"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
"#,
    0,
    "0 1 2\n",
    "echo 0 1 2\n",
  );
}

#[test]
fn list() {
  integration_test(
    &["--list"],
    r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this comment will be ignored

a Z="\t z":
"#,
    0,
    r"Available recipes:
    a Z='\t z'
    hello a b='B\t' c='C' # this does a thing
",
    "",
  );
}

#[test]
fn show_suggestion() {
  integration_test(
    &["--show", "hell"],
    r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
    255,
    "",
    "Justfile does not contain recipe `hell`.\nDid you mean `hello`?\n",
  );
}

#[test]
fn show_no_suggestion() {
  integration_test(
    &["--show", "hell"],
    r#"
helloooooo a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
    255,
    "",
    "Justfile does not contain recipe `hell`.\n",
  );
}

#[test]
fn run_suggestion() {
  integration_test(
    &["hell"],
    r#"
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

a Z="\t z":
"#,
    255,
    "",
    "error: Justfile does not contain recipe `hell`.\nDid you mean `hello`?\n",
  );
}

#[test]
fn line_continuation_with_space() {
  integration_test(
    &[],
    r#"
foo:
  echo a\
         b  \
             c
"#,
    0,
    "a b c\n",
    "echo a       b             c\n",
  );
}

#[test]
fn line_continuation_with_quoted_space() {
  integration_test(
    &[],
    r#"
foo:
  echo 'a\
         b  \
             c'
"#,
    0,
    "a       b             c\n",
    "echo 'a       b             c'\n",
  );
}

#[test]
fn line_continuation_no_space() {
  integration_test(
    &[],
    r#"
foo:
  echo a\
  b\
  c
"#,
    0,
    "abc\n",
    "echo abc\n",
  );
}

#[test]
fn quiet_recipe() {
  integration_test(
    &[],
    r#"
@quiet:
  # a
  # b
  @echo c
"#,
    0,
    "c\n",
    "echo c\n",
  );
}

#[test]
fn quiet_shebang_recipe() {
  integration_test(
    &[],
    r#"
@quiet:
  #!/bin/sh
  echo hello
"#,
    0,
    "hello\n",
    "#!/bin/sh\necho hello\n",
  );
}

#[test]
fn shebang_line_numbers() {
  integration_test(
    &[],
    r#"
quiet:
  #!/usr/bin/env cat

  a

  b


  c


"#,
    0,
    "#!/usr/bin/env cat



a

b


c


",
    "",
  );
}

#[test]
fn complex_dependencies() {
  integration_test(
    &["b"],
    r#"
a: b
b:
c: b a
"#,
    0,
    "",
    ""
  );
}

#[test]
fn parameter_shadows_variable() {
  integration_test(
    &["a"],
    "FOO = 'hello'\na FOO:",
    255,
    "",
    "error: Parameter `FOO` shadows variable of the same name
  |
2 | a FOO:
  |   ^^^
",
  );
}


#[test]
fn dependency_takes_arguments() {
  integration_test(
    &["b"],
    "b: a\na FOO:",
    255,
    "",
    "error: Recipe `b` depends on `a` which requires arguments. Dependencies may not require arguments
  |
1 | b: a
  |    ^
",
  );
}

#[test]
fn duplicate_parameter() {
  integration_test(
    &["a"],
    "a foo foo:",
    255,
    "",
    "error: Recipe `a` has duplicate parameter `foo`
  |
1 | a foo foo:
  |       ^^^
",
  );
}

#[test]
fn duplicate_dependency() {
  integration_test(
    &["a"],
    "b:\na: b b",
    255,
    "",
    "error: Recipe `a` has duplicate dependency `b`
  |
2 | a: b b
  |      ^
",
  );
}

#[test]
fn duplicate_recipe() {
  integration_test(
    &["b"],
    "b:\nb:",
    255,
    "",
    "error: Recipe `b` first defined on line 1 is redefined on line 2
  |
2 | b:
  | ^
",
  );
}

#[test]
fn duplicate_variable() {
  integration_test(
    &["foo"],
    "a = 'hello'\na = 'hello'\nfoo:",
    255,
    "",
    "error: Variable `a` has multiple definitions
  |
2 | a = 'hello'
  | ^
",
  );
}

#[test]
fn unexpected_token() {
  integration_test(
    &["foo"],
    "foo: 'bar'",
    255,
    "",
    "error: Expected name, end of line, or end of file, but found raw string
  |
1 | foo: 'bar'
  |      ^^^^^
",
  );
}


#[test]
fn self_dependency() {
  integration_test(
    &["a"],
    "a: a",
    255,
    "",
    "error: Recipe `a` depends on itself
  |
1 | a: a
  |    ^
",
  );
}

#[test]
fn long_circular_recipe_dependency() {
  integration_test(
    &["a"],
    "a: b\nb: c\nc: d\nd: a",
    255,
    "",
    "error: Recipe `d` has circular dependency `a -> b -> c -> d -> a`
  |
4 | d: a
  |    ^
",
  );
}

#[test]
fn variable_self_dependency() {
  integration_test(
    &["a"],
    "z = z\na:",
    255,
    "",
    "error: Variable `z` is defined in terms of itself
  |
1 | z = z
  | ^
",
  );
}

#[test]
fn variable_circular_dependency() {
  integration_test(
    &["a"],
    "x = y\ny = z\nz = x\na:",
    255,
    "",
    "error: Variable `x` depends on its own value: `x -> y -> z -> x`
  |
1 | x = y
  | ^
",
  );
}

#[test]
fn invalid_escape_sequence() {
  integration_test(
    &["a"],
    r#"x = "\q"
a:"#,
    255,
    "",
    "error: `\\q` is not a valid escape sequence
  |
1 | x = \"\\q\"
  |     ^^^^
",
  );
}


#[test]
fn multiline_raw_string() {
  integration_test(
    &["a"],
    "
string = 'hello
whatever'

a:
  echo '{{string}}'
",
    0,
    "hello
whatever
",
    "echo 'hello
whatever'
",
  );
}

#[test]
fn error_line_after_multiline_raw_string() {
  integration_test(
    &["a"],
    "
string = 'hello

whatever' + 'yo'

a:
  echo '{{foo}}'
",
    255,
    "",
    "error: Variable `foo` not defined
  |
7 |   echo '{{foo}}'
  |           ^^^
",
  );
}

#[test]
fn error_column_after_multiline_raw_string() {
  integration_test(
    &["a"],
    "
string = 'hello

whatever' + bar

a:
  echo '{{string}}'
",
    255,
    "",
    "error: Variable `bar` not defined
  |
4 | whatever' + bar
  |             ^^^
",
  );
}

#[test]
fn multiline_raw_string_in_interpolation() {
  integration_test(
    &["a"],
    r#"
a:
  echo '{{"a" + '
  ' + "b"}}'
"#,
    0,
    "a
  b
",
    "echo 'a
  b'
",
  );
}

#[test]
fn error_line_after_multiline_raw_string_in_interpolation() {
  integration_test(
    &["a"],
    r#"
a:
  echo '{{"a" + '
  ' + "b"}}'

  echo {{b}}
"#,
    255,
    "",
    "error: Variable `b` not defined
  |
6 |   echo {{b}}
  |          ^
",
  );
}

#[test]
fn unterminated_raw_string() {
  integration_test(
    &["a"],
    "
a b=':
",
    255,
    "",
    "error: Unterminated string
  |
2 | a b=':
  |     ^
",
  );
}

#[test]
fn unterminated_string() {
  integration_test(
    &["a"],
    r#"
a b=":
"#,
    255,
    "",
    r#"error: Unterminated string
  |
2 | a b=":
  |     ^
"#,
  );
}

#[test]
fn variadic_recipe() {
  integration_test(
    &["a", "0", "1", "2", "3", " 4 "],
    "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
    0,
    "0 1 2 3 4\n",
    "echo 0 1 2 3  4 \n",
  );
}

#[test]
fn variadic_ignore_default() {
  integration_test(
    &["a", "0", "1", "2", "3", " 4 "],
    "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    0,
    "0 1 2 3 4\n",
    "echo 0 1 2 3  4 \n",
  );
}

#[test]
fn variadic_use_default() {
  integration_test(
    &["a", "0", "1"],
    "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    0,
    "0 1 HELLO\n",
    "echo 0 1 HELLO\n",
  );
}

#[test]
fn variadic_too_few() {
  integration_test(
    &["a", "0", "1"],
    "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
    255,
    "",
    "error: Recipe `a` got 2 arguments but takes at least 3\n",
  );
}

#[test]
fn argument_grouping() {
  integration_test(
    &["BAR", "0", "FOO", "1", "2", "BAZ", "3", "4", "5"],
    "
FOO A B='blarg':
  echo foo: {{A}} {{B}}

BAR X:
  echo bar: {{X}}

BAZ +Z:
  echo baz: {{Z}}
",
    0,
    "bar: 0\nfoo: 1 2\nbaz: 3 4 5\n",
    "echo bar: 0\necho foo: 1 2\necho baz: 3 4 5\n",
  );
}

#[test]
fn missing_second_dependency() {
  integration_test(
    &[],
    "
x:

a: x y
",
    255,
    "",
    "error: Recipe `a` has unknown dependency `y`
  |
4 | a: x y
  |      ^
",
  );
}
