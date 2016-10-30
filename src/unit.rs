extern crate tempdir;

use super::{Token, Error, ErrorKind, Justfile, RunError};
use super::TokenKind::*;
use std::collections::BTreeMap;

fn tokenize_success(text: &str, expected_summary: &str) {
  let tokens = super::tokenize(text).unwrap();
  let roundtrip = tokens.iter().map(|t| {
    let mut s = String::new();
    s += t.prefix;
    s += t.lexeme;
    s
  }).collect::<Vec<_>>().join("");
  let summary = token_summary(&tokens);
  if summary != expected_summary {
    panic!("token summary mismatch:\nexpected: {}\ngot:      {}\n", expected_summary, summary);
  }
  assert_eq!(text, roundtrip);
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
    match t.kind {
      super::TokenKind::Backtick           => "`",
      super::TokenKind::Colon              => ":",
      super::TokenKind::Comment{..}        => "#",
      super::TokenKind::Dedent             => "<",
      super::TokenKind::Eof                => ".",
      super::TokenKind::Eol                => "$",
      super::TokenKind::Equals             => "=",
      super::TokenKind::Indent{..}         => ">",
      super::TokenKind::InterpolationEnd   => "}",
      super::TokenKind::InterpolationStart => "{",
      super::TokenKind::Line{..}           => "^",
      super::TokenKind::Name               => "N",
      super::TokenKind::Plus               => "+",
      super::TokenKind::StringToken        => "'",
      super::TokenKind::Text               => "_",
    }
  }).collect::<Vec<_>>().join("")
}

fn parse_success(text: &str) -> Justfile {
  match super::parse(text) {
    Ok(justfile) => justfile,
    Err(error) => panic!("Expected successful parse but got error:\n{}", error),
  }
}

fn parse_summary(input: &str, output: &str) {
  let justfile = parse_success(input);
  let s = format!("{:#}", justfile);
  if s != output {
    println!("got:\n\"{}\"\n", s);
    println!("\texpected:\n\"{}\"", output);
    assert_eq!(s, output);
  }
}

fn parse_error(text: &str, expected: Error) {
  if let Err(error) = super::parse(text) {
    assert_eq!(error.text,   expected.text);
    assert_eq!(error.index,  expected.index);
    assert_eq!(error.line,   expected.line);
    assert_eq!(error.column, expected.column);
    assert_eq!(error.kind,   expected.kind);
    assert_eq!(error.width,  expected.width);
    assert_eq!(error,        expected);
  } else {
    panic!("Expected {:?} but parse succeeded", expected.kind);
  }
}

#[test]
fn tokenize_recipe_interpolation_eol() {
  let text = "foo:
 {{hello}}
";
  tokenize_success(text, "N:$>^{N}$<.");
}

#[test]
fn tokenize_recipe_interpolation_eof() {
  let text = "foo:
 {{hello}}";
  tokenize_success(text, "N:$>^{N}<.");
}

#[test]
fn tokenize_recipe_complex_interpolation_expression() {
  let text = "foo:\n {{a + b + \"z\" + blarg}}";
  tokenize_success(text, "N:$>^{N+N+'+N}<.");
}

#[test]
fn tokenize_recipe_multiple_interpolations() {
  let text = "foo:\n {{a}}0{{b}}1{{c}}";
  tokenize_success(text, "N:$>^{N}_{N}_{N}<.");
}

#[test]
fn tokenize_junk() {
  let text = "bob

hello blah blah blah : a b c #whatever
";
  tokenize_success(text, "N$$NNNN:NNN#$.");
}

#[test]
fn tokenize_empty_lines() {
  let text = "
hello:
  asdf
  bsdf

  csdf

  dsdf
  ";

  tokenize_success(text, "$N:$>^_$^_$$^_$$^_$<.");
}

#[test]
fn tokenize_interpolation_backticks() {
  tokenize_success(
    "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    "N:$>^_{`+`}<."
  );
}

#[test]
fn tokenize_assignment_backticks() {
  tokenize_success(
    "a = `echo hello` + `echo goodbye`",
    "N=`+`."
  );
}

#[test]
fn tokenize_multiple() {
  let text = "
hello:
  a
  b

  c

  d

bob:
  frank
  ";

  tokenize_success(text, "$N:$>^_$^_$$^_$$^_$$<N:$>^_$<.");
}


#[test]
fn tokenize_comment() {
  tokenize_success("a:=#", "N:=#.")
}

#[test]
fn tokenize_space_then_tab() {
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
    width:  None,
    kind:   ErrorKind::InconsistentLeadingWhitespace{expected: " ", found: "\t"},
  });
}

#[test]
fn tokenize_tabs_then_tab_space() {
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
    width:  None,
    kind:   ErrorKind::InconsistentLeadingWhitespace{expected: "\t\t", found: "\t  "},
  });
}

#[test]
fn tokenize_outer_shebang() {
  let text = "#!/usr/bin/env bash";
  tokenize_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   ErrorKind::OuterShebang
  });
}

#[test]
fn tokenize_unknown() {
  let text = "~";
  tokenize_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   ErrorKind::UnknownStartOfToken
  });
}

#[test]
fn parse_empty() {
  parse_summary("

# hello


  ", "");
}

#[test]
fn parse_complex() {
  parse_summary("
x:
y:
z:
foo = \"xx\"
bar = foo
goodbye = \"y\"
hello a b    c   : x y    z #hello
  #! blah
  #blarg
  {{ foo + bar}}abc{{ goodbye\t  + \"x\" }}xyz
  1
  2
  3
", "bar = foo

foo = \"xx\"

goodbye = \"y\"

hello a b c: x y z
    #! blah
    #blarg
    {{foo + bar}}abc{{goodbye + \"x\"}}xyz
    1
    2
    3

x:

y:

z:");
}

#[test]
fn parse_assignments() {
  parse_summary(
r#"a = "0"
c = a + b + a + b
b = "1"
"#, 

r#"a = "0"

b = "1"

c = a + b + a + b"#);
}

#[test]
fn parse_assignment_backticks() {
  parse_summary(
"a = `echo hello`
c = a + b + a + b
b = `echo goodbye`", 

"a = `echo hello`

b = `echo goodbye`

c = a + b + a + b");
}

#[test]
fn parse_interpolation_backticks() {
  parse_summary(
r#"a:
 echo {{  `echo hello` + "blarg"   }} {{   `echo bob`   }}"#, 
r#"a:
    echo {{`echo hello` + "blarg"}} {{`echo bob`}}"#, 
 );
}

#[test]
fn missing_colon() {
  let text = "a b c\nd e f";
  parse_error(text, Error {
    text:   text,
    index:  5,
    line:   0,
    column: 5,
    width:  Some(1),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Name, Colon], found: Eol},
  });
}

#[test]
fn missing_eol() {
  let text = "a b c: z =";
  parse_error(text, Error {
    text:   text,
    index:  9,
    line:   0,
    column: 9,
    width:  Some(1),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Name, Eol, Eof], found: Equals},
  });
}

#[test]
fn eof_test() {
  parse_summary("x:\ny:\nz:\na b c: x y z", "a b c: x y z\n\nx:\n\ny:\n\nz:");
}

#[test]
fn duplicate_argument() {
  let text = "a b b:";
  parse_error(text, Error {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   ErrorKind::DuplicateArgument{recipe: "a", argument: "b"}
  });
}

#[test]
fn argument_shadows_varible() {
  let text = "foo = \"h\"\na foo:";
  parse_error(text, Error {
    text:   text,
    index:  12,
    line:   1,
    column: 2,
    width:  Some(3),
    kind:   ErrorKind::ArgumentShadowsVariable{argument: "foo"}
  });
}

#[test]
fn dependency_with_arguments() {
  let text = "foo arg:\nb: foo";
  parse_error(text, Error {
    text:   text,
    index:  12,
    line:   1,
    column: 3,
    width:  Some(3),
    kind:   ErrorKind::DependencyHasArguments{recipe: "b", dependency: "foo"}
  });
}

#[test]
fn duplicate_dependency() {
  let text = "a b c: b c z z";
  parse_error(text, Error {
    text:   text,
    index:  13,
    line:   0,
    column: 13,
    width:  Some(1),
    kind:   ErrorKind::DuplicateDependency{recipe: "a", dependency: "z"}
  });
}

#[test]
fn duplicate_recipe() {
  let text = "a:\nb:\na:";
  parse_error(text, Error {
    text:   text,
    index:  6,
    line:   2,
    column: 0,
    width:  Some(1),
    kind:   ErrorKind::DuplicateRecipe{recipe: "a", first: 0}
  });
}

#[test]
fn circular_recipe_dependency() {
  let text = "a: b\nb: a";
  parse_error(text, Error {
    text:   text,
    index:  8,
    line:   1,
    column: 3,
    width:  Some(1),
    kind:   ErrorKind::CircularRecipeDependency{recipe: "b", circle: vec!["a", "b", "a"]}
  });
}

#[test]
fn circular_variable_dependency() {
  let text = "a = b\nb = a";
  parse_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   ErrorKind::CircularVariableDependency{variable: "a", circle: vec!["a", "b", "a"]}
  });
}

#[test]
fn duplicate_variable() {
  let text = "a = \"0\"\na = \"0\"";
  parse_error(text, Error {
    text:   text,
    index:  8,
    line:   1,
    column: 0,
    width:  Some(1),
    kind:   ErrorKind::DuplicateVariable{variable: "a"}
  });
}

#[test]
fn unterminated_string() {
  let text = r#"a = ""#;
  parse_error(text, Error {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  None,
    kind:   ErrorKind::UnterminatedString,
  });
}

#[test]
fn unterminated_string_with_escapes() {
  let text = r#"a = "\n\t\r\"\\"#;
  parse_error(text, Error {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  None,
    kind:   ErrorKind::UnterminatedString,
  });
}

#[test]
fn string_quote_escape() {
  parse_summary(
    r#"a = "hello\"""#,
    r#"a = "hello\"""#
  );
}

#[test]
fn string_escapes() {
  parse_summary(
    r#"a = "\n\t\r\"\\""#,
    r#"a = "\n\t\r\"\\""#
  );
}

#[test]
fn arguments() {
  parse_summary(
"a b c:
  {{b}} {{c}}",
"a b c:
    {{b}} {{c}}",
  );
}

#[test]
fn self_recipe_dependency() {
  let text = "a: a";
  parse_error(text, Error {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  Some(1),
    kind:   ErrorKind::CircularRecipeDependency{recipe: "a", circle: vec!["a", "a"]}
  });
}

#[test]
fn self_variable_dependency() {
  let text = "a = a";
  parse_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   ErrorKind::CircularVariableDependency{variable: "a", circle: vec!["a", "a"]}
  });
}

#[test]
fn unknown_dependency() {
  let text = "a: b";
  parse_error(text, Error {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  Some(1),
    kind:   ErrorKind::UnknownDependency{recipe: "a", unknown: "b"}
  });
}

#[test]
fn mixed_leading_whitespace() {
  let text = "a:\n\t echo hello";
  parse_error(text, Error {
    text:   text,
    index:  3,
    line:   1,
    column: 0,
    width:  None,
    kind:   ErrorKind::MixedLeadingWhitespace{whitespace: "\t "}
  });
}

#[test]
fn conjoin_or() {
  assert_eq!("1",             super::Or(&[1      ]).to_string());
  assert_eq!("1 or 2",        super::Or(&[1,2    ]).to_string());
  assert_eq!("1, 2, or 3",    super::Or(&[1,2,3  ]).to_string());
  assert_eq!("1, 2, 3, or 4", super::Or(&[1,2,3,4]).to_string());
}

#[test]
fn conjoin_and() {
  assert_eq!("1",             super::And(&[1      ]).to_string());
  assert_eq!("1 and 2",        super::And(&[1,2    ]).to_string());
  assert_eq!("1, 2, and 3",    super::And(&[1,2,3  ]).to_string());
  assert_eq!("1, 2, 3, and 4", super::And(&[1,2,3,4]).to_string());
}

#[test]
fn unknown_recipes() {
  match parse_success("a:\nb:\nc:").run(&BTreeMap::new(), &["a", "x", "y", "z"], false, false).unwrap_err() {
    RunError::UnknownRecipes{recipes} => assert_eq!(recipes, &["x", "y", "z"]),
    other => panic!("expected an unknown recipe error, but got: {}", other),
  }
}

#[test]
fn extra_whitespace() {
  // we might want to make extra leading whitespace a line continuation in the future,
  // so make it a error for now
  let text = "a:\n blah\n  blarg";
  parse_error(text, Error {
    text:   text,
    index:  10,
    line:   2,
    column: 1,
    width:  Some(6),
    kind:   ErrorKind::ExtraLeadingWhitespace
  });

  // extra leading whitespace is okay in a shebang recipe
  parse_success("a:\n #!\n  print(1)");
}

#[test]
fn bad_recipe_names() {
  // We are extra strict with names. Although the tokenizer
  // will tokenize anything that matches /[a-zA-Z0-9_-]+/
  // as a name, we throw an error if names do not match
  // / [a-z](-?[a-z])* /. This is to support future expansion
  // of justfile and command line syntax.
  fn bad_name(text: &str, name: &str, index: usize, line: usize, column: usize) {
    parse_error(text, Error {
      text:   text,
      index:  index,
      line:   line,
      column: column,
      width:  Some(name.len()),
      kind:   ErrorKind::BadName{name: name}
    });
  }

  bad_name("-a",     "-a",   0, 0, 0);
  bad_name("_a",     "_a",   0, 0, 0);
  bad_name("a-",     "a-",   0, 0, 0);
  bad_name("a_",     "a_",   0, 0, 0);
  bad_name("a__a",   "a__a", 0, 0, 0);
  bad_name("a--a",   "a--a", 0, 0, 0);
  bad_name("a: a--", "a--",  3, 0, 3);
  bad_name("a: 9a",  "9a",   3, 0, 3);
  bad_name("a: 9a",  "9a",   3, 0, 3);
  bad_name("a:\nZ:", "Z",    3, 1, 0);
}

#[test]
fn bad_interpolation_variable_name() {
  let text = "a:\n echo {{hello--hello}}";
  parse_error(text, Error {
    text:   text,
    index:  11,
    line:   1,
    column: 8,
    width:  Some(12),
    kind:   ErrorKind::BadName{name: "hello--hello"}
  });
}

#[test]
fn interpolation_outside_of_recipe() {
  let text = "{{";
  parse_error(text, Error {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(2),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Name], found: InterpolationStart},
  });
}

#[test]
fn unclosed_interpolation_delimiter() {
  let text = "a:\n echo {{ foo";
  parse_error(text, Error {
    text:   text,
    index:  15,
    line:   1,
    column: 12,
    width:  Some(0),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Plus, Eol, InterpolationEnd], found: Dedent},
  });
}

#[test]
fn unknown_expression_variable() {
  let text = "x = yy";
  parse_error(text, Error {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  Some(2),
    kind:   ErrorKind::UndefinedVariable{variable: "yy"},
  });
}

#[test]
fn unknown_interpolation_variable() {
  let text = "x:\n {{   hello}}";
  parse_error(text, Error {
    text:   text,
    index:  9,
    line:   1,
    column: 6,
    width:  Some(5),
    kind:   ErrorKind::UndefinedVariable{variable: "hello"},
  });
}

#[test]
fn unknown_second_interpolation_variable() {
  let text = "wtf=\"x\"\nx:\n echo\n foo {{wtf}} {{ lol }}";
  parse_error(text, Error {
    text:   text,
    index:  33,
    line:   3,
    column: 16,
    width:  Some(3),
    kind:   ErrorKind::UndefinedVariable{variable: "lol"},
  });
}

#[test]
fn tokenize_order() {
  let text = r"
b: a
  @mv a b

a:
  @touch F
  @touch a

d: c
  @rm c

c: b
  @mv b c";
  tokenize_success(text, "$N:N$>^_$$<N:$>^_$^_$$<N:N$>^_$$<N:N$>^_<.");
}

#[test]
fn run_shebang() {
  // this test exists to make sure that shebang recipes
  // run correctly. although this script is still
  // executed by sh its behavior depends on the value of a
  // variable and continuing even though a command fails,
  // whereas in plain recipes variables are not available
  // in subsequent lines and execution stops when a line
  // fails
  let text = "
a:
 #!/usr/bin/env sh
 code=200
 function x { return $code; }
 x
 x
";

  match parse_success(text).run(&BTreeMap::new(), &["a"], false, false).unwrap_err() {
    RunError::Code{recipe, code} => {
      assert_eq!(recipe, "a");
      assert_eq!(code, 200);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn code_error() {
  match parse_success("fail:\n @function x { return 100; }; x").run(&BTreeMap::new(), &["fail"], false, false).unwrap_err() {
    RunError::Code{recipe, code} => {
      assert_eq!(recipe, "fail");
      assert_eq!(code, 100);
    },
    other => panic!("expected a code run error, but got: {}", other),
  }
}

#[test]
fn run_args() {
  let text = r#"
a return code:
 @function x { {{return}} {{code + "0"}}; }; x"#;

  match parse_success(text).run(&BTreeMap::new(), &["a", "return", "15"], false, false).unwrap_err() {
    RunError::Code{recipe, code} => {
      assert_eq!(recipe, "a");
      assert_eq!(code, 150);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_args() {
  match parse_success("a b c d:").run(&BTreeMap::new(), &["a", "b", "c"], false, false).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, expected} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 2);
      assert_eq!(expected, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_default() {
  match parse_success("a b c d:\n echo {{b}}{{c}}{{d}}").run(&BTreeMap::new(), &["a"], false, false).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, expected} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 0);
      assert_eq!(expected, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn backtick_code() {
  match parse_success("a:\n echo {{`function f { return 100; }; f`}}").run(&BTreeMap::new(), &["a"], false, false).unwrap_err() {
    RunError::BacktickCode{code, token} => {
      assert_eq!(code, 100);
      assert_eq!(token.lexeme, "`function f { return 100; }; f`");
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn unknown_overrides() {
  let mut overrides = BTreeMap::new();
  overrides.insert("foo", "bar");
  overrides.insert("baz", "bob");
  match parse_success("a:\n echo {{`function f { return 100; }; f`}}")
  .run(&overrides, &["a"], false, false).unwrap_err() {
    RunError::UnknownOverrides{overrides} => {
      assert_eq!(overrides, &["baz", "foo"]);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}
