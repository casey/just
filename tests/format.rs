use super::*;

#[test]
fn unstable_not_passed() {
  Test::new()
    .arg("--fmt")
    .justfile("")
    .stderr_regex("error: The `--fmt` command is currently unstable..*")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn check_without_fmt() {
  Test::new()
    .arg("--check")
    .justfile("")
    .stderr_regex(
      "error: the following required arguments were not provided:
  --fmt
(.|\\n)+",
    )
    .status(2)
    .run();
}

#[test]
fn check_ok() {
  Test::new()
    .arg("--unstable")
    .arg("--fmt")
    .arg("--check")
    .justfile(
      r#"
# comment   with   spaces

export x := `backtick
with
lines`

recipe: deps
    echo "$x"

deps:
    echo {{ x }}
    echo '$x'
"#,
    )
    .status(EXIT_SUCCESS)
    .run();
}

#[test]
fn check_found_diff() {
  Test::new()
    .arg("--unstable")
    .arg("--fmt")
    .arg("--check")
    .justfile("x:=``\n")
    .stdout(
      "
    -x:=``
    +x := ``
  ",
    )
    .stderr(
      "
    error: Formatted justfile differs from original.
  ",
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn check_found_diff_quiet() {
  Test::new()
    .arg("--unstable")
    .arg("--fmt")
    .arg("--check")
    .arg("--quiet")
    .justfile("x:=``\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn check_diff_color() {
  Test::new()
        .justfile("x:=``\n")
        .arg("--unstable")
        .arg("--fmt")
        .arg("--check")
        .arg("--color")
        .arg("always")
        .stdout("\n    \u{1b}[31m-x:=``\n    \u{1b}[0m\u{1b}[32m+x := ``\n    \u{1b}[0m")
        .stderr("\n    \u{1b}[1;31merror\u{1b}[0m: \u{1b}[1mFormatted justfile differs from original.\u{1b}[0m\n  ")
        .status(EXIT_FAILURE)
        .run();
}

#[test]
fn unstable_passed() {
  let tmp = tempdir();

  let justfile = tmp.path().join("justfile");

  fs::write(&justfile, "x    :=    'hello'   ").unwrap();

  let output = Command::new(executable_path("just"))
    .current_dir(tmp.path())
    .arg("--fmt")
    .arg("--unstable")
    .output()
    .unwrap();

  if !output.status.success() {
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    panic!("justfile failed with status: {}", output.status);
  }

  assert_eq!(fs::read_to_string(&justfile).unwrap(), "x := 'hello'\n");
}

#[test]
fn write_error() {
  // skip this test if running as root, since root can write files even if
  // permissions would otherwise forbid it
  #[cfg(not(windows))]
  if nix::unistd::getuid() == nix::unistd::ROOT {
    return;
  }

  let tempdir = temptree! {
    justfile: "x    :=    'hello'   ",
  };

  let test = Test::with_tempdir(tempdir)
    .no_justfile()
    .args(["--fmt", "--unstable"])
    .status(EXIT_FAILURE)
    .stderr_regex(if cfg!(windows) {
      r"error: Failed to write justfile to `.*`: Access is denied. \(os error 5\)\n"
    } else {
      r"error: Failed to write justfile to `.*`: Permission denied \(os error 13\)\n"
    });

  let justfile_path = test.justfile_path();

  let output = Command::new("chmod")
    .arg("400")
    .arg(&justfile_path)
    .output()
    .unwrap();

  assert!(output.status.success());

  let _tempdir = test.run();

  assert_eq!(
    fs::read_to_string(&justfile_path).unwrap(),
    "x    :=    'hello'   "
  );
}

#[test]
fn alias_good() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    alias f := foo

    foo:
        echo foo
  ",
    )
    .stdout(
      "
    alias f := foo

    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn alias_fix_indent() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    alias f:=    foo

    foo:
        echo foo
  ",
    )
    .stdout(
      "
    alias f := foo

    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn assignment_singlequote() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := 'foo'
  ",
    )
    .stdout(
      "
    foo := 'foo'
  ",
    )
    .run();
}

#[test]
fn assignment_doublequote() {
  Test::new()
    .arg("--dump")
    .justfile(
      r#"
    foo := "foo"
  "#,
    )
    .stdout(
      r#"
    foo := "foo"
  "#,
    )
    .run();
}

#[test]
fn assignment_indented_singlequote() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := '''
      foo
    '''
  ",
    )
    .stdout(
      r"
    foo := '''
      foo
    '''
  ",
    )
    .run();
}

#[test]
fn assignment_indented_doublequote() {
  Test::new()
    .arg("--dump")
    .justfile(
      r#"
    foo := """
      foo
    """
  "#,
    )
    .stdout(
      r#"
    foo := """
      foo
    """
  "#,
    )
    .run();
}

#[test]
fn assignment_backtick() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := `foo`
  ",
    )
    .stdout(
      "
    foo := `foo`
  ",
    )
    .run();
}

#[test]
fn assignment_indented_backtick() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := ```
      foo
    ```
  ",
    )
    .stdout(
      "
    foo := ```
      foo
    ```
  ",
    )
    .run();
}

#[test]
fn assignment_name() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar := 'bar'
    foo := bar
  ",
    )
    .stdout(
      "
    bar := 'bar'
    foo := bar
  ",
    )
    .run();
}

#[test]
fn assignment_parenthesized_expression() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := ('foo')
  ",
    )
    .stdout(
      "
    foo := ('foo')
  ",
    )
    .run();
}

#[test]
fn assignment_export() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    export foo := 'foo'
  ",
    )
    .stdout(
      "
    export foo := 'foo'
  ",
    )
    .run();
}

#[test]
fn assignment_concat_values() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := 'foo' + 'bar'
  ",
    )
    .stdout(
      "
    foo := 'foo' + 'bar'
  ",
    )
    .run();
}

#[test]
fn assignment_if_oneline() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
  ",
    )
    .stdout(
      "
    foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
  ",
    )
    .run();
}

#[test]
fn assignment_if_multiline() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := if 'foo' != 'foo' {
      'foo'
    } else {
      'bar'
    }
  ",
    )
    .stdout(
      "
    foo := if 'foo' != 'foo' { 'foo' } else { 'bar' }
  ",
    )
    .run();
}

#[test]
fn assignment_nullary_function() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := arch()
  ",
    )
    .stdout(
      "
    foo := arch()
  ",
    )
    .run();
}

#[test]
fn assignment_unary_function() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := env_var('foo')
  ",
    )
    .stdout(
      "
    foo := env_var('foo')
  ",
    )
    .run();
}

#[test]
fn assignment_binary_function() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := env_var_or_default('foo', 'bar')
  ",
    )
    .stdout(
      "
    foo := env_var_or_default('foo', 'bar')
  ",
    )
    .run();
}

#[test]
fn assignment_path_functions() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo  := without_extension('foo/bar.baz')
    foo2 := file_stem('foo/bar.baz')
    foo3 := parent_directory('foo/bar.baz')
    foo4 := file_name('foo/bar.baz')
    foo5 := extension('foo/bar.baz')
  ",
    )
    .stdout(
      "
  foo := without_extension('foo/bar.baz')
  foo2 := file_stem('foo/bar.baz')
  foo3 := parent_directory('foo/bar.baz')
  foo4 := file_name('foo/bar.baz')
  foo5 := extension('foo/bar.baz')
  ",
    )
    .run();
}

#[test]
fn recipe_ordinary() {
  Test::new()
    .justfile(
      "
    foo:
        echo bar
  ",
    )
    .arg("--dump")
    .stdout(
      "
    foo:
        echo bar
  ",
    )
    .run();
}

#[test]
fn recipe_with_docstring() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # bar
    foo:
        echo bar
  ",
    )
    .stdout(
      "
    # bar
    foo:
        echo bar
  ",
    )
    .run();
}

#[test]
fn recipe_with_comments_in_body() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        # bar
        echo bar
  ",
    )
    .stdout(
      "
    foo:
        # bar
        echo bar
  ",
    )
    .run();
}

#[test]
fn recipe_body_is_comment() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        # bar
  ",
    )
    .stdout(
      "
    foo:
        # bar
  ",
    )
    .run();
}

#[test]
fn recipe_several_commands() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        echo bar
        echo baz
  ",
    )
    .stdout(
      "
    foo:
        echo bar
        echo baz
  ",
    )
    .run();
}

#[test]
fn recipe_quiet() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    @foo:
        echo bar
  ",
    )
    .stdout(
      "
    @foo:
        echo bar
  ",
    )
    .run();
}

#[test]
fn recipe_quiet_command() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        @echo bar
  ",
    )
    .stdout(
      "
    foo:
        @echo bar
  ",
    )
    .run();
}

#[test]
fn recipe_quiet_comment() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        @# bar
  ",
    )
    .stdout(
      "
    foo:
        @# bar
  ",
    )
    .run();
}

#[test]
fn recipe_ignore_errors() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        -echo foo
  ",
    )
    .stdout(
      "
    foo:
        -echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR:
        echo foo
  ",
    )
    .stdout(
      "
    foo BAR:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_default() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR='bar':
        echo foo
  ",
    )
    .stdout(
      "
    foo BAR='bar':
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_envar() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo $BAR:
        echo foo
  ",
    )
    .stdout(
      "
    foo $BAR:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_default_envar() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo $BAR='foo':
        echo foo
  ",
    )
    .stdout(
      "
    foo $BAR='foo':
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_concat() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR=('bar' + 'baz'):
        echo foo
  ",
    )
    .stdout(
      "
    foo BAR=('bar' + 'baz'):
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameters() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR BAZ:
        echo foo
  ",
    )
    .stdout(
      "
    foo BAR BAZ:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameters_envar() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo $BAR $BAZ:
        echo foo
  ",
    )
    .stdout(
      "
    foo $BAR $BAZ:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_variadic_plus() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo +BAR:
        echo foo
  ",
    )
    .stdout(
      "
    foo +BAR:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_variadic_star() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo *BAR:
        echo foo
  ",
    )
    .stdout(
      "
    foo *BAR:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_positional_variadic() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR *BAZ:
        echo foo
  ",
    )
    .stdout(
      "
    foo BAR *BAZ:
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_variadic_default() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo +BAR='bar':
        echo foo
  ",
    )
    .stdout(
      "
    foo +BAR='bar':
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_in_body() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR:
        echo {{ BAR }}
  ",
    )
    .stdout(
      "
    foo BAR:
        echo {{ BAR }}
  ",
    )
    .run();
}

#[test]
fn recipe_parameter_conditional() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR:
        echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
  ",
    )
    .stdout(
      "
    foo BAR:
        echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
  ",
    )
    .run();
}

#[test]
fn recipe_escaped_braces() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo BAR:
        echo '{{{{BAR}}}}'
  ",
    )
    .stdout(
      "
    foo BAR:
        echo '{{{{BAR}}}}'
  ",
    )
    .run();
}

#[test]
fn recipe_assignment_in_body() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar := 'bar'

    foo:
        echo $bar
  ",
    )
    .stdout(
      "
    bar := 'bar'

    foo:
        echo $bar
  ",
    )
    .run();
}

#[test]
fn recipe_dependency() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar:
        echo bar

    foo: bar
        echo foo
  ",
    )
    .stdout(
      "
    bar:
        echo bar

    foo: bar
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_dependency_param() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar BAR:
        echo bar

    foo: (bar 'bar')
        echo foo
  ",
    )
    .stdout(
      "
    bar BAR:
        echo bar

    foo: (bar 'bar')
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_dependency_params() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar BAR BAZ:
        echo bar

    foo: (bar 'bar' 'baz')
        echo foo
  ",
    )
    .stdout(
      "
    bar BAR BAZ:
        echo bar

    foo: (bar 'bar' 'baz')
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_dependencies() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar:
        echo bar

    baz:
        echo baz

    foo: baz bar
        echo foo
  ",
    )
    .stdout(
      "
    bar:
        echo bar

    baz:
        echo baz

    foo: baz bar
        echo foo
  ",
    )
    .run();
}

#[test]
fn recipe_dependencies_params() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar BAR:
        echo bar

    baz BAZ:
        echo baz

    foo: (baz 'baz') (bar 'bar')
        echo foo
  ",
    )
    .stdout(
      "
    bar BAR:
        echo bar

    baz BAZ:
        echo baz

    foo: (baz 'baz') (bar 'bar')
        echo foo
  ",
    )
    .run();
}

#[test]
fn set_true_explicit() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    set export := true
  ",
    )
    .stdout(
      "
    set export := true
  ",
    )
    .run();
}

#[test]
fn set_true_implicit() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    set export
  ",
    )
    .stdout(
      "
    set export := true
  ",
    )
    .run();
}

#[test]
fn set_false() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    set export := false
  ",
    )
    .stdout(
      "
    set export := false
  ",
    )
    .run();
}

#[test]
fn set_shell() {
  Test::new()
    .arg("--dump")
    .justfile(
      r#"
    set shell := ['sh', "-c"]
  "#,
    )
    .stdout(
      r#"
    set shell := ['sh', "-c"]
  "#,
    )
    .run();
}

#[test]
fn comment() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # foo
  ",
    )
    .stdout(
      "
    # foo
  ",
    )
    .run();
}

#[test]
fn comment_multiline() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # foo
    # bar
  ",
    )
    .stdout(
      "
    # foo
    # bar
  ",
    )
    .run();
}

#[test]
fn comment_leading() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # foo

    foo := 'bar'
  ",
    )
    .stdout(
      "
    # foo

    foo := 'bar'
  ",
    )
    .run();
}

#[test]
fn comment_trailing() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := 'bar'

    # foo
  ",
    )
    .stdout(
      "
    foo := 'bar'

    # foo
  ",
    )
    .run();
}

#[test]
fn comment_before_recipe() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # foo

    foo:
        echo foo
  ",
    )
    .stdout(
      "
    # foo

    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn comment_before_docstring_recipe() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # bar

    # foo
    foo:
        echo foo
  ",
    )
    .stdout(
      "
    # bar

    # foo
    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn group_recipes() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        echo foo
    bar:
        echo bar
  ",
    )
    .stdout(
      "
    foo:
        echo foo

    bar:
        echo bar
  ",
    )
    .run();
}

#[test]
fn group_aliases() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    alias f := foo

    alias b := bar

    foo:
        echo foo

    bar:
        echo bar
  ",
    )
    .stdout(
      "
    alias f := foo
    alias b := bar

    foo:
        echo foo

    bar:
        echo bar
  ",
    )
    .run();
}

#[test]
fn group_assignments() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo := 'foo'
    bar := 'bar'
  ",
    )
    .stdout(
      "
    foo := 'foo'
    bar := 'bar'
  ",
    )
    .run();
}

#[test]
fn group_sets() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    set export := true
    set positional-arguments := true
  ",
    )
    .stdout(
      "
    set export := true
    set positional-arguments := true
  ",
    )
    .run();
}

#[test]
fn group_comments() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    # foo

    # bar
  ",
    )
    .stdout(
      "
    # foo
    # bar
  ",
    )
    .run();
}

#[test]
fn separate_recipes_aliases() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    alias f := foo
    foo:
        echo foo
  ",
    )
    .stdout(
      "
    alias f := foo

    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn no_trailing_newline() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    foo:
        echo foo",
    )
    .stdout(
      "
    foo:
        echo foo
  ",
    )
    .run();
}

#[test]
fn subsequent() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
    bar:
    foo: && bar
        echo foo",
    )
    .stdout(
      "
    bar:

    foo: && bar
        echo foo
  ",
    )
    .run();
}

#[test]
fn exported_parameter() {
  Test::new()
    .justfile("foo +$f:")
    .args(["--dump"])
    .stdout("foo +$f:\n")
    .run();
}

#[test]
fn multi_argument_attribute() {
  Test::new()
    .justfile(
      "
        set unstable

        [script('a', 'b', 'c')]
        foo:
      ",
    )
    .arg("--dump")
    .stdout(
      "
        set unstable := true

        [script('a', 'b', 'c')]
        foo:
      ",
    )
    .run();
}

#[test]
fn doc_attribute_suppresses_comment() {
  Test::new()
    .justfile(
      "
        set unstable

        # COMMENT
        [doc('ATTRIBUTE')]
        foo:
      ",
    )
    .arg("--dump")
    .stdout(
      "
        set unstable := true

        [doc('ATTRIBUTE')]
        foo:
      ",
    )
    .run();
}

#[test]
fn unchanged_justfiles_are_not_written_to_disk() {
  let tmp = tempdir();

  let justfile = tmp.path().join("justfile");

  fs::write(&justfile, "").unwrap();

  let mut permissions = fs::metadata(&justfile).unwrap().permissions();
  permissions.set_readonly(true);
  fs::set_permissions(&justfile, permissions).unwrap();

  Test::with_tempdir(tmp)
    .no_justfile()
    .args(["--fmt", "--unstable"])
    .run();
}

#[test]
fn if_else() {
  Test::new()
    .justfile(
      "
        x := if '' == '' { '' } else if '' == '' { '' } else { '' }
      ",
    )
    .arg("--dump")
    .stdout(
      "
        x := if '' == '' { '' } else if '' == '' { '' } else { '' }
      ",
    )
    .run();
}

#[test]
fn private_variable() {
  Test::new()
    .justfile(
      "
        [private]
        foo := 'bar'
      ",
    )
    .arg("--dump")
    .stdout(
      "
        [private]
        foo := 'bar'
      ",
    )
    .run();
}
