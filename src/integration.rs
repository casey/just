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
    println!("bad stdout:\ngot:\n{}\n\nexpected:\n{}", stderr, expected_stderr);
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
