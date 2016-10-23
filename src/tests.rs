/*
extern crate tempdir;

use super::{ErrorKind, Justfile};

fn expect_error(text: &str, line: usize, expected_error_kind: ErrorKind) {
  match super::parse(text) {
    Ok(_) => panic!("Expected {:?} but parse succeeded", expected_error_kind),
    Err(error) => {
      if error.line != line {
        panic!("Expected {:?} error on line {} but error was on line {}",
               expected_error_kind, line, error.line);
      }
      if error.kind != expected_error_kind {
        panic!("Expected {:?} error but got {:?}", expected_error_kind, error.kind);
      }
    }
  }
}

fn check_recipe(
  justfile: &Justfile,
  name: &str,
  line: usize,
  leading_whitespace: &str,
  lines: &[&str],
  dependencies: &[&str]
) {
  let recipe = match justfile.recipes.get(name) {
    Some(recipe) => recipe,
    None => panic!("Justfile had no recipe \"{}\"", name),
  };
  assert_eq!(recipe.name, name);
  assert_eq!(recipe.line_number, line);
  assert_eq!(recipe.leading_whitespace, leading_whitespace);
  assert_eq!(recipe.lines, lines);
  assert_eq!(recipe.dependencies.iter().cloned().collect::<Vec<_>>(), dependencies);
}

#[test]
fn circular_dependency() {
  expect_error("a: b\nb: a", 1, ErrorKind::CircularDependency{circle: vec!["a", "b", "a"]});
}

#[test]
fn duplicate_dependency() {
  expect_error("a: b b", 0, ErrorKind::DuplicateDependency{name: "b"});
}

#[test]
fn duplicate_recipe() {
  expect_error(
    "a:\na:",
    1, ErrorKind::DuplicateRecipe{first: 0, name: "a"}
  );
}

#[test]
fn tab_after_spaces() {
  expect_error(
    "a:\n \tspaces",
    1, ErrorKind::TabAfterSpace{whitespace: " \t"}
  );
}

#[test]
fn mixed_leading_whitespace() {
  expect_error(
    "a:\n\t  spaces",
    1, ErrorKind::MixedLeadingWhitespace{whitespace: "\t  "}
  );
}

#[test]
fn inconsistent_leading_whitespace() {
  expect_error(
    "a:\n\t\ttabs\n\t\ttabs\n  spaces",
    3, ErrorKind::InconsistentLeadingWhitespace{expected: "\t\t", found: "  "}
  );
}

#[test]
fn shebang_errors() {
  expect_error("#!/bin/sh", 0, ErrorKind::OuterShebang);
  expect_error("a:\n echo hello\n #!/bin/sh", 2, ErrorKind::NonLeadingShebang{recipe:"a"});
}

#[test]
fn unknown_dependency() {
  expect_error("a: b", 0, ErrorKind::UnknownDependency{name: "a", unknown: "b"});
}

#[test]
fn extra_whitespace() {
  expect_error("a:\n blah\n  blarg", 2, ErrorKind::ExtraLeadingWhitespace);
  expect_success("a:\n #!\n  print(1)");
}

#[test]
fn unparsable() {
  expect_error("hello", 0, ErrorKind::Unparsable);
}

/*
   can we bring this error back?
#[test]
fn unparsable_dependencies() {
  expect_error("a: -f", 0, ErrorKind::UnparsableDependencies);
}
*/

/*
   we should be able to emit these errors
#[test]
fn bad_recipe_names() {
  fn expect_bad_name(text: &str, name: &str) {
    expect_error(text, 0, ErrorKind::UnknownStartOfToken{name: name});
  }
  expect_bad_name("Z:", "Z");
  expect_bad_name("a-:", "a-");
  expect_bad_name("-a:", "-a");
  expect_bad_name("a--a:", "a--a");
  expect_bad_name("@:", "@");
}
*/

#[test]
fn parse() {
  let justfile = expect_success("a: b c\nb: c\n echo hello\n\nc:\n\techo goodbye\n#\n#hello");
  assert!(justfile.recipes.keys().cloned().collect::<Vec<_>>() == vec!["a", "b", "c"]);
  check_recipe(&justfile, "a", 0, "",   &[              ], &["b", "c"]);
  check_recipe(&justfile, "b", 1, " ",  &["echo hello"  ], &["c"     ]);
  check_recipe(&justfile, "c", 4, "\t", &["echo goodbye"], &[        ]);
}

#[test]
fn first() {
  let justfile = expect_success("#hello\n#goodbye\na:\nb:\nc:\n");
  assert!(justfile.first().unwrap() == "a");
}

#[test]
fn unknown_recipes() {
  match expect_success("a:\nb:\nc:").run(&["a", "x", "y", "z"]).unwrap_err() {
    super::RunError::UnknownRecipes{recipes} => assert_eq!(recipes, &["x", "y", "z"]),
    other @ _ => panic!("expected an unknown recipe error, but got: {}", other),
  }
}

#[test]
fn code_error() {
  match expect_success("fail:\n @function x { return 100; }; x").run(&["fail"]).unwrap_err() {
    super::RunError::Code{recipe, code} => {
      assert_eq!(recipe, "fail");
      assert_eq!(code, 100);
    },
    other @ _ => panic!("expected a code run error, but got: {}", other),
  }
}

#[test]
fn run_order() {
  let tmp = tempdir::TempDir::new("run_order").unwrap_or_else(|err| panic!("tmpdir: failed to create temporary directory: {}", err));
  let path = tmp.path().to_str().unwrap_or_else(|| panic!("tmpdir: path was not valid UTF-8")).to_owned();
  let text = r"
a:
  @touch a

b: a
  @mv a b

c: b
  @mv b c

d: c
  @rm c
";
  super::std::env::set_current_dir(path).expect("failed to set current directory");
  expect_success(text).run(&["a", "d"]).unwrap();
}

#[test]
fn shebang() {
  // this test exists to make sure that shebang recipes
  // run correctly. although this script is still
  // executed by sh its behavior depends on the value of a
  // variable and continuing even though a command fails
  let text = "
a:
 #!/usr/bin/env sh
 code=200
 function x { return $code; }
 x
 x
";

  match expect_success(text).run(&["a"]).unwrap_err() {
    super::RunError::Code{recipe, code} => {
      assert_eq!(recipe, "a");
      assert_eq!(code, 200);
    },
    other @ _ => panic!("expected an code run error, but got: {}", other),
  }
}


*/

use super::{Token, Error, ErrorKind, Justfile};

fn tokenize_success(text: &str, expected_summary: &str) {
  let tokens = super::tokenize(text).unwrap();
  let roundtrip = tokens.iter().map(|t| {
    let mut s = String::new();
    s += t.prefix;
    s += t.lexeme;
    s
  }).collect::<Vec<_>>().join("");
  assert_eq!(text, roundtrip);
  assert_eq!(token_summary(&tokens), expected_summary);
}

fn tokenize_error(text: &str, expected: Error) {
  if let Err(error) = super::tokenize(text) {
    assert_eq!(error.text,   expected.text);
    assert_eq!(error.index,  expected.index);
    assert_eq!(error.line,   expected.line);
    assert_eq!(error.column, expected.column);
    assert_eq!(error.kind,   expected.kind);
    assert_eq!(error,        expected);
  } else {
    panic!("tokenize() succeeded but expected: {}\n{}", expected, text);
  }
}

fn token_summary(tokens: &[Token]) -> String {
  tokens.iter().map(|t| {
    match t.class {
      super::TokenClass::Line{..}    => "*",
      super::TokenClass::Name        => "N",
      super::TokenClass::Colon       => ":",
      super::TokenClass::Equals      => "=",
      super::TokenClass::Comment{..} => "#",
      super::TokenClass::Indent{..}  => ">",
      super::TokenClass::Dedent      => "<",
      super::TokenClass::Eol         => "$",
      super::TokenClass::Eof         => ".",
    }
  }).collect::<Vec<_>>().join("")
}

fn parse_success(text: &str) -> Justfile {
  match super::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error {}", error),
  }
}

#[test]
fn tokenize() {
  let text = "bob

hello blah blah blah : a b c #whatever
";
  tokenize_success(text, "N$$NNNN:NNN#$.");

  let text = "
hello:
  a
  b

  c

  d

bob:
  frank
  ";
  
  tokenize_success(text, "$N:$>*$*$$*$$*$$<N:$>*$.");

  tokenize_success("a:=#", "N:=#.")
}

#[test]
fn inconsistent_leading_whitespace() {
  let text = "a:
 0
 1
\t2
";
  tokenize_error(text, Error {
    text:   text,
    index:  9,
    line:   3,
    column: 0,
    kind:   ErrorKind::InconsistentLeadingWhitespace{expected: " ", found: "\t"},
  });

  let text = "a:
\t\t0
\t\t 1
\t  2
";
  tokenize_error(text, Error {
    text:   text,
    index:  12,
    line:   3,
    column: 0,
    kind:   ErrorKind::InconsistentLeadingWhitespace{expected: "\t\t", found: "\t  "},
  });
}

#[test]
fn outer_shebang() {
  let text = "#!/usr/bin/env bash";
  tokenize_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    kind:   ErrorKind::OuterShebang
  });
}

#[test]
fn unknown_start_of_token() {
  let text = "~";
  tokenize_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    kind:   ErrorKind::UnknownStartOfToken
  });
}

#[test]
fn parse() {
  parse_success("

# hello


  ");
}
