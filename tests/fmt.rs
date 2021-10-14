use crate::common::*;

test! {
  name: unstable_not_passed,
  justfile: "",
  args: ("--fmt"),
  stderr: "
    error: The `--fmt` command is currently unstable. \
    Invoke `just` with the `--unstable` flag to enable unstable features.
  ",
  status: EXIT_FAILURE,
}

test! {
  name: check_without_fmt,
  justfile: "",
  args: ("--check"),
  stderr_regex: "error: The following required arguments were not provided:
    --fmt
(.|\\n)+",
  status: EXIT_FAILURE,
}

test! {
  name: dry_run_ok,
  justfile: r#"
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
  args: ("--unstable", "--fmt", "--check"),
  status: EXIT_SUCCESS,
}

test! {
  name: dry_run_found_diff,
  justfile: "x:=``\n",
  args: ("--unstable", "--fmt", "--check"),
  stdout: "
  @@ -1 +1 @@
  -x:=``
  +x := ``
  ",
  stderr: "
    error: Formatted justfile differs from original.
  ",
  status: EXIT_FAILURE,
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

  assert_eq!(fs::read_to_string(&justfile).unwrap(), "x := 'hello'\n",);
}

#[test]
fn write_error() {
  let tempdir = temptree! {
    justfile: "x    :=    'hello'   ",
  };

  let test = Test::with_tempdir(tempdir)
    .no_justfile()
    .args(&["--fmt", "--unstable"])
    .status(EXIT_FAILURE)
    .stderr_regex(if cfg!(windows) {
      r"error: Failed to write justfile to `.*`: Access is denied. \(os error 5\)\n"
    } else {
      r"error: Failed to write justfile to `.*`: Permission denied \(os error 13\)\n"
    });

  let justfile_path = test.justfile_path();

  ("chmod", "400", &justfile_path).run();

  let _tempdir = test.run();

  assert_eq!(
    fs::read_to_string(&justfile_path).unwrap(),
    "x    :=    'hello'   "
  );
}

test! {
  name: alias_good,
  justfile: "
    alias f := foo

    foo:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    alias f := foo

    foo:
        echo foo
  ",
}

test! {
  name: alias_fix_indent,
  justfile: "
    alias f:=    foo

    foo:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    alias f := foo

    foo:
        echo foo
  ",
}

test! {
  name: assignment_singlequote,
  justfile: "
    foo := 'foo'
  ",
  args: ("--dump"),
  stdout: "
    foo := 'foo'
  ",
}

test! {
  name: assignment_doublequote,
  justfile: r#"
    foo := "foo"
  "#,
  args: ("--dump"),
  stdout: r#"
    foo := "foo"
  "#,
}

test! {
  name: assignment_indented_singlequote,
  justfile: "
    foo := '''
      foo
    '''
  ",
  args: ("--dump"),
  stdout: r"
    foo := '''
      foo
    '''
  ",
}

test! {
  name: assignment_indented_doublequote,
  justfile: r#"
    foo := """
      foo
    """
  "#,
  args: ("--dump"),
  stdout: r#"
    foo := """
      foo
    """
  "#,
}

test! {
  name: assignment_backtick,
  justfile: "
    foo := `foo`
  ",
  args: ("--dump"),
  stdout: "
    foo := `foo`
  ",
}

test! {
  name: assignment_indented_backtick,
  justfile: "
    foo := ```
      foo
    ```
  ",
  args: ("--dump"),
  stdout: "
    foo := ```
      foo
    ```
  ",
}

test! {
  name: assignment_name,
  justfile: "
    bar := 'bar'
    foo := bar
  ",
  args: ("--dump"),
  stdout: "
    bar := 'bar'
    foo := bar
  ",
}

test! {
  name: assignment_parenthized_expression,
  justfile: "
    foo := ('foo')
  ",
  args: ("--dump"),
  stdout: "
    foo := ('foo')
  ",
}

test! {
  name: assignment_export,
  justfile: "
    export foo := 'foo'
  ",
  args: ("--dump"),
  stdout: "
    export foo := 'foo'
  ",
}

test! {
  name: assignment_concat_values,
  justfile: "
    foo := 'foo' + 'bar'
  ",
  args: ("--dump"),
  stdout: "
    foo := 'foo' + 'bar'
  ",
}

test! {
  name: assignment_if_oneline,
  justfile: "
    foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
  ",
  args: ("--dump"),
  stdout: "
    foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
  ",
}

test! {
  name: assignment_if_multiline,
  justfile: "
    foo := if 'foo' != 'foo' {
      'foo'
    } else {
      'bar'
    }
  ",
  args: ("--dump"),
  stdout: "
    foo := if 'foo' != 'foo' { 'foo' } else { 'bar' }
  ",
}

test! {
  name: assignment_nullary_function,
  justfile: "
    foo := arch()
  ",
  args: ("--dump"),
  stdout: "
    foo := arch()
  ",
}

test! {
  name: assignment_unary_function,
  justfile: "
    foo := env_var('foo')
  ",
  args: ("--dump"),
  stdout: "
    foo := env_var('foo')
  ",
}

test! {
  name: assignment_binary_function,
  justfile: "
    foo := env_var_or_default('foo', 'bar')
  ",
  args: ("--dump"),
  stdout: "
    foo := env_var_or_default('foo', 'bar')
  ",
}

test! {
  name: assignment_path_functions,
  justfile: "
    foo  := without_extension('foo/bar.baz')
    foo2 := file_stem('foo/bar.baz')
    foo3 := parent_directory('foo/bar.baz')
    foo4 := file_name('foo/bar.baz')
    foo5 := extension('foo/bar.baz')
  ",
  args: ("--dump"),
  stdout: "
  foo := without_extension('foo/bar.baz')
  foo2 := file_stem('foo/bar.baz')
  foo3 := parent_directory('foo/bar.baz')
  foo4 := file_name('foo/bar.baz')
  foo5 := extension('foo/bar.baz')
  ",
}

test! {
  name: recipe_ordinary,
  justfile: "
    foo:
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        echo bar
  ",
}

test! {
  name: recipe_with_docstring,
  justfile: "
    # bar
    foo:
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    # bar
    foo:
        echo bar
  ",
}

test! {
  name: recipe_with_comments_in_body,
  justfile: "
    foo:
        # bar
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        # bar
        echo bar
  ",
}

test! {
  name: recipe_body_is_comment,
  justfile: "
    foo:
        # bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        # bar
  ",
}

test! {
  name: recipe_several_commands,
  justfile: "
    foo:
        echo bar
        echo baz
  ",
  args: ("--dump"),
  stdout: "
    foo:
        echo bar
        echo baz
  ",
}

test! {
  name: recipe_quiet,
  justfile: "
    @foo:
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    @foo:
        echo bar
  ",
}

test! {
  name: recipe_quiet_command,
  justfile: "
    foo:
        @echo bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        @echo bar
  ",
}

test! {
  name: recipe_quiet_comment,
  justfile: "
    foo:
        @# bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        @# bar
  ",
}

test! {
  name: recipe_ignore_errors,
  justfile: "
    foo:
        -echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo:
        -echo foo
  ",
}

test! {
  name: recipe_parameter,
  justfile: "
    foo BAR:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo BAR:
        echo foo
  ",
}

test! {
  name: recipe_parameter_default,
  justfile: "
    foo BAR='bar':
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo BAR='bar':
        echo foo
  ",
}

test! {
  name: recipe_parameter_envar,
  justfile: "
    foo $BAR:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo $BAR:
        echo foo
  ",
}

test! {
  name: recipe_parameter_default_envar,
  justfile: "
    foo $BAR='foo':
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo $BAR='foo':
        echo foo
  ",
}

test! {
  name: recipe_parameter_concat,
  justfile: "
    foo BAR=('bar' + 'baz'):
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo BAR=('bar' + 'baz'):
        echo foo
  ",
}

test! {
  name: recipe_parameters,
  justfile: "
    foo BAR BAZ:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo BAR BAZ:
        echo foo
  ",
}

test! {
  name: recipe_parameters_envar,
  justfile: "
    foo $BAR $BAZ:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo $BAR $BAZ:
        echo foo
  ",
}

test! {
  name: recipe_variadic_plus,
  justfile: "
    foo +BAR:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo +BAR:
        echo foo
  ",
}

test! {
  name: recipe_variadic_star,
  justfile: "
    foo *BAR:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo *BAR:
        echo foo
  ",
}

test! {
  name: recipe_positional_variadic,
  justfile: "
    foo BAR *BAZ:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo BAR *BAZ:
        echo foo
  ",
}

test! {
  name: recipe_variadic_default,
  justfile: "
    foo +BAR='bar':
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    foo +BAR='bar':
        echo foo
  ",
}

test! {
  name: recipe_parameter_in_body,
  justfile: "
    foo BAR:
        echo {{ BAR }}
  ",
  args: ("--dump"),
  stdout: "
    foo BAR:
        echo {{ BAR }}
  ",
}

test! {
  name: recipe_parameter_conditional,
  justfile: "
    foo BAR:
        echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
  ",
  args: ("--dump"),
  stdout: "
    foo BAR:
        echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
  ",
}

test! {
  name: recipe_escaped_braces,
  justfile: "
    foo BAR:
        echo '{{{{BAR}}}}'
  ",
  args: ("--dump"),
  stdout: "
    foo BAR:
        echo '{{{{BAR}}}}'
  ",
}

test! {
  name: recipe_assignment_in_body,
  justfile: "
    bar := 'bar'

    foo:
        echo $bar
  ",
  args: ("--dump"),
  stdout: "
    bar := 'bar'

    foo:
        echo $bar
  ",
}

test! {
  name: recipe_dependency,
  justfile: "
    bar:
        echo bar

    foo: bar
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    bar:
        echo bar

    foo: bar
        echo foo
  ",
}

test! {
  name: recipe_dependency_param,
  justfile: "
    bar BAR:
        echo bar

    foo: (bar 'bar')
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    bar BAR:
        echo bar

    foo: (bar 'bar')
        echo foo
  ",
}

test! {
  name: recipe_dependency_params,
  justfile: "
    bar BAR BAZ:
        echo bar

    foo: (bar 'bar' 'baz')
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    bar BAR BAZ:
        echo bar

    foo: (bar 'bar' 'baz')
        echo foo
  ",
}

test! {
  name: recipe_dependencies,
  justfile: "
    bar:
        echo bar

    baz:
        echo baz

    foo: baz bar
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    bar:
        echo bar

    baz:
        echo baz

    foo: baz bar
        echo foo
  ",
}

test! {
  name: recipe_dependencies_params,
  justfile: "
    bar BAR:
        echo bar

    baz BAZ:
        echo baz

    foo: (baz 'baz') (bar 'bar')
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    bar BAR:
        echo bar

    baz BAZ:
        echo baz

    foo: (baz 'baz') (bar 'bar')
        echo foo
  ",
}

test! {
  name: set_true_explicit,
  justfile: "
    set export := true
  ",
  args: ("--dump"),
  stdout: "
    set export := true
  ",
}

test! {
  name: set_true_implicit,
  justfile: "
    set export
  ",
  args: ("--dump"),
  stdout: "
    set export := true
  ",
}

test! {
  name: set_false,
  justfile: "
    set export := false
  ",
  args: ("--dump"),
  stdout: "
    set export := false
  ",
}

test! {
  name: set_shell,
  justfile: r#"
    set shell := ['sh', "-c"]
  "#,
  args: ("--dump"),
  stdout: r#"
    set shell := ['sh', "-c"]
  "#,
}

test! {
  name: comment,
  justfile: "
    # foo
  ",
  args: ("--dump"),
  stdout: "
    # foo
  ",
}

test! {
  name: comment_multiline,
  justfile: "
    # foo
    # bar
  ",
  args: ("--dump"),
  stdout: "
    # foo
    # bar
  ",
}

test! {
  name: comment_leading,
  justfile: "
    # foo

    foo := 'bar'
  ",
  args: ("--dump"),
  stdout: "
    # foo

    foo := 'bar'
  ",
}

test! {
  name: comment_trailing,
  justfile: "
    foo := 'bar'

    # foo
  ",
  args: ("--dump"),
  stdout: "
    foo := 'bar'

    # foo
  ",
}

test! {
  name: comment_before_recipe,
  justfile: "
    # foo

    foo:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    # foo

    foo:
        echo foo
  ",
}

test! {
  name: comment_before_docstring_recipe,
  justfile: "
    # bar

    # foo
    foo:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    # bar

    # foo
    foo:
        echo foo
  ",
}

test! {
  name: group_recipies,
  justfile: "
    foo:
        echo foo
    bar:
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    foo:
        echo foo

    bar:
        echo bar
  ",
}

test! {
  name: group_aliases,
  justfile: "
    alias f := foo

    alias b := bar

    foo:
        echo foo

    bar:
        echo bar
  ",
  args: ("--dump"),
  stdout: "
    alias f := foo
    alias b := bar

    foo:
        echo foo

    bar:
        echo bar
  ",
}

test! {
  name: group_assignments,
  justfile: "
    foo := 'foo'
    bar := 'bar'
  ",
  args: ("--dump"),
  stdout: "
    foo := 'foo'
    bar := 'bar'
  ",
}

test! {
  name: group_sets,
  justfile: "
    set export := true
    set positional-arguments := true
  ",
  args: ("--dump"),
  stdout: "
    set export := true
    set positional-arguments := true
  ",
}

test! {
  name: group_comments,
  justfile: "
    # foo

    # bar
  ",
  args: ("--dump"),
  stdout: "
    # foo
    # bar
  ",
}

test! {
  name: separate_recipes_aliases,
  justfile: "
    alias f := foo
    foo:
        echo foo
  ",
  args: ("--dump"),
  stdout: "
    alias f := foo

    foo:
        echo foo
  ",
}

test! {
  name: no_trailing_newline,
  justfile: "
    foo:
        echo foo",
  args: ("--dump"),
  stdout: "
    foo:
        echo foo
  ",
}

test! {
  name: subsequent,
  justfile: "
    bar:
    foo: && bar
        echo foo",
  args: ("--dump"),
  stdout: "
    bar:

    foo: && bar
        echo foo
  ",
}
