use crate::common::*;

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
  stderr:   format!("{} {}\n{}\n{}\n{}\n",
    "error: Call to function `without_extension` failed:",
    "Could not extract parent from ``",
    "  |",
    "1 | we  := without_extension(\'\')",
    "  |        ^^^^^^^^^^^^^^^^^").as_str(),
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
  stderr:   format!("{}\n{}\n{}\n{}\n",
    "error: Call to function `extension` failed: Could not extract extension from ``",
    "  |",
    "1 | we  := extension(\'\')",
    "  |        ^^^^^^^^^").as_str(),
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
  stderr:   format!("{}\n{}\n{}\n{}\n",
    "error: Call to function `extension` failed: Could not extract extension from `foo`",
    "  |",
    "1 | we  := extension(\'foo\')",
    "  |        ^^^^^^^^^").as_str(),
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
  stderr:   format!("{}\n{}\n{}\n{}\n",
    "error: Call to function `file_stem` failed: Could not extract file stem from ``",
    "  |",
    "1 | we  := file_stem(\'\')",
    "  |        ^^^^^^^^^").as_str(),
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
  stderr:   format!("{}\n{}\n{}\n{}\n",
    "error: Call to function `file_name` failed: Could not extract file name from ``",
    "  |",
    "1 | we  := file_name(\'\')",
    "  |        ^^^^^^^^^").as_str(),
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
  stderr:   format!("{} {}\n{}\n{}\n{}\n",
    "error: Call to function `parent_directory` failed:",
    "Could not extract parent directory from ``",
    "  |",
    "1 | we  := parent_directory(\'\')",
    "  |        ^^^^^^^^^^^^^^^^").as_str(),
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
  stderr:   format!("{} {}\n{}\n{}\n{}\n",
    "error: Call to function `parent_directory` failed:",
    "Could not extract parent directory from `/`",
    "  |",
    "1 | we  := parent_directory(\'/\')",
    "  |        ^^^^^^^^^^^^^^^^").as_str(),
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
  |
2 |   echo {{env_var('ZADDY')}}
  |          ^^^^^^^
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
foo a=arch() o=os() f=os_family():
  echo {{a}} {{o}} {{f}}
"#,
  stdout:   format!("{} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
  stderr:   format!("echo {} {} {}\n", target::arch(), target::os(), target::os_family()).as_str(),
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
