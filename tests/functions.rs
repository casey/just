use super::*;

test! {
  name:     test_os_arch_functions_in_interpolation,
  justfile: r#"
foo:
  echo {{arch()}} {{os()}} {{os_family()}} {{num_cpus()}}
"#,
  stdout:   format!("{} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
  stderr:   format!("echo {} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
}

test! {
  name:     test_os_arch_functions_in_expression,
  justfile: r#"
a := arch()
o := os()
f := os_family()
n := num_cpus()

foo:
  echo {{a}} {{o}} {{f}} {{n}}
"#,
  stdout:   format!("{} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
  stderr:   format!("echo {} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
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

#[cfg(not(windows))]
test! {
  name: path_functions,
  justfile: r#"
we  := without_extension('/foo/bar/baz.hello')
fs  := file_stem('/foo/bar/baz.hello')
fn  := file_name('/foo/bar/baz.hello')
dir := parent_directory('/foo/bar/baz.hello')
ext := extension('/foo/bar/baz.hello')
jn  := join('a', 'b')

foo:
  /bin/echo '{{we}}' '{{fs}}' '{{fn}}' '{{dir}}' '{{ext}}' '{{jn}}'
"#,
  stdout:   "/foo/bar/baz baz baz.hello /foo/bar hello a/b\n",
  stderr:   "/bin/echo '/foo/bar/baz' 'baz' 'baz.hello' '/foo/bar' 'hello' 'a/b'\n",
}

#[cfg(not(windows))]
test! {
  name: path_functions2,
  justfile: r#"
we  := without_extension('/foo/bar/baz')
fs  := file_stem('/foo/bar/baz.hello.ciao')
fn  := file_name('/bar/baz.hello.ciao')
dir := parent_directory('/foo/')
ext := extension('/foo/bar/baz.hello.ciao')

foo:
  /bin/echo '{{we}}' '{{fs}}' '{{fn}}' '{{dir}}' '{{ext}}'
"#,
  stdout:   "/foo/bar/baz baz.hello baz.hello.ciao / ciao\n",
  stderr:   "/bin/echo '/foo/bar/baz' 'baz.hello' 'baz.hello.ciao' '/' 'ciao'\n",
}

#[cfg(not(windows))]
test! {
  name: broken_without_extension_function,
  justfile: r#"
we  := without_extension('')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{} {}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `without_extension` failed:",
    "Could not extract parent from ``",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := without_extension(\'\')",
    "  │        ^^^^^^^^^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_extension_function,
  justfile: r#"
we  := extension('')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `extension` failed: Could not extract extension from ``",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := extension(\'\')",
    "  │        ^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_extension_function2,
  justfile: r#"
we  := extension('foo')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `extension` failed: Could not extract extension from `foo`",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := extension(\'foo\')",
    "  │        ^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_file_stem_function,
  justfile: r#"
we  := file_stem('')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `file_stem` failed: Could not extract file stem from ``",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := file_stem(\'\')",
    "  │        ^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_file_name_function,
  justfile: r#"
we  := file_name('')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `file_name` failed: Could not extract file name from ``",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := file_name(\'\')",
    "  │        ^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_directory_function,
  justfile: r#"
we  := parent_directory('')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{} {}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `parent_directory` failed:",
    "Could not extract parent directory from ``",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := parent_directory(\'\')",
    "  │        ^^^^^^^^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
}

#[cfg(not(windows))]
test! {
  name: broken_directory_function2,
  justfile: r#"
we  := parent_directory('/')

foo:
  /bin/echo '{{we}}'
"#,
  stdout:   "",
  stderr:   format!("{} {}\n{}\n{}\n{}\n{}\n",
    "error: Call to function `parent_directory` failed:",
    "Could not extract parent directory from `/`",
    " ——▶ justfile:1:8",
    "  │",
    "1 │ we  := parent_directory(\'/\')",
    "  │        ^^^^^^^^^^^^^^^^").as_str(),
  status:   EXIT_FAILURE,
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
 ——▶ justfile:2:10
  │
2 │   echo {{env_var('ZADDY')}}
  │          ^^^^^^^
",
  status:   EXIT_FAILURE,
}

test! {
  name:     test_just_executable_function,
  justfile: "
    a:
      @printf 'Executable path is: %s\\n' '{{ just_executable() }}'
  ",
  args:     ("a"),
  stdout:   format!("Executable path is: {}\n", executable_path("just").to_str().unwrap()).as_str(),
  stderr:   "",
  status:   EXIT_SUCCESS,
}

test! {
  name:     test_os_arch_functions_in_default,
  justfile: r#"
foo a=arch() o=os() f=os_family() n=num_cpus():
  echo {{a}} {{o}} {{f}} {{n}}
"#,
  stdout:   format!("{} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
  stderr:   format!("echo {} {} {} {}\n", target::arch(), target::os(), target::family(), num_cpus::get()).as_str(),
}

test! {
  name: clean,
  justfile: "
    foo:
      echo {{ clean('a/../b') }}
  ",
  stdout: "b\n",
  stderr: "echo b\n",
}

test! {
  name: uppercase,
  justfile: "
    foo:
      echo {{ uppercase('bar') }}
  ",
  stdout: "BAR\n",
  stderr: "echo BAR\n",
}

test! {
  name: lowercase,
  justfile: "
    foo:
      echo {{ lowercase('BAR') }}
  ",
  stdout: "bar\n",
  stderr: "echo bar\n",
}

test! {
  name: uppercamelcase,
  justfile: "
    foo:
      echo {{ uppercamelcase('foo bar') }}
  ",
  stdout: "FooBar\n",
  stderr: "echo FooBar\n",
}

test! {
  name: lowercamelcase,
  justfile: "
    foo:
      echo {{ lowercamelcase('foo bar') }}
  ",
  stdout: "fooBar\n",
  stderr: "echo fooBar\n",
}

test! {
  name: snakecase,
  justfile: "
    foo:
      echo {{ snakecase('foo bar') }}
  ",
  stdout: "foo_bar\n",
  stderr: "echo foo_bar\n",
}

test! {
  name: kebabcase,
  justfile: "
    foo:
      echo {{ kebabcase('foo bar') }}
  ",
  stdout: "foo-bar\n",
  stderr: "echo foo-bar\n",
}

test! {
  name: shoutysnakecase,
  justfile: "
    foo:
      echo {{ shoutysnakecase('foo bar') }}
  ",
  stdout: "FOO_BAR\n",
  stderr: "echo FOO_BAR\n",
}

test! {
  name: titlecase,
  justfile: "
    foo:
      echo {{ titlecase('foo bar') }}
  ",
  stdout: "Foo Bar\n",
  stderr: "echo Foo Bar\n",
}

test! {
  name: shoutykebabcase,
  justfile: "
    foo:
      echo {{ shoutykebabcase('foo bar') }}
  ",
  stdout: "FOO-BAR\n",
  stderr: "echo FOO-BAR\n",
}

test! {
  name: trim,
  justfile: "
    foo:
      echo {{ trim('   bar   ') }}
  ",
  stdout: "bar\n",
  stderr: "echo bar\n",
}

test! {
  name: replace,
  justfile: "
    foo:
      echo {{ replace('barbarbar', 'bar', 'foo') }}
  ",
  stdout: "foofoofoo\n",
  stderr: "echo foofoofoo\n",
}

test! {
  name: replace_regex,
  justfile: "
    foo:
      echo {{ replace_regex('123bar123bar123bar', '\\d+bar', 'foo') }}
  ",
  stdout: "foofoofoo\n",
  stderr: "echo foofoofoo\n",
}

test! {
  name: invalid_replace_regex,
  justfile: "
    foo:
      echo {{ replace_regex('barbarbar', 'foo\\', 'foo') }}
  ",
  stderr:
"error: Call to function `replace_regex` failed: regex parse error:
    foo\\
       ^
error: incomplete escape sequence, reached end of pattern prematurely
 ——▶ justfile:2:11
  │
2 │   echo {{ replace_regex('barbarbar', 'foo\\', 'foo') }}
  │           ^^^^^^^^^^^^^
",
  status: EXIT_FAILURE,
}

test! {
    name: capitalize,
    justfile: "
      foo:
        echo {{ capitalize('BAR') }}
    ",
    stdout: "Bar\n",
    stderr: "echo Bar\n",
}

#[test]
fn semver_matches() {
  Test::new()
    .justfile(
      "
      foo:
        echo {{ semver_matches('0.1.0', '>=0.1.0') }}
        echo {{ semver_matches('0.1.0', '=0.0.1') }}
    ",
    )
    .stdout("true\nfalse\n")
    .stderr("echo true\necho false\n")
    .run();
}

fn assert_eval_eq(expression: &str, result: &str) {
  Test::new()
    .justfile(format!("x := {expression}"))
    .args(["--evaluate", "x"])
    .stdout(result)
    .unindent_stdout(false)
    .run();
}

#[test]
fn trim_end_matches() {
  assert_eval_eq("trim_end_matches('foo', 'o')", "f");
  assert_eval_eq("trim_end_matches('fabab', 'ab')", "f");
  assert_eval_eq("trim_end_matches('fbaabab', 'ab')", "fba");
}

#[test]
fn trim_end_match() {
  assert_eval_eq("trim_end_match('foo', 'o')", "fo");
  assert_eval_eq("trim_end_match('fabab', 'ab')", "fab");
}

#[test]
fn trim_start_matches() {
  assert_eval_eq("trim_start_matches('oof', 'o')", "f");
  assert_eval_eq("trim_start_matches('ababf', 'ab')", "f");
  assert_eval_eq("trim_start_matches('ababbaf', 'ab')", "baf");
}

#[test]
fn trim_start_match() {
  assert_eval_eq("trim_start_match('oof', 'o')", "of");
  assert_eval_eq("trim_start_match('ababf', 'ab')", "abf");
}

#[test]
fn trim_start() {
  assert_eval_eq("trim_start('  f  ')", "f  ");
}

#[test]
fn trim_end() {
  assert_eval_eq("trim_end('  f  ')", "  f");
}

#[test]
#[cfg(not(windows))]
fn join() {
  assert_eval_eq("join('a', 'b', 'c', 'd')", "a/b/c/d");
  assert_eval_eq("join('a', '/b', 'c', 'd')", "/b/c/d");
  assert_eval_eq("join('a', '/b', '/c', 'd')", "/c/d");
  assert_eval_eq("join('a', '/b', '/c', '/d')", "/d");
}

#[test]
#[cfg(windows)]
fn join() {
  assert_eval_eq("join('a', 'b', 'c', 'd')", "a\\b\\c\\d");
  assert_eval_eq("join('a', '\\b', 'c', 'd')", "\\b\\c\\d");
  assert_eval_eq("join('a', '\\b', '\\c', 'd')", "\\c\\d");
  assert_eval_eq("join('a', '\\b', '\\c', '\\d')", "\\d");
}

#[test]
fn join_argument_count_error() {
  Test::new()
    .justfile("x := join('a')")
    .args(["--evaluate"])
    .stderr(
      "
      error: Function `join` called with 1 argument but takes 2 or more
       ——▶ justfile:1:6
        │
      1 │ x := join(\'a\')
        │      ^^^^
      ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn test_path_exists_filepath_exist() {
  Test::new()
    .tree(tree! {
      testfile: ""
    })
    .justfile("x := path_exists('testfile')")
    .args(["--evaluate", "x"])
    .stdout("true")
    .run();
}

#[test]
fn test_path_exists_filepath_doesnt_exist() {
  Test::new()
    .justfile("x := path_exists('testfile')")
    .args(["--evaluate", "x"])
    .stdout("false")
    .run();
}

#[test]
fn error_errors_with_message() {
  Test::new()
    .justfile("x := error ('Thing Not Supported')")
    .args(["--evaluate"])
    .status(1)
    .stderr(
      "
      error: Call to function `error` failed: Thing Not Supported
       ——▶ justfile:1:6
        │
      1 │ x := error ('Thing Not Supported')
        │      ^^^^^
    ",
    )
    .run();
}

#[test]
fn test_absolute_path_resolves() {
  let test_object = Test::new()
    .justfile("path := absolute_path('./test_file')")
    .args(["--evaluate", "path"]);

  let mut tempdir = test_object.tempdir.path().to_owned();

  // Just retrieves the current directory via env::current_dir(), which
  // does the moral equivalent of canonicalize, which will remove symlinks.
  // So, we have to canonicalize here, so that we can match it.
  if cfg!(unix) {
    tempdir = tempdir.canonicalize().unwrap();
  }

  test_object
    .stdout(tempdir.join("test_file").to_str().unwrap().to_owned())
    .run();
}

#[test]
fn test_absolute_path_resolves_parent() {
  let test_object = Test::new()
    .justfile("path := absolute_path('../test_file')")
    .args(["--evaluate", "path"]);

  let mut tempdir = test_object.tempdir.path().to_owned();

  // Just retrieves the current directory via env::current_dir(), which
  // does the moral equivalent of canonicalize, which will remove symlinks.
  // So, we have to canonicalize here, so that we can match it.
  if cfg!(unix) {
    tempdir = tempdir.canonicalize().unwrap();
  }

  test_object
    .stdout(
      tempdir
        .parent()
        .unwrap()
        .join("test_file")
        .to_str()
        .unwrap()
        .to_owned(),
    )
    .run();
}

#[test]
fn path_exists_subdir() {
  Test::new()
    .tree(tree! {
      foo: "",
      bar: {
      }
    })
    .justfile("x := path_exists('foo')")
    .current_dir("bar")
    .args(["--evaluate", "x"])
    .stdout("true")
    .run();
}

#[test]
fn uuid() {
  Test::new()
    .justfile("x := uuid()")
    .args(["--evaluate", "x"])
    .stdout_regex("........-....-....-....-............")
    .run();
}

#[test]
fn sha256() {
  Test::new()
    .justfile("x := sha256('5943ee37-0000-1000-8000-010203040506')")
    .args(["--evaluate", "x"])
    .stdout("2330d7f5eb94a820b54fed59a8eced236f80b633a504289c030b6a65aef58871")
    .run();
}

#[test]
fn sha256_file() {
  Test::new()
    .justfile("x := sha256_file('sub/shafile')")
    .tree(tree! {
      sub: {
        shafile: "just is great\n",
      }
    })
    .current_dir("sub")
    .args(["--evaluate", "x"])
    .stdout("177b3d79aaafb53a7a4d7aaba99a82f27c73370e8cb0295571aade1e4fea1cd2")
    .run();
}

#[test]
fn just_pid() {
  let mut test = Test::new()
    .args(["--evaluate", "x"])
    .justfile("x := just_pid()");
  let child = test.spawn();
  test.stdout(child.id().to_string()).check_output(child);
}
