use super::*;

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
    .status(2);
}

#[test]
fn check_ok() {
  Test::new()
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
    .success();
}

#[test]
fn from_stdin() {
  Test::new()
    .args(["--fmt", "--justfile", "-"])
    .stdin("x:=``\n")
    .stdout("x := ``\n")
    .success();
}

#[test]
fn already_formatted_from_stdin() {
  Test::new()
    .args(["--fmt", "--justfile", "-"])
    .stdin("x := ``\n")
    .stdout("x := ``\n")
    .success();
}

#[test]
fn check_from_stdin() {
  Test::new()
    .args(["--fmt", "--check", "--justfile", "-"])
    .stdin("x:=``\n")
    .stdout(
      "
        -x:=``
        +x := ``
      ",
    )
    .stderr(
      "
        error: formatted justfile differs from original
      ",
    )
    .failure();
}

#[test]
fn check_found_diff() {
  Test::new()
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
        error: formatted justfile differs from original
      ",
    )
    .failure();
}

#[test]
fn check_found_diff_quiet() {
  Test::new()
    .arg("--fmt")
    .arg("--check")
    .arg("--quiet")
    .justfile("x:=``\n")
    .failure();
}

#[test]
fn check_diff_color() {
  Test::new()
        .justfile("x:=``\n")
        .arg("--fmt")
        .arg("--check")
        .arg("--color")
        .arg("always")
        .stdout("\n    \u{1b}[31m-x:=``\n    \u{1b}[0m\u{1b}[32m+x := ``\n    \u{1b}[0m")
        .stderr("\n    \u{1b}[1;31merror\u{1b}[0m: \u{1b}[1mformatted justfile differs from original\u{1b}[0m\n  ")
        .failure();
}

#[test]
fn write_error() {
  // skip this test if running as root, since root can write files even if
  // permissions would otherwise forbid it
  #[cfg(not(windows))]
  if nix::unistd::getuid() == nix::unistd::ROOT {
    return;
  }

  let test = Test::new()
    .write("justfile", "x    :=    'hello'   ")
    .arg("--fmt")
    .stderr_regex(if cfg!(windows) {
      r"error: failed to write justfile to `.*`: Access is denied. \(os error 5\)\n"
    } else {
      r"error: failed to write justfile to `.*`: Permission denied \(os error 13\)\n"
    });

  let justfile_path = test.justfile_path();

  let output = Command::new("chmod")
    .arg("400")
    .arg(&justfile_path)
    .output()
    .unwrap();

  assert!(output.status.success());

  let _tempdir = test.failure();

  assert_eq!(
    fs::read_to_string(&justfile_path).unwrap(),
    "x    :=    'hello'   "
  );
}

#[test]
fn alias_good() {
  assert_dump(
    "
        alias f := foo

        foo:
            echo foo
      ",
    "
        alias f := foo

        foo:
            echo foo
      ",
  );
}

#[test]
fn alias_fix_indent() {
  assert_dump(
    "
        alias f:=    foo

        foo:
            echo foo
      ",
    "
        alias f := foo

        foo:
            echo foo
      ",
  );
}

#[test]
fn assignment_singlequote() {
  assert_dump(
    "
        foo := 'foo'
      ",
    "
        foo := 'foo'
      ",
  );
}

#[test]
fn assignment_doublequote() {
  assert_dump(
    r#"
        foo := "foo"
      "#,
    r#"
        foo := "foo"
      "#,
  );
}

#[test]
fn assignment_indented_singlequote() {
  assert_dump(
    "
        foo := '''
          foo
        '''
      ",
    "
        foo := '''
          foo
        '''
      ",
  );
}

#[test]
fn assignment_indented_doublequote() {
  assert_dump(
    r#"
        foo := """
          foo
        """
      "#,
    r#"
        foo := """
          foo
        """
      "#,
  );
}

#[test]
fn assignment_backtick() {
  assert_dump(
    "
        foo := `foo`
      ",
    "
        foo := `foo`
      ",
  );
}

#[test]
fn assignment_indented_backtick() {
  assert_dump(
    "
        foo := ```
          foo
        ```
      ",
    "
        foo := ```
          foo
        ```
      ",
  );
}

#[test]
fn assignment_name() {
  assert_dump(
    "
        bar := 'bar'
        foo := bar
      ",
    "
        bar := 'bar'
        foo := bar
      ",
  );
}

#[test]
fn assignment_parenthesized_expression() {
  assert_dump(
    "
        foo := ('foo')
      ",
    "
        foo := ('foo')
      ",
  );
}

#[test]
fn assignment_export() {
  assert_dump(
    "
        export foo := 'foo'
      ",
    "
        export foo := 'foo'
      ",
  );
}

#[test]
fn assignment_concat_values() {
  assert_dump(
    "
        foo := 'foo' + 'bar'
      ",
    "
        foo := 'foo' + 'bar'
      ",
  );
}

#[test]
fn assignment_list_concat_values() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
        set lists

        foo := ['bar'] ++ ['baz']
      ",
    )
    .unstable()
    .stdout(
      "
        set lists

        foo := ['bar'] ++ ['baz']
      ",
    )
    .success();
}

#[test]
fn assignment_if_oneline() {
  assert_dump(
    "
        foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
      ",
    "
        foo := if 'foo' == 'foo' { 'foo' } else { 'bar' }
      ",
  );
}

#[test]
fn assignment_if_without_else() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
        set lists

        foo := if 'foo' == 'foo' { 'foo' }
      ",
    )
    .unstable()
    .stdout(
      "
        set lists

        foo := if 'foo' == 'foo' { 'foo' }
      ",
    )
    .success();
}

#[test]
fn assignment_if_multiline() {
  assert_dump(
    "
        foo := if 'foo' != 'foo' {
          'foo'
        } else {
          'bar'
        }
      ",
    "
        foo := if 'foo' != 'foo' { 'foo' } else { 'bar' }
      ",
  );
}

#[test]
fn assignment_nullary_function() {
  assert_dump(
    "
        foo := arch()
      ",
    "
        foo := arch()
      ",
  );
}

#[test]
fn assignment_unary_function() {
  assert_dump(
    "
        foo := env_var('foo')
      ",
    "
        foo := env_var('foo')
      ",
  );
}

#[test]
fn assignment_binary_function() {
  assert_dump(
    "
        foo := env_var_or_default('foo', 'bar')
      ",
    "
        foo := env_var_or_default('foo', 'bar')
      ",
  );
}

#[test]
fn assignment_path_functions() {
  assert_dump(
    "
        foo  := without_extension('foo/bar.baz')
        foo2 := file_stem('foo/bar.baz')
        foo3 := parent_directory('foo/bar.baz')
        foo4 := file_name('foo/bar.baz')
        foo5 := extension('foo/bar.baz')
      ",
    "
        foo := without_extension('foo/bar.baz')
        foo2 := file_stem('foo/bar.baz')
        foo3 := parent_directory('foo/bar.baz')
        foo4 := file_name('foo/bar.baz')
        foo5 := extension('foo/bar.baz')
      ",
  );
}

#[test]
fn recipe_ordinary() {
  assert_dump(
    "
        foo:
            echo bar
      ",
    "
        foo:
            echo bar
      ",
  );
}

#[test]
fn recipe_with_docstring() {
  assert_dump(
    "
        # bar
        foo:
            echo bar
      ",
    "
        # bar
        foo:
            echo bar
      ",
  );
}

#[test]
fn recipe_with_comments_in_body() {
  assert_dump(
    "
        foo:
            # bar
            echo bar
      ",
    "
        foo:
            # bar
            echo bar
      ",
  );
}

#[test]
fn recipe_body_is_comment() {
  assert_dump(
    "
        foo:
            # bar
      ",
    "
        foo:
            # bar
      ",
  );
}

#[test]
fn recipe_several_commands() {
  assert_dump(
    "
        foo:
            echo bar
            echo baz
      ",
    "
        foo:
            echo bar
            echo baz
      ",
  );
}

#[test]
fn recipe_quiet() {
  assert_dump(
    "
        @foo:
            echo bar
      ",
    "
        @foo:
            echo bar
      ",
  );
}

#[test]
fn recipe_quiet_command() {
  assert_dump(
    "
        foo:
            @echo bar
      ",
    "
        foo:
            @echo bar
      ",
  );
}

#[test]
fn recipe_quiet_comment() {
  assert_dump(
    "
        foo:
            @# bar
      ",
    "
        foo:
            @# bar
      ",
  );
}

#[test]
fn recipe_ignore_errors() {
  assert_dump(
    "
        foo:
            -echo foo
      ",
    "
        foo:
            -echo foo
      ",
  );
}

#[test]
fn recipe_parameter() {
  assert_dump(
    "
        foo BAR:
            echo foo
      ",
    "
        foo BAR:
            echo foo
      ",
  );
}

#[test]
fn recipe_parameter_default() {
  assert_dump(
    "
        foo BAR='bar':
            echo foo
      ",
    "
        foo BAR='bar':
            echo foo
      ",
  );
}

#[test]
fn recipe_parameter_envar() {
  assert_dump(
    "
        foo $BAR:
            echo foo
      ",
    "
        foo $BAR:
            echo foo
      ",
  );
}

#[test]
fn recipe_parameter_default_envar() {
  assert_dump(
    "
        foo $BAR='foo':
            echo foo
      ",
    "
        foo $BAR='foo':
            echo foo
      ",
  );
}

#[test]
fn recipe_parameter_concat() {
  assert_dump(
    "
        foo BAR=('bar' + 'baz'):
            echo foo
      ",
    "
        foo BAR=('bar' + 'baz'):
            echo foo
      ",
  );
}

#[test]
fn recipe_parameters() {
  assert_dump(
    "
        foo BAR BAZ:
            echo foo
      ",
    "
        foo BAR BAZ:
            echo foo
      ",
  );
}

#[test]
fn recipe_parameters_envar() {
  assert_dump(
    "
        foo $BAR $BAZ:
            echo foo
      ",
    "
        foo $BAR $BAZ:
            echo foo
      ",
  );
}

#[test]
fn recipe_variadic_plus() {
  assert_dump(
    "
        foo +BAR:
            echo foo
      ",
    "
        foo +BAR:
            echo foo
      ",
  );
}

#[test]
fn recipe_variadic_star() {
  assert_dump(
    "
        foo *BAR:
            echo foo
      ",
    "
        foo *BAR:
            echo foo
      ",
  );
}

#[test]
fn recipe_positional_variadic() {
  assert_dump(
    "
        foo BAR *BAZ:
            echo foo
      ",
    "
        foo BAR *BAZ:
            echo foo
      ",
  );
}

#[test]
fn recipe_variadic_default() {
  assert_dump(
    "
        foo +BAR='bar':
            echo foo
      ",
    "
        foo +BAR='bar':
            echo foo
      ",
  );
}

#[test]
fn recipe_parameter_in_body() {
  assert_dump(
    "
        foo BAR:
            echo {{ BAR }}
      ",
    "
        foo BAR:
            echo {{ BAR }}
      ",
  );
}

#[test]
fn recipe_parameter_conditional() {
  assert_dump(
    "
        foo BAR:
            echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
      ",
    "
        foo BAR:
            echo {{ if 'foo' == 'foo' { 'foo' } else { 'bar' } }}
      ",
  );
}

#[test]
fn recipe_escaped_braces() {
  assert_dump(
    "
        foo BAR:
            echo '{{{{BAR}}}}'
      ",
    "
        foo BAR:
            echo '{{{{BAR}}}}'
      ",
  );
}

#[test]
fn recipe_assignment_in_body() {
  assert_dump(
    "
        bar := 'bar'

        foo:
            echo $bar
      ",
    "
        bar := 'bar'

        foo:
            echo $bar
      ",
  );
}

#[test]
fn recipe_dependency() {
  assert_dump(
    "
        bar:
            echo bar

        foo: bar
            echo foo
      ",
    "
        bar:
            echo bar

        foo: bar
            echo foo
      ",
  );
}

#[test]
fn recipe_dependency_param() {
  assert_dump(
    "
        bar BAR:
            echo bar

        foo: (bar 'bar')
            echo foo
      ",
    "
        bar BAR:
            echo bar

        foo: (bar 'bar')
            echo foo
      ",
  );
}

#[test]
fn recipe_dependency_params() {
  assert_dump(
    "
        bar BAR BAZ:
            echo bar

        foo: (bar 'bar' 'baz')
            echo foo
      ",
    "
        bar BAR BAZ:
            echo bar

        foo: (bar 'bar' 'baz')
            echo foo
      ",
  );
}

#[test]
fn recipe_dependencies() {
  assert_dump(
    "
        bar:
            echo bar

        baz:
            echo baz

        foo: baz bar
            echo foo
      ",
    "
        bar:
            echo bar

        baz:
            echo baz

        foo: baz bar
            echo foo
      ",
  );
}

#[test]
fn recipe_dependencies_params() {
  assert_dump(
    "
        bar BAR:
            echo bar

        baz BAZ:
            echo baz

        foo: (baz 'baz') (bar 'bar')
            echo foo
      ",
    "
        bar BAR:
            echo bar

        baz BAZ:
            echo baz

        foo: (baz 'baz') (bar 'bar')
            echo foo
      ",
  );
}

#[test]
fn set_true_explicit() {
  assert_dump(
    "
        set export := true
      ",
    "
        set export
      ",
  );
}

#[test]
fn set_true_implicit() {
  assert_dump(
    "
        set export
      ",
    "
        set export
      ",
  );
}

#[test]
fn set_false() {
  assert_dump(
    "
        set export := false
      ",
    "
        set export := false
      ",
  );
}

#[test]
fn set_shell() {
  assert_dump(
    r#"
        set shell := ['sh', "-c"]
      "#,
    r#"
        set shell := ['sh', "-c"]
      "#,
  );
}

#[test]
fn comment() {
  assert_dump(
    "
        # foo
      ",
    "
        # foo
      ",
  );
}

#[test]
fn comment_multiline() {
  assert_dump(
    "
        # foo
        # bar
      ",
    "
        # foo
        # bar
      ",
  );
}

#[test]
fn comment_leading() {
  assert_dump(
    "
        # foo

        foo := 'bar'
      ",
    "
        # foo

        foo := 'bar'
      ",
  );
}

#[test]
fn comment_trailing() {
  assert_dump(
    "
        foo := 'bar'

        # foo
      ",
    "
        foo := 'bar'

        # foo
      ",
  );
}

#[test]
fn comment_before_recipe() {
  assert_dump(
    "
        # foo

        foo:
            echo foo
      ",
    "
        # foo

        foo:
            echo foo
      ",
  );
}

#[test]
fn comment_before_docstring_recipe() {
  assert_dump(
    "
        # bar

        # foo
        foo:
            echo foo
      ",
    "
        # bar

        # foo
        foo:
            echo foo
      ",
  );
}

#[test]
fn newlines_between_items_is_preserved() {
  assert_dump(
    "
        foo:

        bar:
      ",
    "
        foo:

        bar:
      ",
  );
}

#[test]
fn multiple_newlines_between_items_are_collapsed() {
  assert_dump(
    "
        foo:


        bar:
      ",
    "
        foo:

        bar:
      ",
  );
}

#[test]
fn newline_after_recipe_with_body_is_preserved() {
  assert_dump(
    "
        foo:
            echo FOO

        bar:
      ",
    "
        foo:
            echo FOO

        bar:
      ",
  );
}

#[test]
fn adjacency_is_respected() {
  assert_dump(
    "
        foo:
        bar:
      ",
    "
        foo:
        bar:
      ",
  );
}

#[test]
fn no_trailing_newline() {
  assert_dump(
    "
    foo:
        echo foo",
    "
        foo:
            echo foo
      ",
  );
}

#[test]
fn subsequent() {
  assert_dump(
    "
        bar:

        foo: && bar
            echo foo
      ",
    "
        bar:

        foo: && bar
            echo foo
      ",
  );
}

#[test]
fn exported_parameter() {
  assert_dump("foo +$f:", "foo +$f:\n");
}

#[test]
fn multi_argument_attribute() {
  assert_dump(
    "
        [script('a', 'b', 'c')]
        foo:
      ",
    "
        [script('a', 'b', 'c')]
        foo:
      ",
  );
}

#[test]
fn doc_attribute_suppresses_comment() {
  assert_dump(
    "
        # COMMENT
        [doc('ATTRIBUTE')]
        foo:
      ",
    "
        # COMMENT
        [doc('ATTRIBUTE')]
        foo:
      ",
  );
}

#[test]
fn doc_attribute_expression() {
  assert_dump(
    "
        [doc('f' + 'oo')]
        foo:
      ",
    "
        [doc('f' + 'oo')]
        foo:
      ",
  );
}

#[test]
fn unchanged_justfiles_are_not_written_to_disk() {
  let tmp = tempdir();

  let justfile = tmp.path().join("justfile");

  fs::write(&justfile, "\n").unwrap();

  let mut permissions = fs::metadata(&justfile).unwrap().permissions();
  permissions.set_readonly(true);
  fs::set_permissions(&justfile, permissions).unwrap();

  Test::with_tempdir(tmp).arg("--fmt").success();
}

#[test]
fn if_else() {
  assert_dump(
    "
        x := if '' == '' { '' } else if '' == '' { '' } else { '' }
      ",
    "
        x := if '' == '' { '' } else if '' == '' { '' } else { '' }
      ",
  );
}

#[test]
fn private_variable() {
  assert_dump(
    "
        [private]
        foo := 'bar'
      ",
    "
        [private]
        foo := 'bar'
      ",
  );
}

#[test]
fn private_alias() {
  assert_dump(
    "
        [private]
        alias f := foo

        foo:
            echo foo
      ",
    "
        [private]
        alias f := foo

        foo:
            echo foo
      ",
  );
}

#[test]
fn module_groups_are_preserved() {
  Test::new()
    .justfile(
      r#"
        [group('bar')]
        [group("baz")]
        mod foo
      "#,
    )
    .write("foo.just", "")
    .arg("--dump")
    .stdout(
      r#"
        [group('bar')]
        [group("baz")]
        mod foo
      "#,
    )
    .success();
}

#[test]
fn module_private_is_preserved() {
  Test::new()
    .justfile(
      "
        [private]
        mod foo
      ",
    )
    .write("foo.just", "")
    .arg("--dump")
    .stdout(
      "
        [private]
        mod foo
      ",
    )
    .success();
}

#[test]
fn module_docs_are_preserved() {
  Test::new()
    .justfile(
      "
        # bar
        mod foo
      ",
    )
    .write("foo.just", "")
    .arg("--dump")
    .stdout(
      "
        # bar
        mod foo
      ",
    )
    .success();
}

#[test]
fn arg_attribute_long() {
  assert_dump(
    "
        [arg('bar', long='bar')]
        @foo bar:
      ",
    "
        [arg('bar', long='bar')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_long_bare() {
  assert_dump(
    "
        [arg('bar', long)]
        @foo bar:
      ",
    "
        [arg('bar', long)]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_short_bare() {
  assert_dump(
    "
        [arg('bar', short)]
        @foo bar:
      ",
    "
        [arg('bar', short)]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_pattern() {
  assert_dump(
    "
        [arg('bar', pattern='bar')]
        @foo bar:
      ",
    "
        [arg('bar', pattern='bar')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_pattern_expression() {
  assert_dump(
    "
        [arg('bar', pattern='b' + 'ar')]
        @foo bar:
      ",
    "
        [arg('bar', pattern='b' + 'ar')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_long_and_pattern() {
  assert_dump(
    "
        [arg('bar', long='foo', pattern='baz')]
        @foo bar:
      ",
    "
        [arg('bar', long='foo', pattern='baz')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_help() {
  assert_dump(
    "
        [arg('bar', help='foo')]
        @foo bar:
      ",
    "
        [arg('bar', help='foo')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_help_expression() {
  assert_dump(
    "
        [arg('bar', help='f' + 'oo')]
        @foo bar:
      ",
    "
        [arg('bar', help='f' + 'oo')]
        @foo bar:
      ",
  );
}

#[test]
fn arg_attribute_flag() {
  Test::new()
    .justfile(
      "
        set lists

        [arg('bar', long='bar', flag)]
        @foo bar:
      ",
    )
    .unstable()
    .arg("--dump")
    .stdout(
      "
        set lists

        [arg('bar', long='bar', flag)]
        @foo bar:
      ",
    )
    .success();
}

#[test]
fn arg_attribute_value() {
  assert_dump(
    "
        BAZ := 'baz'

        [arg('bar', long='bar', value=BAZ + env('FOO', 'foo'))]
        @foo bar:
      ",
    "
        BAZ := 'baz'

        [arg('bar', long='bar', value=BAZ + env('FOO', 'foo'))]
        @foo bar:
      ",
  );
}

#[test]
fn missing_import_file() {
  Test::new()
    .args(["--fmt", "--check"])
    .justfile("import 'foo'\n")
    .success();
}

#[test]
fn missing_module_file() {
  Test::new()
    .args(["--fmt", "--check"])
    .justfile("mod foo\n")
    .success();
}

#[test]
fn undefined_variable() {
  Test::new()
    .args(["--fmt", "--check"])
    .justfile(
      "
        foo:
            echo {{ ABC }}
      ",
    )
    .success();
}

#[test]
fn indentation_two_spaces() {
  Test::new()
    .args(["--fmt", "--check", "--indentation", "  "])
    .justfile("foo:\n  echo bar\n")
    .success();
}

#[test]
fn indentation_tab() {
  Test::new()
    .args(["--fmt", "--check", "--indentation", "\t"])
    .justfile("foo:\n\techo bar\n")
    .success();
}

#[test]
fn indentation_check_with_custom() {
  Test::new()
    .args(["--fmt", "--check", "--indentation", "  "])
    .justfile("foo:\n    echo bar\n")
    .stdout(" foo:\n-    echo bar\n+  echo bar\n")
    .stderr(
      "
        error: formatted justfile differs from original
      ",
    )
    .failure();
}

#[test]
fn dump_indentation_two_spaces() {
  Test::new()
    .args(["--dump", "--indentation", "  "])
    .justfile(
      "
        foo:
            echo bar
      ",
    )
    .stdout("foo:\n  echo bar\n")
    .success();
}

#[test]
fn dump_indentation_tab() {
  Test::new()
    .args(["--dump", "--indentation", "\t"])
    .justfile(
      "
        foo:
            echo bar
      ",
    )
    .stdout("foo:\n\techo bar\n")
    .success();
}

#[test]
fn indentation_env() {
  Test::new()
    .arg("--dump")
    .env("JUST_INDENTATION", "  ")
    .justfile(
      "
        foo:
            echo bar
      ",
    )
    .stdout("foo:\n  echo bar\n")
    .success();
}

#[test]
fn indentation_setting_dump() {
  Test::new()
    .arg("--dump")
    .justfile(
      "
        set indentation := ' '

        foo:
            echo bar
      ",
    )
    .stdout(
      "
        set indentation := ' '

        foo:
         echo bar
      ",
    )
    .success();
}

#[test]
fn indentation_setting_format() {
  let output = Test::new()
    .arg("--fmt")
    .justfile(
      "
        set indentation := ' '

        foo:
            echo bar
      ",
    )
    .stderr_regex("wrote justfile to `.*/justfile`\n")
    .success();

  assert_eq!(
    fs::read_to_string(output.tempdir.path().join("justfile")).unwrap(),
    "set indentation := ' '

foo:
 echo bar
",
  );
}

#[test]
fn indentation_flag_overrides_setting_dump() {
  Test::new()
    .args(["--dump", "--indentation", "      "])
    .justfile(
      "
        set indentation := ' '

        foo:
          echo bar
      ",
    )
    .stdout(
      "
        set indentation := ' '

        foo:
              echo bar
      ",
    )
    .success();
}

#[test]
fn indentation_flag_overrides_setting_format() {
  let output = Test::new()
    .args(["--fmt", "--indentation", "      "])
    .justfile(
      "
        set indentation := ' '

        foo:
          echo bar
      ",
    )
    .stderr_regex("wrote justfile to `.*/justfile`\n")
    .success();

  assert_eq!(
    fs::read_to_string(output.tempdir.path().join("justfile")).unwrap(),
    "set indentation := ' '

foo:
      echo bar
",
  );
}

#[test]
fn indentation_setting_must_be_string_literal() {
  Test::new()
    .justfile("set indentation := ('  ' + '  ')")
    .stderr(
      "
        error: `indentation` setting must be a plain string literal
         ——▶ justfile:1:5
          │
        1 │ set indentation := ('  ' + '  ')
          │     ^^^^^^^^^^^
      ",
    )
    .failure();
}

#[test]
fn indentation_setting_invalid_values() {
  #[track_caller]
  fn case(justfile: &str, stderr: &str) {
    Test::new().justfile(justfile).stderr(stderr).failure();
  }

  case(
    "set indentation := ''",
    "
      error: indentation must not be empty
       ——▶ justfile:1:20
        │
      1 │ set indentation := ''
        │                    ^^
    ",
  );

  case(
    "set indentation := 'x'",
    "
      error: indentation must be spaces or tabs
       ——▶ justfile:1:20
        │
      1 │ set indentation := 'x'
        │                    ^^^
    ",
  );

  case(
    "set indentation := \" \\t\"",
    "
      error: indentation may not be mixed
       ——▶ justfile:1:20
        │
      1 │ set indentation := \" \\t\"
        │                    ^^^^^
    ",
  );
}

#[test]
fn multi_line_comments_before_recipes_are_not_broken_up() {
  assert_dump(
    "
        # foo
        # bar
        baz:
      ",
    "
        # foo
        # bar
        baz:
      ",
  );
}

#[test]
fn trailing_comment_assignment() {
  assert_dump(
    "
        foo := 'bar' # baz
      ",
    "
        foo := 'bar' # baz
      ",
  );
}

#[test]
fn trailing_comment_alias() {
  assert_dump(
    "
        alias f := foo # baz
        foo:
      ",
    "
        alias f := foo # baz
        foo:
      ",
  );
}

#[test]
fn trailing_comment_bodyless_recipe() {
  assert_dump(
    "
        foo: # bar
      ",
    "
        foo: # bar
      ",
  );
}

#[test]
fn trailing_comment_set() {
  assert_dump(
    "
        set quiet # foo
      ",
    "
        set quiet # foo
      ",
  );
}

#[test]
fn trailing_comment_unexport() {
  assert_dump(
    "
        unexport FOO # bar
      ",
    "
        unexport FOO # bar
      ",
  );
}

#[test]
fn trailing_comment_does_not_become_doc_comment() {
  assert_dump(
    "
        foo := 'bar' # baz
        qux:
      ",
    "
        foo := 'bar' # baz
        qux:
      ",
  );
}

#[test]
fn trailing_comment_recipe_with_body_is_stripped() {
  assert_dump(
    "
        foo: # bar
          echo baz
      ",
    "
        foo:
            echo baz
      ",
  );
}

#[test]
fn trailing_comment_export() {
  assert_dump(
    "
        export foo := 'bar' # baz
      ",
    "
        export foo := 'bar' # baz
      ",
  );
}

#[test]
fn trailing_comment_recipe_with_dependencies_and_body_is_stripped() {
  assert_dump(
    "
        foo: bar # baz
          echo qux

        bar:
      ",
    "
        foo: bar
            echo qux

        bar:
      ",
  );
}

#[test]
fn multiple_trailing_comments() {
  assert_dump(
    "
        foo := 'bar' # comment1
        baz := 'qux' # comment2
      ",
    "
        foo := 'bar' # comment1
        baz := 'qux' # comment2
      ",
  );
}

#[test]
fn trailing_comments_separated_by_blank_line() {
  assert_dump(
    "
        foo := 'bar' # comment1

        baz := 'qux' # comment2
      ",
    "
        foo := 'bar' # comment1

        baz := 'qux' # comment2
      ",
  );
}
