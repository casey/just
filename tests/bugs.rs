use super::*;

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
