use super::*;

// A backslash as the last character of a justfile is dropped by
// `Lexer::lex_escape`: when `self.next` is `None`, no branch fires, so neither
// a token nor an error is produced. In an indented context the leftover
// in-progress token then trips `assert_eq!(self.current_token_length(), 0)` in
// `Lexer::lex_dedent` (src/lexer.rs:603) and just panics with exit code 101.
// In an unindented context the backslash is silently discarded, even though a
// backslash followed by anything other than a newline is otherwise an invalid
// escape sequence error. Both cases should be compile errors.
#[test]
fn backslash_at_end_of_file() {
  Test::new()
    .justfile("x := 'y'\n \\")
    .stderr_regex("error:.*")
    .failure();

  Test::new()
    .justfile("x := 'y' \\")
    .arg("--dump")
    .stderr_regex("error:.*")
    .failure();
}

// An unterminated format string whose last interpolation has been closed lexes
// via the `}` arm of `Lexer::lex_normal`, so the in-progress token starts with
// `}}` rather than a quote. When the lexer then hits end-of-file and calls
// `Lexer::error(UnterminatedString)`, `StringKind::from_token_start` fails and
// the user sees "internal error, this may indicate a bug in just"
// (src/lexer.rs:232) instead of the normal unterminated string error that
// `x := f'` produces.
#[test]
fn unterminated_format_string_produces_internal_error() {
  Test::new()
    .justfile("x := f'{{}}")
    .stderr_regex("error: unterminated string.*")
    .failure();
}

// The documented `\<newline>` escape in cooked strings only matches a bare
// linefeed: `Parser::cook_string` handles `'\n'` after a backslash but not
// `'\r'` (src/parser.rs:1135), so in a justfile with CRLF line endings the
// escape fails with "`\r` is not a valid escape sequence". CRLF is accepted
// everywhere else, including non-string line continuations.
#[test]
fn crlf_cooked_string_line_continuation() {
  Test::new()
    .justfile("x := \"a\\\r\nb\"\r\n")
    .args(["--evaluate", "x"])
    .stdout("ab")
    .success();
}

// `Lexer::lex_string` computes the format-string flag for backticks as well as
// strings (src/lexer.rs:830), so an `f`-prefixed backtick containing an
// interpolation parses as an ordinary format string and the command is
// silently never executed, while an `f`-prefixed backtick without an
// interpolation is a parse error. It should be an error in both cases;
// backticks have no format variant in the grammar.
#[test]
fn format_backtick_is_not_executed() {
  Test::new()
    .justfile("x := f`echo {{arch()}}`")
    .arg("--evaluate")
    .stderr_regex("error:.*backtick.*")
    .failure();
}

// The `!include` migration check in `Lexer::lex_normal` fires on
// `self.rest().starts_with("!include")` (src/lexer.rs:473) anywhere a `!` is
// lexed, not just at the start of a line where the old directive could appear,
// so negating a variable whose name starts with `include` is a compile error.
#[test]
fn negation_misparsed_as_include_directive() {
  Test::new()
    .justfile(
      "
        set unstable
        set lists

        include-tests := bool('false')

        x := !include-tests
      ",
    )
    .args(["--evaluate", "x"])
    .stdout("true")
    .success();
}

// `%#z` is a parse-only chrono specifier: `datetime_parse` accepts it but
// chrono's formatter returns `fmt::Error` for it, and the `to_string` call in
// `Function::datetime` (src/function.rs:295) panics with "a Display
// implementation returned an error unexpectedly". Invalid format strings
// should produce an error, as `%!` does. `datetime_utc` has the same bug.
#[test]
fn datetime_with_parse_only_specifier_panics() {
  Test::new()
    .justfile("x := datetime('%#z')")
    .arg("--evaluate")
    .stderr_regex("error:.*")
    .failure();
}

// `Config::timestamp` formats `--timestamp-format` with the same panicking
// `to_string` pattern (src/config.rs:154): an invalid specifier like `%Q`
// aborts just with a panic instead of reporting an error, even though the
// `datetime` function was already hardened against exactly this.
#[test]
fn invalid_timestamp_format_panics() {
  Test::new()
    .justfile(
      "
        foo:
          @echo bar
      ",
    )
    .args(["--timestamp", "--timestamp-format", "%Q", "foo"])
    .stderr_regex("error:.*")
    .failure();
}

// `Utf8Path::parent` returns `Some("")` for any single-component relative
// path, and `Function::parent_directory` (src/function.rs:576) maps the empty
// parent to ".". That fallback is correct for a bare filename like `foo`, but
// for `.` and `..` it claims the directory is its own parent. The lexical
// parents are `..` and `../..` respectively (an error would also be
// reasonable, but returning the input's own location is not).
#[test]
fn parent_directory_of_dot_paths() {
  assert_eval("parent_directory('.')", "..");
  assert_eval("parent_directory('..')", "../..");
}

// Module `[doc(...)]` expressions are const-evaluated by the analyzer
// (src/analyzer.rs:277) without first resolving their variable references,
// unlike recipe doc attributes. An undefined variable produces
// `Error::internal`, which `Error::unwrap_const` cannot map, hitting
// `unreachable!` (src/error.rs:455) and panicking. The same construct on a
// recipe produces the compile error asserted below.
#[test]
fn module_doc_with_undefined_variable_panics() {
  Test::new()
    .write("bar.just", "")
    .justfile(
      "
        [doc(foo)]
        mod bar
      ",
    )
    .arg("--list")
    .stderr(
      "
        error: variable `foo` not defined
         ——▶ justfile:1:6
          │
        1 │ [doc(foo)]
          │      ^^^
      ",
    )
    .failure();
}

// The static circular-dependency check misses cycles routed through a
// user-defined function: `AssignmentResolver::resolve_reference` handles calls
// via `Analyzer::resolve_call`, which checks arity but never descends into the
// function body (src/assignment_resolver.rs:61). The runtime recursion guard
// uses so many stack frames per cycle iteration that a debug build overflows
// its stack and aborts with SIGABRT before reaching the limit of 256.
#[test]
fn function_assignment_reference_cycle_overflows_stack() {
  Test::new()
    .justfile(
      "
        set unstable

        a := f('bar')

        f(y) := a + y
      ",
    )
    .args(["--evaluate", "a"])
    .stderr_regex("error:.*")
    .failure();
}

// `Evaluator::evaluate_variable` consults the scope — which includes the
// prelude constants — before checking the module's own not-yet-evaluated
// assignments (src/evaluator.rs:607). Assignments are evaluated in
// alphabetical order, so a variable that sorts before a redefined constant
// sees the built-in value while one that sorts after it sees the redefinition:
// renaming `a` to `z` below changes its value. One name must not denote two
// values in a single run, and assignment order is documented as irrelevant.
#[test]
fn constant_shadowing_depends_on_assignment_names() {
  Test::new()
    .justfile(
      "
        a := HEX

        HEX := 'foo'
      ",
    )
    .args(["--evaluate", "a"])
    .stdout("foo")
    .success();
}

// `Evaluator::evaluate_defined_function` builds a fresh root scope containing
// only the function's parameters (src/evaluator.rs:289), so every global the
// body references is re-evaluated from its assignment on each call instead of
// being read from the already-evaluated module scope. Backtick assignments
// therefore execute more than once per run — the recipe below observes `g`
// with two different values on a single line.
#[test]
fn user_defined_functions_reevaluate_assignments() {
  Test::new()
    .justfile(
      "
        set unstable

        g := `echo x >> counter; wc -l < counter | tr -d ' '`

        f() := g

        foo:
          @echo {{ g }} {{ f() }} {{ g }}
      ",
    )
    .stdout("1 1 1\n")
    .success();
}

// `Compiler::compile` deduplicates ASTs by source path, so a file named by two
// `mod` statements is compiled once, with `module_path` set to only one of the
// modules. `Analyzer::analyze` matches overrides against that single path
// (src/analyzer.rs:199), so overrides for one module are silently ignored
// while overrides for the other apply to both: below, `--set a::x` has no
// effect, and `--set b::x over` would change what `a::show` prints.
#[test]
fn overrides_for_modules_sharing_a_source_file_are_ignored() {
  Test::new()
    .write("shared.just", "x := 'def'\nshow:\n    @echo x={{ x }}\n")
    .justfile(
      "
        mod a 'shared.just'

        mod b 'shared.just'
      ",
    )
    .args(["--set", "a::x", "over", "a::show"])
    .stdout("x=over\n")
    .success();
}

// The guard/infallible sigil conflict check runs on every line of a recipe
// body (src/analyzer.rs:305) without consulting the `continued` flag that
// gates the neighboring leading-whitespace check, so text at the start of a
// continuation line is misinterpreted as sigils. Without `set guards` this
// justfile prints `a -?bar`; with it, it is a compile error.
#[test]
fn sigils_rejected_on_continuation_lines() {
  Test::new()
    .justfile(
      "
        set guards

        foo:
          echo a \\
          -?bar
      ",
    )
    .stdout("a -?bar\n")
    .stderr("echo a -?bar\n")
    .success();
}

// With `set ignore-comments`, `UnresolvedRecipe::resolve` skips comment lines
// when resolving variable references (src/unresolved_recipe.rs:161), but
// `Recipe::run_script` still evaluates every body line of a shebang recipe
// (src/recipe.rs:456). An interpolation in a comment — which the existing
// `dont_analyze_comments` test establishes must be ignored — hits the
// evaluator unresolved and produces "internal runtime error ... attempted to
// evaluate undefined variable". Comment lines that continue a previous line
// take the same mismatched path in `run_shell`.
#[test]
fn ignore_comments_evaluates_comments_in_scripts() {
  Test::new()
    .justfile(
      "
        set ignore-comments

        foo:
          #!/bin/sh
          # {{ undefined }}
          echo ok
      ",
    )
    .stdout("ok\n")
    .success();
}

// Whether a recipe is a shebang recipe is decided at parse time from the raw
// first line, but `Shebang::new` runs on the evaluated text: when the
// interpolation evaluates to empty, it returns `None` and `Recipe::run_script`
// reports "internal runtime error, this may indicate a bug in just: bad
// shebang line: #!" (src/recipe.rs:497), telling the user to file an issue
// against just. Ordinary user input should produce an ordinary error naming
// the recipe.
#[test]
fn empty_shebang_interpolation_produces_internal_error() {
  Test::new()
    .justfile(
      "
        foo:
          #!{{ '' }}
          echo bar
      ",
    )
    .stderr_regex("error: recipe `foo`.*")
    .failure();
}

// `Parser::take_doc_comment` absorbs any comment directly above a recipe as
// its doc string, and the doc is re-emitted as `# {doc}` (src/item.rs:80), so
// `just --fmt` rewrites the shebang line of an executable justfile as
// `# !/usr/bin/env ...`, silently breaking the documented "just scripts"
// feature: after formatting, the OS no longer execs the file with just.
#[test]
fn fmt_corrupts_shebang() {
  Test::new()
    .justfile(
      "
        #!/usr/bin/env -S just --justfile
        foo:
            echo runs
      ",
    )
    .arg("--fmt")
    .stderr_regex("(?s).*")
    .expect_file(
      "justfile",
      "#!/usr/bin/env -S just --justfile\nfoo:\n    echo runs\n",
    )
    .success();
}

// `Subcommand::dump` reads the analyzer-merged `settings.indentation`, which
// honors imports, while `Subcommand::format` uses `Ast::indentation`
// (src/ast.rs:17), a raw scan of root items only that also ignores
// `Item::is_enabled`, so `--fmt` and `--dump` render the same justfile with
// different indentation when the setting comes from an import (or from a
// `[windows]`/`[unix]`-gated setting, making `--dump` platform-dependent).
#[test]
fn fmt_ignores_imported_indentation_setting() {
  Test::new()
    .write("inc.just", "set indentation := \"\\t\"\n")
    .justfile(
      "
        import 'inc.just'

        foo:
            echo bar
      ",
    )
    .arg("--fmt")
    .stderr_regex("(?s).*")
    .expect_file("justfile", "import 'inc.just'\n\nfoo:\n\techo bar\n")
    .success();
}

// The `variable_references` collection in `Justfile::run`
// (src/justfile.rs:200) walks the dependency graph with a plain stack and no
// visited set, so a chain of recipes that each depend twice on the next is
// traversed 2^n times: depth 20 takes ~0.3 seconds in a debug build, depth 24
// ~4 seconds, and depth 40 effectively hangs forever, even though the graph
// has only 41 recipes and execution itself is deduplicated.
#[test]
fn dependency_graph_traversal_is_exponential() {
  let n = 40;

  let mut justfile = String::new();

  for i in 0..n {
    justfile.push_str(&format!("r{i}: r{} r{}\n", i + 1, i + 1));
  }

  justfile.push_str(&format!("r{n}:\n"));

  let tempdir = tempdir();

  fs::write(tempdir.path().join("justfile"), justfile).unwrap();

  let mut child = Command::new(JUST)
    .args(["--dry-run", "r0"])
    .current_dir(tempdir.path())
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .unwrap();

  let start = Instant::now();

  while start.elapsed() < Duration::from_secs(10) {
    if let Some(status) = child.try_wait().unwrap() {
      assert!(status.success());
      return;
    }

    thread::sleep(Duration::from_millis(100));
  }

  child.kill().unwrap();
  child.wait().unwrap();

  panic!("dependency graph traversal of {n} recipes did not complete within 10 seconds");
}
