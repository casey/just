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
        panic!("Expected {:?} error but got {:?}", error.kind, expected_error_kind);
      }
    }
  }
}

fn check_recipe(
  justfile: &Justfile,
  name: &str,
  line: usize,
  leading_whitespace: &str,
  commands: &[&str],
  dependencies: &[&str]
) {
  let recipe = match justfile.recipes.get(name) {
    Some(recipe) => recipe,
    None => panic!("Justfile had no recipe \"{}\"", name),
  };
  assert_eq!(recipe.name, name);
  assert_eq!(recipe.line, line);
  assert_eq!(recipe.leading_whitespace, leading_whitespace);
  assert_eq!(recipe.commands, commands);
  assert_eq!(recipe.dependencies.iter().cloned().collect::<Vec<_>>(), dependencies);
}

fn expect_success(text: &str) -> Justfile {
  match super::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error {}", error),
  }
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
fn tab_after_paces() {
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
fn shebang() {
  expect_error("#!/bin/sh", 0, ErrorKind::Shebang);
  expect_error("a:\n #!/bin/sh", 1, ErrorKind::Shebang);
}

#[test]
fn unknown_dependency() {
  expect_error("a: b", 0, ErrorKind::UnknownDependency{name: "a", unknown: "b"});
}

#[test]
fn unparsable() {
  expect_error("hello", 0, ErrorKind::Unparsable);
}

#[test]
fn unparsable_dependencies() {
  expect_error("a: -f", 0, ErrorKind::UnparsableDependencies);
}

#[test]
fn bad_recipe_names() {
  fn expect_bad_name(text: &str, name: &str) {
    expect_error(text, 0, ErrorKind::BadRecipeName{name: name});
  }
  expect_bad_name("Z:", "Z");
  expect_bad_name("a-:", "a-");
  expect_bad_name("-a:", "-a");
  expect_bad_name("a--a:", "a--a");
  expect_bad_name("@:", "@");
}

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
    other @ _ => panic!("expected an code run error, but got: {}", other),
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
