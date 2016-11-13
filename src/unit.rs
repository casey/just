extern crate tempdir;
extern crate brev;

use super::{Token, CompileError, ErrorKind, Justfile, RunError, RunOptions};
use super::TokenKind::*;

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

fn tokenize_error(text: &str, expected: CompileError) {
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
      At                 => "@",
      Backtick           => "`",
      Colon              => ":",
      Comment{..}        => "#",
      Dedent             => "<",
      Eof                => ".",
      Eol                => "$",
      Equals             => "=",
      Indent{..}         => ">",
      InterpolationEnd   => "}",
      InterpolationStart => "{",
      Line{..}           => "^",
      Name               => "N",
      Plus               => "+",
      RawString          => "'",
      StringToken        => "\"",
      Text               => "_",
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

fn parse_error(text: &str, expected: CompileError) {
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
fn tokanize_strings() {
  tokenize_success(
    r#"a = "'a'" + '"b"' + "'c'" + '"d"'#echo hello"#,
    r#"N="+'+"+'#."#
  );
}

#[test]
fn tokenize_recipe_interpolation_eol() {
  let text = "foo: # some comment
 {{hello}}
";
  tokenize_success(text, "N:#$>^{N}$<.");
}

#[test]
fn tokenize_recipe_interpolation_eof() {
  let text = "foo: # more comments
 {{hello}}
# another comment
";
  tokenize_success(text, "N:#$>^{N}$<#$.");
}

#[test]
fn tokenize_recipe_complex_interpolation_expression() {
  let text = "foo: #lol\n {{a + b + \"z\" + blarg}}";
  tokenize_success(text, "N:#$>^{N+N+\"+N}<.");
}

#[test]
fn tokenize_recipe_multiple_interpolations() {
  let text = "foo:#ok\n {{a}}0{{b}}1{{c}}";
  tokenize_success(text, "N:#$>^{N}_{N}_{N}<.");
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
# this does something
hello:
  asdf
  bsdf

  csdf

  dsdf # whatever

# yolo
  ";

  tokenize_success(text, "$#$N:$>^_$^_$$^_$$^_$$<#$.");
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

# hello
bob:
  frank
  ";

  tokenize_success(text, "$N:$>^_$^_$$^_$$^_$$<#$N:$>^_$<.");
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
  tokenize_error(text, CompileError {
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
  tokenize_error(text, CompileError {
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
  tokenize_error(text, CompileError {
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
  tokenize_error(text, CompileError {
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
fn parse_string_default() {
  parse_summary(r#"

foo a="b\t":


  "#, r#"foo a='b\t':"#);
}

#[test]
fn parse_raw_string_default() {
  parse_summary(r#"

foo a='b\t':


  "#, r#"foo a='b\\t':"#);
}

#[test]
fn parse_export() {
  parse_summary(r#"
export a = "hello"

  "#, r#"export a = "hello""#);
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
fn parse_shebang() {
  parse_summary("
practicum = 'hello'
install:
\t#!/bin/sh
\tif [[ -f {{practicum}} ]]; then
\t\treturn
\tfi
", "practicum = \"hello\"

install:
    #!/bin/sh
    if [[ -f {{practicum}} ]]; then
    \treturn
    fi"
  );
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
  parse_error(text, CompileError {
    text:   text,
    index:  5,
    line:   0,
    column: 5,
    width:  Some(1),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Name, Colon], found: Eol},
  });
}

#[test]
fn missing_default_eol() {
  let text = "hello arg=\n";
  parse_error(text, CompileError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(1),
    kind:   ErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Eol},
  });
}

#[test]
fn missing_default_eof() {
  let text = "hello arg=";
  parse_error(text, CompileError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(0),
    kind:   ErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Eof},
  });
}

#[test]
fn missing_default_colon() {
  let text = "hello arg=:";
  parse_error(text, CompileError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(1),
    kind:   ErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Colon},
  });
}

#[test]
fn missing_default_backtick() {
  let text = "hello arg=`hello`";
  parse_error(text, CompileError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(7),
    kind:   ErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Backtick},
  });
}

#[test]
fn required_after_default() {
  let text = "hello arg='foo' bar:";
  parse_error(text, CompileError {
    text:   text,
    index:  16,
    line:   0,
    column: 16,
    width:  Some(3),
    kind:   ErrorKind::RequiredParameterFollowsDefaultParameter{parameter: "bar"},
  });
}

#[test]
fn missing_eol() {
  let text = "a b c: z =";
  parse_error(text, CompileError {
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
fn duplicate_parameter() {
  let text = "a b b:";
  parse_error(text, CompileError {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   ErrorKind::DuplicateParameter{recipe: "a", parameter: "b"}
  });
}

#[test]
fn parameter_shadows_varible() {
  let text = "foo = \"h\"\na foo:";
  parse_error(text, CompileError {
    text:   text,
    index:  12,
    line:   1,
    column: 2,
    width:  Some(3),
    kind:   ErrorKind::ParameterShadowsVariable{parameter: "foo"}
  });
}

#[test]
fn dependency_has_parameters() {
  let text = "foo arg:\nb: foo";
  parse_error(text, CompileError {
    text:   text,
    index:  12,
    line:   1,
    column: 3,
    width:  Some(3),
    kind:   ErrorKind::DependencyHasParameters{recipe: "b", dependency: "foo"}
  });
}

#[test]
fn duplicate_dependency() {
  let text = "a b c: b c z z";
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  None,
    kind:   ErrorKind::UnterminatedString,
  });
}

#[test]
fn unterminated_raw_string() {
  let text = "r a='asdf";
  parse_error(text, CompileError {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
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
fn parameters() {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
fn range() {
  assert!(super::contains(&(0..1), 0));
  assert!(super::contains(&(10..20), 15));
  assert!(!super::contains(&(0..0), 0));
  assert!(!super::contains(&(1..10), 0));
  assert!(!super::contains(&(1..10), 10));
}

#[test]
fn unknown_recipes() {
  match parse_success("a:\nb:\nc:").run(&["a", "x", "y", "z"], &Default::default()).unwrap_err() {
    RunError::UnknownRecipes{recipes, suggestion} => {
      assert_eq!(recipes, &["x", "y", "z"]);
      assert_eq!(suggestion, None);
    }
    other => panic!("expected an unknown recipe error, but got: {}", other),
  }
}

#[test]
fn extra_whitespace() {
  let text = "a:\n blah\n  blarg";
  parse_error(text, CompileError {
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
fn interpolation_outside_of_recipe() {
  let text = "{{";
  parse_error(text, CompileError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(2),
    kind:   ErrorKind::UnexpectedToken{expected: vec![Name, At], found: InterpolationStart},
  });
}

#[test]
fn unclosed_interpolation_delimiter() {
  let text = "a:\n echo {{ foo";
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  parse_error(text, CompileError {
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
  // executed by a shell its behavior depends on the value of a
  // variable and continuing even though a command fails,
  // whereas in plain recipes variables are not available
  // in subsequent lines and execution stops when a line
  // fails
  let text = "
a:
 #!/usr/bin/env sh
 code=200
  x() { return $code; }
    x
      x
";

  match parse_success(text).run(&["a"], &Default::default()).unwrap_err() {
    RunError::Code{recipe, code} => {
      assert_eq!(recipe, "a");
      assert_eq!(code, 200);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn code_error() {
  match parse_success("fail:\n @exit 100")
    .run(&["fail"], &Default::default()).unwrap_err() {
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
 @x() { {{return}} {{code + "0"}}; }; x"#;

  match parse_success(text).run(&["a", "return", "15"], &Default::default()).unwrap_err() {
    RunError::Code{recipe, code} => {
      assert_eq!(recipe, "a");
      assert_eq!(code, 150);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_some_arguments() {
  match parse_success("a b c d:").run(&["a", "b", "c"], &Default::default()).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, min, max} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 2);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_all_arguments() {
  match parse_success("a b c d:\n echo {{b}}{{c}}{{d}}")
        .run(&["a"], &Default::default()).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, min, max} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 0);
      assert_eq!(min, 3);
      assert_eq!(max, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_some_defaults() {
  match parse_success("a b c d='hello':").run(&["a", "b"], &Default::default()).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, min, max} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 1);
      assert_eq!(min, 2);
      assert_eq!(max, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn missing_all_defaults() {
  match parse_success("a b c='r' d='h':").run(&["a"], &Default::default()).unwrap_err() {
    RunError::ArgumentCountMismatch{recipe, found, min, max} => {
      assert_eq!(recipe, "a");
      assert_eq!(found, 0);
      assert_eq!(min, 1);
      assert_eq!(max, 3);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn backtick_code() {
  match parse_success("a:\n echo {{`f() { return 100; }; f`}}")
        .run(&["a"], &Default::default()).unwrap_err() {
    RunError::BacktickCode{code, token} => {
      assert_eq!(code, 100);
      assert_eq!(token.lexeme, "`f() { return 100; }; f`");
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn unknown_overrides() {
  let mut options: RunOptions = Default::default();
  options.overrides.insert("foo", "bar");
  options.overrides.insert("baz", "bob");
  match parse_success("a:\n echo {{`f() { return 100; }; f`}}")
        .run(&["a"], &options).unwrap_err() {
    RunError::UnknownOverrides{overrides} => {
      assert_eq!(overrides, &["baz", "foo"]);
    },
    other => panic!("expected an code run error, but got: {}", other),
  }
}

#[test]
fn export_assignment_backtick() {
  let text = r#"
export exported_variable = "A"
b = `echo $exported_variable`

recipe:
  echo {{b}}
"#;

  let options = RunOptions {
    quiet: true,
    ..Default::default()
  };

  match parse_success(text).run(&["recipe"], &options).unwrap_err() {
    RunError::BacktickCode{code: _, token} => {
      assert_eq!(token.lexeme, "`echo $exported_variable`");
    },
    other => panic!("expected a backtick code errror, but got: {}", other),
  }
}

#[test]
fn export_failure() {
  let text = r#"
export foo = "a"
baz = "c"
export bar = "b"
export abc = foo + bar + baz

wut:
  echo $foo $bar $baz
"#;

  let options = RunOptions {
    quiet: true,
    ..Default::default()
  };

  match parse_success(text).run(&["wut"], &options).unwrap_err() {
    RunError::Code{code: _, recipe} => {
      assert_eq!(recipe, "wut");
    },
    other => panic!("expected a recipe code errror, but got: {}", other),
  }
}

#[test]
fn readme_test() {
  let mut justfiles = vec![];
  let mut current = None;
 
  for line in brev::slurp("README.md").lines() {
    if let Some(mut justfile) = current {
      if line == "```" {
        justfiles.push(justfile);
        current = None;
      } else {
        justfile += line;
        justfile += "\n";
        current = Some(justfile);
      }
    } else if line == "```make" {
      current = Some(String::new());
    }
  }

  for justfile in justfiles {
    parse_success(&justfile);
  }
}
