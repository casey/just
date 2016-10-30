extern crate tempdir;
extern crate brev;

use tempdir::TempDir;
use super::std::process::Command;

fn integration_test(
  name:            &str,
  args:            &[&str],
  justfile:        &str,
  expected_status: i32,
  expected_stdout: &str,
  expected_stderr: &str,
) {
  let tmp = TempDir::new(name)
    .unwrap_or_else(|err| panic!("tmpdir: failed to create temporary directory: {}", err));
  let mut path = tmp.path().to_path_buf();
  path.push("justfile");
  brev::dump(path, justfile);
  let mut binary = super::std::env::current_dir().unwrap();
  binary.push("./target/debug/j");
  let output = Command::new(binary)
    .current_dir(tmp.path())
    .args(args)
    .output()
    .expect("j invocation failed");

  let mut failure = false;

  let status = output.status.code().unwrap();
  if status != expected_status {
    println!("bad status: {} != {}", status, expected_status);
    failure = true;
  }

  let stdout = super::std::str::from_utf8(&output.stdout).unwrap();
  if stdout != expected_stdout {
    println!("bad stdout:\ngot:\n{}\n\nexpected:\n{}", stdout, expected_stdout);
    failure = true;
  }

  let stderr = super::std::str::from_utf8(&output.stderr).unwrap();
  if stderr != expected_stderr {
    println!("bad stderr:\ngot:\n{}\n\nexpected:\n{}", stderr, expected_stderr);
    failure = true;
  }

  if failure {
    panic!("test failed");
  }
}

#[test]
fn default() {
  integration_test(
    "default",
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
    "quiet",
    &[],
    "default:\n @echo hello",
    0,
    "hello\n",
    "",
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
    "order",
    &["a", "d"],
    text,
    0,
    "a\nb\nc\nd\n",
    "echo a\necho b\necho c\necho d\n",
  );
}

#[test]
fn list() {
  let text = 
"b: a
a:
d: c
c: b";
  integration_test(
    "list",
    &["--list"],
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
    "select",
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
    "select",
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
    "show",
    &["--show", "recipe"],
    text,
    0,
    r#"recipe:
    echo {{hello + "bar" + bar}}
"#,
    "",
  );
}
/*
#[test]
fn debug() {
  let text = 
r#"hello = "foo"
bar = hello + hello
recipe:
 echo {{hello + "bar" + bar}}"#;
  integration_test(
    "debug",
    &["--debug"],
    text,
    0,
    r#"bar = hello + hello # "foofoo"

hello = "foo" # "foo"

recipe:
    echo {{hello + "bar" + bar # "foobarfoofoo"}}
"#,
    "",
  );
}
*/

#[test]
fn status() {
  let text = 
"
recipe:
 @function f { return 100; }; f";
  integration_test(
    "status",
    &[],
    text,
    100,
    "",
    "Recipe \"recipe\" failed with exit code 100\n",
  );
}

#[test]
fn error() {
  integration_test(
    "error",
    &[],
    "bar:\nhello:\nfoo: bar baaaaaaaz hello",
    255,
    "",
    "error: recipe `foo` has unknown dependency `baaaaaaaz`
  |
3 | foo: bar baaaaaaaz hello
  |          ^^^^^^^^^
",
  );
}

#[test]
fn backtick_success() {
  integration_test(
    "backtick_success",
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
    "backtick_trimming",
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
    "backtick_code_assignment",
    &[],
    "b = a\na = `function f { return 100; }; f`\nbar:\n echo '{{`function f { return 200; }; f`}}'",
    100,
    "",
    "backtick failed with exit code 100
  |
2 | a = `function f { return 100; }; f`
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
",
  );
}

#[test]
fn backtick_code_interpolation() {
  integration_test(
    "backtick_code_interpolation",
    &[],
    "b = a\na = `echo hello`\nbar:\n echo '{{`function f { return 200; }; f`}}'",
    200,
    "",
    "backtick failed with exit code 200
  |
4 |  echo '{{`function f { return 200; }; f`}}'
  |          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
",
  );
}

#[test]
fn shebang_backtick_failure() {
  integration_test(
    "shebang_backtick_failure",
    &[],
    "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
    123,
    "",
    "backtick failed with exit code 123
  |
4 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn command_backtick_failure() {
  integration_test(
    "command_backtick_failure",
    &[],
    "foo:
 echo hello
 echo {{`exit 123`}}",
    123,
    "hello\n",
    "echo hello\nbacktick failed with exit code 123
  |
3 |  echo {{`exit 123`}}
  |         ^^^^^^^^^^
",
  );
}

#[test]
fn assignment_backtick_failure() {
  integration_test(
    "assignment_backtick_failure",
    &[],
    "foo:
 echo hello
 echo {{`exit 111`}}
a = `exit 222`",
    222,
    "",
    "backtick failed with exit code 222
  |
4 | a = `exit 222`
  |     ^^^^^^^^^^
",
  );
}
