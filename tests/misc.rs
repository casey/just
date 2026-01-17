use super::*;

#[test]
fn alias_listing() {
  Test::new()
    .arg("--list")
    .justfile(
      "
    foo:
      echo foo

    alias f := foo
  ",
    )
    .stdout(
      "
    Available recipes:
        foo # [alias: f]
  ",
    )
    .success();
}

#[test]
fn alias_listing_with_doc() {
  Test::new()
    .justfile(
      "
        # foo command
        foo:
          echo foo

        alias f := foo
      ",
    )
    .arg("--list")
    .stdout(
      "
      Available recipes:
          foo # foo command [alias: f]
    ",
    )
    .success();
}

#[test]
fn alias_listing_multiple_aliases() {
  Test::new()
    .arg("--list")
    .justfile("foo:\n  echo foo\nalias f := foo\nalias fo := foo")
    .stdout(
      "
    Available recipes:
        foo # [aliases: f, fo]
  ",
    )
    .success();
}

#[test]
fn alias_listing_parameters() {
  Test::new()
    .args(["--list"])
    .justfile("foo PARAM='foo':\n  echo {{PARAM}}\nalias f := foo")
    .stdout(
      "
    Available recipes:
        foo PARAM='foo' # [alias: f]
  ",
    )
    .success();
}

#[test]
fn alias_listing_private() {
  Test::new()
    .arg("--list")
    .justfile("foo PARAM='foo':\n  echo {{PARAM}}\nalias _f := foo")
    .stdout(
      "
    Available recipes:
        foo PARAM='foo'
  ",
    )
    .success();
}

#[test]
fn alias() {
  Test::new()
    .arg("f")
    .justfile("foo:\n  echo foo\nalias f := foo")
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn alias_with_parameters() {
  Test::new()
    .arg("f")
    .arg("bar")
    .justfile("foo value='foo':\n  echo {{value}}\nalias f := foo")
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn bad_setting() {
  Test::new()
    .justfile(
      "
    set foo
  ",
    )
    .stderr(
      "
  error: Unknown setting `foo`
   ——▶ justfile:1:5
    │
  1 │ set foo
    │     ^^^
  ",
    )
    .failure();
}

#[test]
fn bad_setting_with_keyword_name() {
  Test::new()
    .justfile(
      "
    set if := 'foo'
  ",
    )
    .stderr(
      "
  error: Unknown setting `if`
   ——▶ justfile:1:5
    │
  1 │ set if := 'foo'
    │     ^^
  ",
    )
    .failure();
}

#[test]
fn alias_with_dependencies() {
  Test::new()
    .arg("b")
    .justfile("foo:\n  echo foo\nbar: foo\nalias b := bar")
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn duplicate_alias() {
  Test::new()
    .justfile("alias foo := bar\nalias foo := baz\n")
    .stderr(
      "
    error: Alias `foo` first defined on line 1 is redefined on line 2
     ——▶ justfile:2:7
      │
    2 │ alias foo := baz
      │       ^^^
  ",
    )
    .failure();
}

#[test]
fn unknown_alias_target() {
  Test::new()
    .justfile("alias foo := bar\n")
    .stderr(
      "
    error: Alias `foo` has an unknown target `bar`
     ——▶ justfile:1:7
      │
    1 │ alias foo := bar
      │       ^^^
  ",
    )
    .failure();
}

#[test]
fn alias_shadows_recipe() {
  Test::new()
    .justfile("bar:\n  echo bar\nalias foo := bar\nfoo:\n  echo foo")
    .stderr(
      "
    error: Alias `foo` defined on line 3 is redefined as a recipe on line 4
     ——▶ justfile:4:1
      │
    4 │ foo:
      │ ^^^
  ",
    )
    .failure();
}

#[test]
fn default() {
  Test::new()
    .justfile("default:\n echo hello\nother: \n echo bar")
    .stdout("hello\n")
    .stderr("echo hello\n")
    .success();
}

#[test]
fn quiet() {
  Test::new()
    .justfile("default:\n @echo hello")
    .stdout("hello\n")
    .success();
}

#[test]
fn verbose() {
  Test::new()
    .arg("--verbose")
    .justfile("default:\n @echo hello")
    .stdout("hello\n")
    .stderr("===> Running recipe `default`...\necho hello\n")
    .success();
}

#[test]
fn order() {
  Test::new()
    .arg("a")
    .arg("d")
    .justfile(
      "
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
    )
    .stdout("a\nb\nc\nd\n")
    .stderr("echo a\necho b\necho c\necho d\n")
    .success();
}

#[test]
fn select() {
  Test::new()
    .arg("d")
    .arg("c")
    .justfile("b:\n  @echo b\na:\n  @echo a\nd:\n  @echo d\nc:\n  @echo c")
    .stdout("d\nc\n")
    .success();
}

#[test]
fn print() {
  Test::new()
    .arg("d")
    .arg("c")
    .justfile("b:\n  echo b\na:\n  echo a\nd:\n  echo d\nc:\n  echo c")
    .stdout("d\nc\n")
    .stderr("echo d\necho c\n")
    .success();
}

#[test]
fn status_passthrough() {
  Test::new()
    .arg("recipe")
    .justfile(
      "

hello:

recipe:
  @exit 100",
    )
    .stderr("error: Recipe `recipe` failed on line 5 with exit code 100\n")
    .status(100);
}

#[test]
fn unknown_dependency() {
  Test::new()
    .justfile("bar:\nhello:\nfoo: bar baaaaaaaz hello")
    .stderr(
      "
    error: Recipe `foo` has unknown dependency `baaaaaaaz`
     ——▶ justfile:3:10
      │
    3 │ foo: bar baaaaaaaz hello
      │          ^^^^^^^^^
  ",
    )
    .failure();
}

#[test]
fn backtick_success() {
  Test::new()
    .justfile("a := `printf Hello,`\nbar:\n printf '{{a + `printf ' world.'`}}'")
    .stdout("Hello, world.")
    .stderr("printf 'Hello, world.'\n")
    .success();
}

#[test]
fn backtick_trimming() {
  Test::new()
    .justfile("a := `echo Hello,`\nbar:\n echo '{{a + `echo ' world.'`}}'")
    .stdout("Hello, world.\n")
    .stderr("echo 'Hello, world.'\n")
    .success();
}

#[test]
fn backtick_code_assignment() {
  Test::new()
    .justfile("b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'")
    .stderr(
      "
    error: Backtick failed with exit code 100
     ——▶ justfile:2:6
      │
    2 │ a := `exit 100`
      │      ^^^^^^^^^^
  ",
    )
    .status(100);
}

#[test]
fn backtick_code_interpolation() {
  Test::new()
    .justfile("b := a\na := `echo hello`\nbar:\n echo '{{`exit 200`}}'")
    .stderr(
      "
    error: Backtick failed with exit code 200
     ——▶ justfile:4:10
      │
    4 │  echo '{{`exit 200`}}'
      │          ^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_mod() {
  Test::new()
    .justfile("f:\n 無{{`exit 200`}}")
    .stderr(
      "
    error: Backtick failed with exit code 200
     ——▶ justfile:2:7
      │
    2 │  無{{`exit 200`}}
      │      ^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_tab() {
  Test::new()
    .justfile(
      "
    backtick-fail:
    \techo {{`exit 200`}}
  ",
    )
    .stderr(
      "    error: Backtick failed with exit code 200
     ——▶ justfile:2:9
      │
    2 │     echo {{`exit 200`}}
      │            ^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_tabs() {
  Test::new()
    .justfile(
      "
    backtick-fail:
    \techo {{\t`exit 200`}}
  ",
    )
    .stderr(
      "error: Backtick failed with exit code 200
 ——▶ justfile:2:10
  │
2 │     echo {{    `exit 200`}}
  │                ^^^^^^^^^^
",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_inner_tab() {
  Test::new()
    .justfile(
      "
    backtick-fail:
    \techo {{\t`exit\t\t200`}}
  ",
    )
    .stderr(
      "
    error: Backtick failed with exit code 200
     ——▶ justfile:2:10
      │
    2 │     echo {{    `exit        200`}}
      │                ^^^^^^^^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_leading_emoji() {
  Test::new()
    .justfile(
      "
    backtick-fail:
    \techo 😬{{`exit 200`}}
  ",
    )
    .stderr(
      "
    error: Backtick failed with exit code 200
     ——▶ justfile:2:13
      │
    2 │     echo 😬{{`exit 200`}}
      │              ^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_interpolation_unicode_hell() {
  Test::new()
    .justfile(
      "
    backtick-fail:
    \techo \t\t\t😬鎌鼬{{\t\t`exit 200 # \t\t\tabc`}}\t\t\t😬鎌鼬
  ",
    )
    .stderr(
      "
    error: Backtick failed with exit code 200
     ——▶ justfile:2:24
      │
    2 │     echo             😬鎌鼬{{        `exit 200 #             abc`}}            😬鎌鼬
      │                                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn backtick_code_long() {
  Test::new()
    .justfile(
      "






    b := a
    a := `echo hello`
    bar:
     echo '{{`exit 200`}}'
  ",
    )
    .stderr(
      "
    error: Backtick failed with exit code 200
      ——▶ justfile:10:10
       │
    10 │  echo '{{`exit 200`}}'
       │          ^^^^^^^^^^
  ",
    )
    .status(200);
}

#[test]
fn shebang_backtick_failure() {
  Test::new()
    .justfile(
      "foo:
 #!/bin/sh
 echo hello
 echo {{`exit 123`}}",
    )
    .stderr(
      "
    error: Backtick failed with exit code 123
     ——▶ justfile:4:9
      │
    4 │  echo {{`exit 123`}}
      │         ^^^^^^^^^^
  ",
    )
    .status(123);
}

#[test]
fn command_backtick_failure() {
  Test::new()
    .justfile(
      "foo:
 echo hello
 echo {{`exit 123`}}",
    )
    .stdout("hello\n")
    .stderr(
      "
    echo hello
    error: Backtick failed with exit code 123
     ——▶ justfile:3:9
      │
    3 │  echo {{`exit 123`}}
      │         ^^^^^^^^^^
  ",
    )
    .status(123);
}

#[test]
fn assignment_backtick_failure() {
  Test::new()
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr(
      "
    error: Backtick failed with exit code 222
     ——▶ justfile:4:6
      │
    4 │ a := `exit 222`
      │      ^^^^^^^^^^
  ",
    )
    .status(222);
}

#[test]
fn unknown_override_options() {
  Test::new()
    .arg("--set")
    .arg("foo")
    .arg("bar")
    .arg("--set")
    .arg("baz")
    .arg("bob")
    .arg("--set")
    .arg("a")
    .arg("b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr(
      "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
    )
    .failure();
}

#[test]
fn unknown_override_args() {
  Test::new()
    .arg("foo=bar")
    .arg("baz=bob")
    .arg("a=b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr(
      "error: Variables `baz` and `foo` overridden on the command line but not present \
    in justfile\n",
    )
    .failure();
}

#[test]
fn unknown_override_arg() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("a")
    .arg("b")
    .justfile(
      "foo:
 echo hello
 echo {{`exit 111`}}
a := `exit 222`",
    )
    .stderr("error: Variable `foo` overridden on the command line but not present in justfile\n")
    .failure();
}

#[test]
fn overrides_first() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("recipe")
    .arg("baz=bar")
    .justfile(
      r#"
foo := "foo"
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    )
    .stdout("arg=baz=bar\nbarbbaz\n")
    .stderr("echo arg=baz=bar\necho barbbaz\n")
    .success();
}

#[test]
fn overrides_not_evaluated() {
  Test::new()
    .arg("foo=bar")
    .arg("a=b")
    .arg("recipe")
    .arg("baz=bar")
    .justfile(
      r#"
foo := `exit 1`
a := "a"
baz := "baz"

recipe arg:
 echo arg={{arg}}
 echo {{foo + a + baz}}"#,
    )
    .stdout("arg=baz=bar\nbarbbaz\n")
    .stderr("echo arg=baz=bar\necho barbbaz\n")
    .success();
}

#[test]
fn dry_run() {
  Test::new()
    .arg("--dry-run")
    .arg("shebang")
    .arg("command")
    .justfile(
      r"
var := `echo stderr 1>&2; echo backtick`

command:
  @touch /this/is/not/a/file
  {{var}}
  echo {{`echo command interpolation`}}

shebang:
  #!/bin/sh
  touch /this/is/not/a/file
  {{var}}
  echo {{`echo shebang interpolation`}}",
    )
    .stderr(
      "#!/bin/sh
touch /this/is/not/a/file
`echo stderr 1>&2; echo backtick`
echo `echo shebang interpolation`
touch /this/is/not/a/file
`echo stderr 1>&2; echo backtick`
echo `echo command interpolation`
",
    )
    .success();
}

#[test]
fn line_error_spacing() {
  Test::new()
    .justfile(
      r"









^^^
",
    )
    .stderr(
      "error: Unknown start of token '^'
  ——▶ justfile:10:1
   │
10 │ ^^^
   │ ^
",
    )
    .failure();
}

#[test]
fn argument_single() {
  Test::new()
    .arg("foo")
    .arg("ARGUMENT")
    .justfile(
      "
foo A:
  echo {{A}}
    ",
    )
    .stdout("ARGUMENT\n")
    .stderr("echo ARGUMENT\n")
    .success();
}

#[test]
fn argument_multiple() {
  Test::new()
    .arg("foo")
    .arg("ONE")
    .arg("TWO")
    .justfile(
      "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    )
    .stdout("A:ONE B:TWO\n")
    .stderr("echo A:ONE B:TWO\n")
    .success();
}

#[test]
fn argument_mismatch_more() {
  Test::new()
    .arg("foo")
    .arg("ONE")
    .arg("TWO")
    .arg("THREE")
    .stderr("error: Justfile does not contain recipe `THREE`\n")
    .justfile(
      "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    )
    .failure();
}

#[test]
fn argument_mismatch_fewer() {
  Test::new()
    .arg("foo")
    .arg("ONE")
    .justfile(
      "
foo A B:
  echo A:{{A}} B:{{B}}
    ",
    )
    .stderr("error: Recipe `foo` got 1 positional argument but takes 2\nusage:\n    just foo A B\n")
    .failure();
}

#[test]
fn argument_mismatch_more_with_default() {
  Test::new()
    .arg("foo")
    .arg("ONE")
    .arg("TWO")
    .arg("THREE")
    .justfile(
      "
foo A B='B':
  echo A:{{A}} B:{{B}}
    ",
    )
    .stderr("error: Justfile does not contain recipe `THREE`\n")
    .failure();
}

#[test]
fn argument_mismatch_fewer_with_default() {
  Test::new()
    .arg("foo")
    .arg("bar")
    .justfile(
      "
foo A B C='C':
  echo A:{{A}} B:{{B}} C:{{C}}
    ",
    )
    .stderr(
      "
    error: Recipe `foo` got 1 positional argument but takes at least 2
    usage:
        just foo A B [C]
  ",
    )
    .failure();
}

#[test]
fn unknown_recipe() {
  Test::new()
    .arg("foo")
    .justfile("hello:")
    .stderr("error: Justfile does not contain recipe `foo`\n")
    .failure();
}

#[test]
fn unknown_recipes() {
  Test::new()
    .arg("foo")
    .arg("bar")
    .justfile("hello:")
    .stderr("error: Justfile does not contain recipe `foo`\n")
    .failure();
}

#[test]
fn color_always() {
  Test::new()
        .arg("--color")
        .arg("always")
        .justfile("b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'")
        .stderr("\u{1b}[1;31merror\u{1b}[0m: \u{1b}[1mBacktick failed with exit code 100\u{1b}[0m\n \u{1b}[1;34m——▶\u{1b}[0m justfile:2:6\n  \u{1b}[1;34m│\u{1b}[0m\n\u{1b}[1;34m2 │\u{1b}[0m a := `exit 100`\n  \u{1b}[1;34m│\u{1b}[0m      \u{1b}[1;31m^^^^^^^^^^\u{1b}[0m\n")
        .status(100);
}

#[test]
fn color_never() {
  Test::new()
    .arg("--color")
    .arg("never")
    .justfile("b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'")
    .stderr(
      "error: Backtick failed with exit code 100
 ——▶ justfile:2:6
  │
2 │ a := `exit 100`
  │      ^^^^^^^^^^
",
    )
    .status(100);
}

#[test]
fn color_auto() {
  Test::new()
    .arg("--color")
    .arg("auto")
    .justfile("b := a\na := `exit 100`\nbar:\n echo '{{`exit 200`}}'")
    .stderr(
      "error: Backtick failed with exit code 100
 ——▶ justfile:2:6
  │
2 │ a := `exit 100`
  │      ^^^^^^^^^^
",
    )
    .status(100);
}

#[test]
fn colors_no_context() {
  Test::new()
    .arg("--color=always")
    .stderr(
      "\u{1b}[1;31merror\u{1b}[0m: \u{1b}[1m\
Recipe `recipe` failed on line 2 with exit code 100\u{1b}[0m\n",
    )
    .justfile(
      "
recipe:
  @exit 100",
    )
    .status(100);
}

#[test]
fn dump() {
  Test::new()
    .arg("--dump")
    .justfile(
      r"
# this recipe does something
recipe a b +d:
 @exit 100",
    )
    .stdout(
      "# this recipe does something
recipe a b +d:
    @exit 100
",
    )
    .success();
}

#[test]
fn mixed_whitespace() {
  Test::new()
    .justfile("bar:\n\t echo hello")
    .stderr(
      "error: Found a mix of tabs and spaces in leading whitespace: `␉␠`
Leading whitespace may consist of tabs or spaces, but not both
 ——▶ justfile:2:1
  │
2 │      echo hello
  │ ^^^^^
",
    )
    .failure();
}

#[test]
fn extra_leading_whitespace() {
  Test::new()
    .justfile("bar:\n\t\techo hello\n\t\t\techo goodbye")
    .stderr(
      "error: Recipe line has extra leading whitespace
 ——▶ justfile:3:3
  │
3 │             echo goodbye
  │         ^^^^^^^^^^^^^^^^
",
    )
    .failure();
}

#[test]
fn inconsistent_leading_whitespace() {
  Test::new()
    .justfile("bar:\n\t\techo hello\n\t echo goodbye")
    .stderr(
      "error: Recipe line has inconsistent leading whitespace. \
            Recipe started with `␉␉` but found line with `␉␠`
 ——▶ justfile:3:1
  │
3 │      echo goodbye
  │ ^^^^^
",
    )
    .failure();
}

#[test]
fn required_after_default() {
  Test::new()
    .justfile("bar:\nhello baz arg='foo' bar:")
    .stderr(
      "error: Non-default parameter `bar` follows default parameter
 ——▶ justfile:2:21
  │
2 │ hello baz arg='foo' bar:
  │                     ^^^
",
    )
    .failure();
}

#[test]
fn required_after_plus_variadic() {
  Test::new()
    .justfile("bar:\nhello baz +arg bar:")
    .stderr(
      "error: Parameter `bar` follows variadic parameter
 ——▶ justfile:2:16
  │
2 │ hello baz +arg bar:
  │                ^^^
",
    )
    .failure();
}

#[test]
fn required_after_star_variadic() {
  Test::new()
    .justfile("bar:\nhello baz *arg bar:")
    .stderr(
      "error: Parameter `bar` follows variadic parameter
 ——▶ justfile:2:16
  │
2 │ hello baz *arg bar:
  │                ^^^
",
    )
    .failure();
}

#[test]
fn use_string_default() {
  Test::new()
    .arg("hello")
    .arg("ABC")
    .justfile(
      r#"
bar:
hello baz arg="XYZ\t\"	":
  echo '{{baz}}...{{arg}}'
"#,
    )
    .stdout("ABC...XYZ\t\"\t\n")
    .stderr("echo 'ABC...XYZ\t\"\t'\n")
    .success();
}

#[test]
fn use_raw_string_default() {
  Test::new()
    .arg("hello")
    .arg("ABC")
    .justfile(
      r#"
bar:
hello baz arg='XYZ"	':
  printf '{{baz}}...{{arg}}'
"#,
    )
    .stdout("ABC...XYZ\"\t")
    .stderr("printf 'ABC...XYZ\"\t'\n")
    .success();
}

#[test]
fn supply_use_default() {
  Test::new()
    .arg("hello")
    .arg("0")
    .arg("1")
    .justfile(
      r"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
",
    )
    .stdout("0 1 C\n")
    .stderr("echo 0 1 C\n")
    .success();
}

#[test]
fn supply_defaults() {
  Test::new()
    .arg("hello")
    .arg("0")
    .arg("1")
    .arg("2")
    .justfile(
      r"
hello a b='B' c='C':
  echo {{a}} {{b}} {{c}}
",
    )
    .stdout("0 1 2\n")
    .stderr("echo 0 1 2\n")
    .success();
}

#[test]
fn list() {
  Test::new()
    .arg("--list")
    .justfile(
      r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this comment will be ignored

a Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
    )
    .stdout(
      r#"
    Available recipes:
        a Z="\t z"
        hello a b='B	' c='C' # this does a thing
  "#,
    )
    .success();
}

#[test]
fn list_alignment() {
  Test::new()
    .arg("--list")
    .justfile(
      r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# something else
a Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
    )
    .stdout(
      r#"
    Available recipes:
        a Z="\t z"           # something else
        hello a b='B	' c='C' # this does a thing
  "#,
    )
    .success();
}

#[test]
fn list_alignment_long() {
  Test::new()
    .arg("--list")
    .justfile(
      r#"

# this does a thing
hello a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# this does another thing
x a b='B	' c='C':
  echo {{a}} {{b}} {{c}}

# something else
this-recipe-is-very-very-very-very-very-very-very-very-important Z="\t z":

# this recipe will not appear
_private-recipe:
"#,
    )
    .stdout(
      r#"
    Available recipes:
        hello a b='B	' c='C' # this does a thing
        this-recipe-is-very-very-very-very-very-very-very-very-important Z="\t z" # something else
        x a b='B	' c='C'     # this does another thing
  "#,
    )
    .success();
}

#[test]
fn list_sorted() {
  Test::new()
    .arg("--list")
    .justfile(
      r"
alias c := b
b:
a:
",
    )
    .stdout(
      r"
    Available recipes:
        a
        b # [alias: c]
  ",
    )
    .success();
}

#[test]
fn list_unsorted() {
  Test::new()
    .arg("--list")
    .arg("--unsorted")
    .justfile(
      r"
alias c := b
b:
a:
",
    )
    .stdout(
      r"
    Available recipes:
        b # [alias: c]
        a
  ",
    )
    .success();
}

#[test]
fn list_heading() {
  Test::new()
    .arg("--list")
    .arg("--list-heading")
    .arg("Cool stuff…\n")
    .justfile(
      r"
a:
b:
",
    )
    .stdout(
      r"
    Cool stuff…
        a
        b
  ",
    )
    .success();
}

#[test]
fn list_prefix() {
  Test::new()
    .arg("--list")
    .arg("--list-prefix")
    .arg("····")
    .justfile(
      r"
a:
b:
",
    )
    .stdout(
      r"
    Available recipes:
    ····a
    ····b
  ",
    )
    .success();
}

#[test]
fn list_empty_prefix_and_heading() {
  Test::new()
    .arg("--list")
    .arg("--list-heading")
    .arg("")
    .arg("--list-prefix")
    .arg("")
    .justfile(
      r"
a:
b:
",
    )
    .stdout(
      r"
    a
    b
  ",
    )
    .success();
}

#[test]
fn run_suggestion() {
  Test::new()
    .arg("hell")
    .justfile("hello:")
    .stderr("error: Justfile does not contain recipe `hell`\nDid you mean `hello`?\n")
    .failure();
}

#[test]
fn private_recipes_are_not_suggested() {
  Test::new()
    .arg("hell")
    .justfile(
      "
        [private]
        hello:
      ",
    )
    .stderr("error: Justfile does not contain recipe `hell`\n")
    .failure();
}

#[test]
fn alias_suggestion() {
  Test::new()
    .arg("hell")
    .justfile(
      "
        alias hello := bar

        bar:
      ",
    )
    .stderr(
      "error: Justfile does not contain recipe `hell`\nDid you mean `hello`, an alias for `bar`?\n",
    )
    .failure();
}

#[test]
fn private_aliases_are_not_suggested() {
  Test::new()
    .arg("hell")
    .justfile(
      "
        [private]
        alias hello := bar

        bar:
      ",
    )
    .stderr("error: Justfile does not contain recipe `hell`\n")
    .failure();
}

#[test]
fn line_continuation_with_space() {
  Test::new()
    .justfile(
      r"
foo:
  echo a\
         b  \
             c
",
    )
    .stdout("ab c\n")
    .stderr("echo ab  c\n")
    .success();
}

#[test]
fn line_continuation_with_quoted_space() {
  Test::new()
    .justfile(
      r"
foo:
  echo 'a\
         b  \
             c'
",
    )
    .stdout("ab  c\n")
    .stderr("echo 'ab  c'\n")
    .success();
}

#[test]
fn line_continuation_no_space() {
  Test::new()
    .justfile(
      r"
foo:
  echo a\
  b\
  c
",
    )
    .stdout("abc\n")
    .stderr("echo abc\n")
    .success();
}

#[test]
fn infallible_command() {
  Test::new()
    .justfile(
      r"
infallible:
  -exit 101
",
    )
    .stderr("exit 101\n")
    .success();
}

#[test]
fn infallible_with_failing() {
  Test::new()
    .justfile(
      r"
infallible:
  -exit 101
  exit 202
",
    )
    .stderr(
      r"exit 101
exit 202
error: Recipe `infallible` failed on line 3 with exit code 202
",
    )
    .status(202);
}

#[test]
fn quiet_recipe() {
  Test::new()
    .justfile(
      r"
@quiet:
  # a
  # b
  @echo c
",
    )
    .stdout("c\n")
    .stderr("echo c\n")
    .success();
}

#[test]
fn quiet_shebang_recipe() {
  Test::new()
    .justfile(
      r"
@quiet:
  #!/bin/sh
  echo hello
",
    )
    .stdout("hello\n")
    .stderr("#!/bin/sh\necho hello\n")
    .success();
}

#[test]
fn complex_dependencies() {
  Test::new()
    .arg("b")
    .justfile(
      r"
a: b
b:
c: b a
",
    )
    .success();
}

#[test]
fn unknown_function_in_assignment() {
  Test::new()
    .arg("bar")
    .justfile(
      r#"foo := foo() + "hello"
bar:"#,
    )
    .stderr(
      r#"error: Call to unknown function `foo`
 ——▶ justfile:1:8
  │
1 │ foo := foo() + "hello"
  │        ^^^
"#,
    )
    .failure();
}

#[test]
fn dependency_takes_arguments_exact() {
  Test::new()
    .arg("b")
    .justfile(
      "
    a FOO:
    b: a
  ",
    )
    .stderr(
      "error: Dependency `a` got 0 arguments but takes 1 argument
 ——▶ justfile:2:4
  │
2 │ b: a
  │    ^
",
    )
    .failure();
}

#[test]
fn dependency_takes_arguments_at_least() {
  Test::new()
    .arg("b")
    .justfile(
      "
    a FOO LUZ='hello':
    b: a
  ",
    )
    .stderr(
      "error: Dependency `a` got 0 arguments but takes at least 1 argument
 ——▶ justfile:2:4
  │
2 │ b: a
  │    ^
",
    )
    .failure();
}

#[test]
fn dependency_takes_arguments_at_most() {
  Test::new()
    .arg("b")
    .justfile(
      "
    a FOO LUZ='hello':
    b: (a '0' '1' '2')
  ",
    )
    .stderr(
      "error: Dependency `a` got 3 arguments but takes at most 2 arguments
 ——▶ justfile:2:5
  │
2 │ b: (a '0' '1' '2')
  │     ^
",
    )
    .failure();
}

#[test]
fn duplicate_parameter() {
  Test::new()
    .arg("a")
    .justfile("a foo foo:")
    .stderr(
      "error: Recipe `a` has duplicate parameter `foo`
 ——▶ justfile:1:7
  │
1 │ a foo foo:
  │       ^^^
",
    )
    .failure();
}

#[test]
fn duplicate_recipe() {
  Test::new()
    .arg("b")
    .justfile("b:\nb:")
    .stderr(
      "error: Recipe `b` first defined on line 1 is redefined on line 2
 ——▶ justfile:2:1
  │
2 │ b:
  │ ^
",
    )
    .failure();
}

#[test]
fn duplicate_variable() {
  Test::new()
    .arg("foo")
    .justfile("a := 'hello'\na := 'hello'\nfoo:")
    .stderr(
      "error: Variable `a` has multiple definitions
 ——▶ justfile:2:1
  │
2 │ a := 'hello'
  │ ^
",
    )
    .failure();
}

#[test]
fn unexpected_token_in_dependency_position() {
  Test::new()
    .arg("foo")
    .justfile("foo: 'bar'")
    .stderr(
      "error: Expected '&&', comment, end of file, end of line, \
    identifier, or '(', but found string
 ——▶ justfile:1:6
  │
1 │ foo: 'bar'
  │      ^^^^^
",
    )
    .failure();
}

#[test]
fn unexpected_token_after_name() {
  Test::new()
    .arg("foo")
    .justfile("foo 'bar'")
    .stderr(
      "error: Expected '*', ':', '$', identifier, or '+', but found string
 ——▶ justfile:1:5
  │
1 │ foo 'bar'
  │     ^^^^^
",
    )
    .failure();
}

#[test]
fn self_dependency() {
  Test::new()
    .arg("a")
    .justfile("a: a")
    .stderr(
      "error: Recipe `a` depends on itself
 ——▶ justfile:1:4
  │
1 │ a: a
  │    ^
",
    )
    .failure();
}

#[test]
fn long_circular_recipe_dependency() {
  Test::new()
    .arg("a")
    .justfile("a: b\nb: c\nc: d\nd: a")
    .stderr(
      "error: Recipe `d` has circular dependency `a -> b -> c -> d -> a`
 ——▶ justfile:4:4
  │
4 │ d: a
  │    ^
",
    )
    .failure();
}

#[test]
fn variable_self_dependency() {
  Test::new()
    .arg("a")
    .justfile("z := z\na:")
    .stderr(
      "error: Variable `z` is defined in terms of itself
 ——▶ justfile:1:1
  │
1 │ z := z
  │ ^
",
    )
    .failure();
}

#[test]
fn variable_circular_dependency() {
  Test::new()
    .arg("a")
    .justfile("x := y\ny := z\nz := x\na:")
    .stderr(
      "error: Variable `x` depends on its own value: `x -> y -> z -> x`
 ——▶ justfile:1:1
  │
1 │ x := y
  │ ^
",
    )
    .failure();
}

#[test]
fn variable_circular_dependency_with_additional_variable() {
  Test::new()
    .arg("a")
    .justfile(
      "
    a := ''
    x := y
    y := x

    a:
  ",
    )
    .stderr(
      "error: Variable `x` depends on its own value: `x -> y -> x`
 ——▶ justfile:2:1
  │
2 │ x := y
  │ ^
",
    )
    .failure();
}

#[test]
fn plus_variadic_recipe() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .arg("2")
    .arg("3")
    .arg(" 4 ")
    .justfile(
      "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 2 3 4\n")
    .stderr("echo 0 1 2 3  4 \n")
    .success();
}

#[test]
fn plus_variadic_ignore_default() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .arg("2")
    .arg("3")
    .arg(" 4 ")
    .justfile(
      "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 2 3 4\n")
    .stderr("echo 0 1 2 3  4 \n")
    .success();
}

#[test]
fn plus_variadic_use_default() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .justfile(
      "
a x y +z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 HELLO\n")
    .stderr("echo 0 1 HELLO\n")
    .success();
}

#[test]
fn plus_variadic_too_few() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .justfile(
      "
a x y +z:
  echo {{x}} {{y}} {{z}}
",
    )
    .stderr(
      "
        error: Recipe `a` got 2 positional arguments but takes at least 3
        usage:
            just a x y z...
      ",
    )
    .failure();
}

#[test]
fn star_variadic_recipe() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .arg("2")
    .arg("3")
    .arg(" 4 ")
    .justfile(
      "
a x y *z:
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 2 3 4\n")
    .stderr("echo 0 1 2 3  4 \n")
    .success();
}

#[test]
fn star_variadic_none() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .justfile(
      "
a x y *z:
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1\n")
    .stderr("echo 0 1 \n")
    .success();
}

#[test]
fn star_variadic_ignore_default() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .arg("2")
    .arg("3")
    .arg(" 4 ")
    .justfile(
      "
a x y *z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 2 3 4\n")
    .stderr("echo 0 1 2 3  4 \n")
    .success();
}

#[test]
fn star_variadic_use_default() {
  Test::new()
    .arg("a")
    .arg("0")
    .arg("1")
    .justfile(
      "
a x y *z='HELLO':
  echo {{x}} {{y}} {{z}}
",
    )
    .stdout("0 1 HELLO\n")
    .stderr("echo 0 1 HELLO\n")
    .success();
}

#[test]
fn star_then_plus_variadic() {
  Test::new()
    .justfile(
      "
foo *a +b:
  echo {{a}} {{b}}
",
    )
    .stderr(
      "error: Expected \':\' or \'=\', but found \'+\'
 ——▶ justfile:1:8
  │
1 │ foo *a +b:
  │        ^
",
    )
    .failure();
}

#[test]
fn plus_then_star_variadic() {
  Test::new()
    .justfile(
      "
foo +a *b:
  echo {{a}} {{b}}
",
    )
    .stderr(
      "error: Expected \':\' or \'=\', but found \'*\'
 ——▶ justfile:1:8
  │
1 │ foo +a *b:
  │        ^
",
    )
    .failure();
}

#[test]
fn argument_grouping() {
  Test::new()
    .arg("BAR")
    .arg("0")
    .arg("FOO")
    .arg("1")
    .arg("2")
    .arg("BAZ")
    .arg("3")
    .arg("4")
    .arg("5")
    .justfile(
      "
FOO A B='blarg':
  echo foo: {{A}} {{B}}

BAR X:
  echo bar: {{X}}

BAZ +Z:
  echo baz: {{Z}}
",
    )
    .stdout("bar: 0\nfoo: 1 2\nbaz: 3 4 5\n")
    .stderr("echo bar: 0\necho foo: 1 2\necho baz: 3 4 5\n")
    .success();
}

#[test]
fn missing_second_dependency() {
  Test::new()
    .justfile(
      "
x:

a: x y
",
    )
    .stderr(
      "error: Recipe `a` has unknown dependency `y`
 ——▶ justfile:3:6
  │
3 │ a: x y
  │      ^
",
    )
    .failure();
}

#[test]
fn list_colors() {
  Test::new()
    .arg("--color")
    .arg("always")
    .arg("--list")
    .justfile(
      "
# comment
a B C +D='hello':
  echo {{B}} {{C}} {{D}}
",
    )
    .stdout(
      "
    Available recipes:
        a \
    \u{1b}[36mB\u{1b}[0m \u{1b}[36mC\u{1b}[0m \u{1b}[35m+\
    \u{1b}[0m\u{1b}[36mD\u{1b}[0m=\u{1b}[32m'hello'\u{1b}[0m \
     \u{1b}[34m#\u{1b}[0m \u{1b}[34mcomment\u{1b}[0m
  ",
    )
    .success();
}

#[test]
fn run_colors() {
  Test::new()
    .arg("--color")
    .arg("always")
    .arg("--highlight")
    .arg("--verbose")
    .justfile(
      "
# comment
a:
  echo hi
",
    )
    .stdout("hi\n")
    .stderr("\u{1b}[1;36m===> Running recipe `a`...\u{1b}[0m\n\u{1b}[1mecho hi\u{1b}[0m\n")
    .success();
}

#[test]
fn no_highlight() {
  Test::new()
    .arg("--color")
    .arg("always")
    .arg("--highlight")
    .arg("--no-highlight")
    .arg("--verbose")
    .justfile(
      "
# comment
a:
  echo hi
",
    )
    .stdout("hi\n")
    .stderr("\u{1b}[1;36m===> Running recipe `a`...\u{1b}[0m\necho hi\n")
    .success();
}

#[test]
fn trailing_flags() {
  Test::new()
    .arg("echo")
    .arg("--some")
    .arg("--awesome")
    .arg("--flags")
    .justfile(
      "
echo A B C:
  echo {{A}} {{B}} {{C}}
",
    )
    .stdout("--some --awesome --flags\n")
    .stderr("echo --some --awesome --flags\n")
    .success();
}

#[test]
fn comment_before_variable() {
  Test::new()
    .arg("echo")
    .justfile(
      "
#
A:='1'
echo:
  echo {{A}}
 ",
    )
    .stdout("1\n")
    .stderr("echo 1\n")
    .success();
}

#[test]
fn invalid_escape_sequence_message() {
  Test::new()
    .justfile(
      r#"
X := "\'"
"#,
    )
    .stderr(
      r#"error: `\'` is not a valid escape sequence
 ——▶ justfile:1:6
  │
1 │ X := "\'"
  │      ^^^^
"#,
    )
    .failure();
}

#[test]
fn unknown_variable_in_default() {
  Test::new()
    .justfile(
      "
     foo x=bar:
   ",
    )
    .stderr(
      r"error: Variable `bar` not defined
 ——▶ justfile:1:7
  │
1 │ foo x=bar:
  │       ^^^
",
    )
    .failure();
}

#[test]
fn unknown_function_in_default() {
  Test::new()
    .justfile(
      "
foo x=bar():
",
    )
    .stderr(
      r"error: Call to unknown function `bar`
 ——▶ justfile:1:7
  │
1 │ foo x=bar():
  │       ^^^
",
    )
    .failure();
}

#[test]
fn default_string() {
  Test::new()
    .justfile(
      "
foo x='bar':
  echo {{x}}
",
    )
    .stdout("bar\n")
    .stderr("echo bar\n")
    .success();
}

#[test]
fn default_concatenation() {
  Test::new()
    .justfile(
      "
foo x=(`echo foo` + 'bar'):
  echo {{x}}
",
    )
    .stdout("foobar\n")
    .stderr("echo foobar\n")
    .success();
}

#[test]
fn default_backtick() {
  Test::new()
    .justfile(
      "
foo x=`echo foo`:
  echo {{x}}
",
    )
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn default_variable() {
  Test::new()
    .justfile(
      "
y := 'foo'
foo x=y:
  echo {{x}}
",
    )
    .stdout("foo\n")
    .stderr("echo foo\n")
    .success();
}

#[test]
fn unterminated_interpolation_eol() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{
  ",
    )
    .stderr(
      r"
    error: Unterminated interpolation
     ——▶ justfile:2:8
      │
    2 │   echo {{
      │        ^^
  ",
    )
    .failure();
}

#[test]
fn unterminated_interpolation_eof() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{
  ",
    )
    .stderr(
      r"
    error: Unterminated interpolation
     ——▶ justfile:2:8
      │
    2 │   echo {{
      │        ^^
  ",
    )
    .failure();
}

#[test]
fn unknown_start_of_token() {
  Test::new()
    .justfile(
      "
assembly_source_files = %(wildcard src/arch/$(arch)/*.s)
      ",
    )
    .stderr(
      r"
    error: Unknown start of token '%'
     ——▶ justfile:1:25
      │
    1 │ assembly_source_files = %(wildcard src/arch/$(arch)/*.s)
      │                         ^
  ",
    )
    .failure();
}

#[test]
fn unknown_start_of_token_invisible_unicode() {
  Test::new()
    .justfile(
      "
\u{200b}foo := 'bar'
      ",
    )
    .stderr(
      "
error: Unknown start of token '\u{200b}' (U+200B)
 ——▶ justfile:1:1
  │
1 │ \u{200b}foo := 'bar'
  │ ^
",
    )
    .failure();
}

#[test]
fn unknown_start_of_token_ascii_control_char() {
  Test::new()
    .justfile(
      "
\0foo := 'bar'
",
    )
    .stderr(
      "
error: Unknown start of token '\0' (U+0000)
 ——▶ justfile:1:1
  │
1 │ \0foo := 'bar'
  │ ^
",
    )
    .failure();
}

#[test]
fn backtick_variable_cat() {
  Test::new()
    .justfile(
      "
stdin := `cat`

default:
  echo {{stdin}}
",
    )
    .stdin("STDIN")
    .stdout("STDIN\n")
    .stderr("echo STDIN\n")
    .success();
}

#[test]
fn backtick_default_cat_stdin() {
  Test::new()
    .justfile(
      "
default stdin = `cat`:
  echo {{stdin}}
",
    )
    .stdin("STDIN")
    .stdout("STDIN\n")
    .stderr("echo STDIN\n")
    .success();
}

#[test]
fn backtick_default_cat_justfile() {
  Test::new()
    .justfile(
      "
    default stdin = `cat justfile`:
      echo '{{stdin}}'
  ",
    )
    .stdout(
      "
    default stdin = `cat justfile`:
      echo {{stdin}}
  ",
    )
    .stderr(
      "
    echo 'default stdin = `cat justfile`:
      echo '{{stdin}}''
  ",
    )
    .success();
}

#[test]
fn backtick_variable_read_single() {
  Test::new()
    .justfile(
      "
password := `read PW && echo $PW`

default:
  echo {{password}}
",
    )
    .stdin("foobar\n")
    .stdout("foobar\n")
    .stderr("echo foobar\n")
    .success();
}

#[test]
fn backtick_variable_read_multiple() {
  Test::new()
    .justfile(
      "
a := `read A && echo $A`
b := `read B && echo $B`

default:
  echo {{a}}
  echo {{b}}
",
    )
    .stdin("foo\nbar\n")
    .stdout("foo\nbar\n")
    .stderr("echo foo\necho bar\n")
    .success();
}

#[test]
fn backtick_default_read_multiple() {
  Test::new()
    .justfile(
      "

default a=`read A && echo $A` b=`read B && echo $B`:
  echo {{a}}
  echo {{b}}
",
    )
    .stdin("foo\nbar\n")
    .stdout("foo\nbar\n")
    .stderr("echo foo\necho bar\n")
    .success();
}

#[test]
fn old_equals_assignment_syntax_produces_error() {
  Test::new()
    .justfile(
      "
    foo = 'bar'

    default:
      echo {{foo}}
  ",
    )
    .stderr(
      "
    error: Expected '*', ':', '$', identifier, or '+', but found '='
     ——▶ justfile:1:5
      │
    1 │ foo = 'bar'
      │     ^
    ",
    )
    .failure();
}

#[test]
fn dependency_argument_string() {
  Test::new()
    .justfile(
      "
    release: (build 'foo') (build 'bar')

    build target:
      echo 'Building {{target}}...'
  ",
    )
    .stdout("Building foo...\nBuilding bar...\n")
    .stderr("echo 'Building foo...'\necho 'Building bar...'\n")
    .shell(false)
    .success();
}

#[test]
fn dependency_argument_parameter() {
  Test::new()
    .justfile(
      "
    default: (release '1.0')

    release version: (build 'foo' version) (build 'bar' version)

    build target version:
      echo 'Building {{target}}@{{version}}...'
  ",
    )
    .stdout("Building foo@1.0...\nBuilding bar@1.0...\n")
    .stderr("echo 'Building foo@1.0...'\necho 'Building bar@1.0...'\n")
    .shell(false)
    .success();
}

#[test]
fn dependency_argument_function() {
  Test::new()
    .justfile(
      "
    foo: (bar env_var_or_default('x', 'y'))

    bar arg:
      echo {{arg}}
  ",
    )
    .stdout("y\n")
    .stderr("echo y\n")
    .shell(false)
    .success();
}

#[test]
fn env_function_as_env_var() {
  Test::new()
    .env("x", "z")
    .justfile(
      "
    foo: (bar env('x'))

    bar arg:
      echo {{arg}}
  ",
    )
    .stdout("z\n")
    .stderr("echo z\n")
    .shell(false)
    .success();
}

#[test]
fn env_function_as_env_var_or_default() {
  Test::new()
    .env("x", "z")
    .justfile(
      "
    foo: (bar env('x', 'y'))

    bar arg:
      echo {{arg}}
  ",
    )
    .stdout("z\n")
    .stderr("echo z\n")
    .shell(false)
    .success();
}

#[test]
fn env_function_as_env_var_with_existing_env_var() {
  Test::new()
    .env("x", "z")
    .justfile(
      "
    foo: (bar env('x'))

    bar arg:
      echo {{arg}}
  ",
    )
    .stdout("z\n")
    .stderr("echo z\n")
    .shell(false)
    .success();
}

#[test]
fn env_function_as_env_var_or_default_with_existing_env_var() {
  Test::new()
    .env("x", "z")
    .justfile(
      "
    foo: (bar env('x', 'y'))

    bar arg:
      echo {{arg}}
  ",
    )
    .stdout("z\n")
    .stderr("echo z\n")
    .shell(false)
    .success();
}

#[test]
fn dependency_argument_backtick() {
  Test::new()
    .justfile(
      "
    export X := 'X'

    foo: (bar `echo $X`)

    bar arg:
      echo {{arg}}
      echo $X
  ",
    )
    .stdout("X\nX\n")
    .stderr("echo X\necho $X\n")
    .shell(false)
    .success();
}

#[test]
fn dependency_argument_assignment() {
  Test::new()
    .justfile(
      "
    v := '1.0'

    default: (release v)

    release version:
      echo Release {{version}}...
  ",
    )
    .stdout("Release 1.0...\n")
    .stderr("echo Release 1.0...\n")
    .shell(false)
    .success();
}

#[test]
fn dependency_argument_plus_variadic() {
  Test::new()
    .justfile(
      "
    foo: (bar 'A' 'B' 'C')

    bar +args:
      echo {{args}}
  ",
    )
    .stdout("A B C\n")
    .stderr("echo A B C\n")
    .shell(false)
    .success();
}

#[test]
fn duplicate_dependency_no_args() {
  Test::new()
    .justfile(
      "
    foo: bar bar bar bar

    bar:
      echo BAR
  ",
    )
    .stdout("BAR\n")
    .stderr("echo BAR\n")
    .shell(false)
    .success();
}

#[test]
fn duplicate_dependency_argument() {
  Test::new()
    .justfile(
      "
    foo: (bar 'BAR') (bar `echo BAR`)

    bar bar:
      echo {{bar}}
  ",
    )
    .stdout("BAR\n")
    .stderr("echo BAR\n")
    .shell(false)
    .success();
}

#[test]
fn pwsh_invocation_directory() {
  if cfg!(not(windows)) {
    return;
  }
  Test::new()
    .justfile(
      r#"
    set shell := ["pwsh", "-NoProfile", "-c"]

    pwd:
      @Test-Path {{invocation_directory()}} > result.txt
  "#,
    )
    .shell(false)
    .success();
}

#[test]
fn variables() {
  Test::new()
    .arg("--variables")
    .justfile(
      "
    z := 'a'
    a := 'z'
  ",
    )
    .stdout("a z\n")
    .shell(false)
    .success();
}

#[test]
fn interpolation_evaluation_ignore_quiet() {
  Test::new()
    .justfile(
      r#"
    foo:
      {{"@echo foo 2>/dev/null"}}
  "#,
    )
    .stderr(
      "
    @echo foo 2>/dev/null
    error: Recipe `foo` failed on line 2 with exit code 127
  ",
    )
    .shell(false)
    .status(127);
}

#[test]
fn interpolation_evaluation_ignore_quiet_continuation() {
  Test::new()
    .justfile(
      r#"
    foo:
      {{""}}\
      @echo foo 2>/dev/null
  "#,
    )
    .stderr(
      "
    @echo foo 2>/dev/null
    error: Recipe `foo` failed on line 3 with exit code 127
  ",
    )
    .shell(false)
    .status(127);
}

#[test]
fn brace_escape() {
  Test::new()
    .justfile(
      "
    foo:
      echo '{{{{'
  ",
    )
    .stdout("{{\n")
    .stderr(
      "
    echo '{{'
  ",
    )
    .success();
}

#[test]
fn brace_escape_extra() {
  Test::new()
    .justfile(
      "
    foo:
      echo '{{{{{'
  ",
    )
    .stdout("{{{\n")
    .stderr(
      "
    echo '{{{'
  ",
    )
    .success();
}

#[test]
fn multi_line_string_in_interpolation() {
  Test::new()
    .justfile(
      "
    foo:
      echo {{'a
      echo b
      echo c'}}z
      echo baz
  ",
    )
    .stdout("a\nb\ncz\nbaz\n")
    .stderr("echo a\n  echo b\n  echo cz\necho baz\n")
    .success();
}

#[test]
fn windows_interpreter_path_no_base() {
  if cfg!(not(windows)) {
    return;
  }
  Test::new()
    .justfile(
      r"
    foo:
      #!powershell

      exit 0
  ",
    )
    .success();
}
