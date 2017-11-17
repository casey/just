use regex::Regex;

use token::Token;
use TokenKind::*;
use compilation_error::{CompilationError, CompilationErrorKind};

fn re(pattern: &str) -> Regex {
  Regex::new(pattern).unwrap()
}

fn token(pattern: &str) -> Regex {
  let mut s = String::new();
  s += r"^(?m)([ \t]*)(";
  s += pattern;
  s += ")";
  re(&s)
}

fn mixed_whitespace(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

pub fn tokenize(text: &str) -> Result<Vec<Token>, CompilationError> {
  lazy_static! {
    static ref BACKTICK:                  Regex = token(r"`[^`\n\r]*`"               );
    static ref COLON:                     Regex = token(r":"                         );
    static ref AT:                        Regex = token(r"@"                         );
    static ref COMMENT:                   Regex = token(r"#([^!\n\r].*)?$"           );
    static ref EOF:                       Regex = token(r"(?-m)$"                    );
    static ref EOL:                       Regex = token(r"\n|\r\n"                   );
    static ref EQUALS:                    Regex = token(r"="                         );
    static ref INTERPOLATION_END:         Regex = token(r"[}][}]"                    );
    static ref INTERPOLATION_START_TOKEN: Regex = token(r"[{][{]"                    );
    static ref NAME:                      Regex = token(r"([a-zA-Z_][a-zA-Z0-9_-]*)" );
    static ref PLUS:                      Regex = token(r"[+]"                       );
    static ref STRING:                    Regex = token("\""                         );
    static ref RAW_STRING:                Regex = token(r#"'[^']*'"#                 );
    static ref UNTERMINATED_RAW_STRING:   Regex = token(r#"'[^']*"#                  );
    static ref INDENT:                    Regex = re(r"^([ \t]*)[^ \t\n\r]"     );
    static ref INTERPOLATION_START:       Regex = re(r"^[{][{]"                 );
    static ref LEADING_TEXT:              Regex = re(r"^(?m)(.+?)[{][{]"        );
    static ref LINE:                      Regex = re(r"^(?m)[ \t]+[^ \t\n\r].*$");
    static ref TEXT:                      Regex = re(r"^(?m)(.+)"               );
  }

  #[derive(PartialEq)]
  enum State<'a> {
    Start,
    Indent(&'a str),
    Text,
    Interpolation,
  }

  fn indentation(text: &str) -> Option<&str> {
    INDENT.captures(text).map(|captures| captures.get(1).unwrap().as_str())
  }

  let mut tokens = vec![];
  let mut rest   = text;
  let mut index  = 0;
  let mut line   = 0;
  let mut column = 0;
  let mut state  = vec![State::Start];

  macro_rules! error {
    ($kind:expr) => {{
      Err(CompilationError {
        text:   text,
        index:  index,
        line:   line,
        column: column,
        width:  None,
        kind:   $kind,
      })
    }};
  }

  loop {
    if column == 0 {
      if let Some(kind) = match (state.last().unwrap(), indentation(rest)) {
        // ignore: was no indentation and there still isn't
        //         or current line is blank
        (&State::Start, Some("")) | (_, None) => {
          None
        }
        // indent: was no indentation, now there is
        (&State::Start, Some(current)) => {
          if mixed_whitespace(current) {
            return error!(CompilationErrorKind::MixedLeadingWhitespace{whitespace: current})
          }
          //indent = Some(current);
          state.push(State::Indent(current));
          Some(Indent)
        }
        // dedent: there was indentation and now there isn't
        (&State::Indent(_), Some("")) => {
          // indent = None;
          state.pop();
          Some(Dedent)
        }
        // was indentation and still is, check if the new indentation matches
        (&State::Indent(previous), Some(current)) => {
          if !current.starts_with(previous) {
            return error!(CompilationErrorKind::InconsistentLeadingWhitespace{
              expected: previous,
              found: current
            });
          }
          None
        }
        // at column 0 in some other state: this should never happen
        (&State::Text, _) | (&State::Interpolation, _) => {
          return error!(CompilationErrorKind::InternalError{
            message: "unexpected state at column 0".to_string()
          });
        }
      } {
        tokens.push(Token {
          index:  index,
          line:   line,
          column: column,
          text:   text,
          prefix: "",
          lexeme: "",
          kind:   kind,
        });
      }
    }

    // insert a dedent if we're indented and we hit the end of the file
    if &State::Start != state.last().unwrap() && EOF.is_match(rest) {
      tokens.push(Token {
        index:  index,
        line:   line,
        column: column,
        text:   text,
        prefix: "",
        lexeme: "",
        kind:   Dedent,
      });
    }

    let (prefix, lexeme, kind) =
    if let (0, &State::Indent(indent), Some(captures)) =
      (column, state.last().unwrap(), LINE.captures(rest)) {
      let line = captures.get(0).unwrap().as_str();
      if !line.starts_with(indent) {
        return error!(CompilationErrorKind::InternalError{message: "unexpected indent".to_string()});
      }
      state.push(State::Text);
      (&line[0..indent.len()], "", Line)
    } else if let Some(captures) = EOF.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eof)
    } else if let State::Text = *state.last().unwrap() {
      if let Some(captures) = INTERPOLATION_START.captures(rest) {
        state.push(State::Interpolation);
        ("", captures.get(0).unwrap().as_str(), InterpolationStart)
      } else if let Some(captures) = LEADING_TEXT.captures(rest) {
        ("", captures.get(1).unwrap().as_str(), Text)
      } else if let Some(captures) = TEXT.captures(rest) {
        ("", captures.get(1).unwrap().as_str(), Text)
      } else if let Some(captures) = EOL.captures(rest) {
        state.pop();
        (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eol)
      } else {
        return error!(CompilationErrorKind::InternalError{
          message: format!("Could not match token in text state: \"{}\"", rest)
        });
      }
    } else if let Some(captures) = INTERPOLATION_START_TOKEN.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), InterpolationStart)
    } else if let Some(captures) = INTERPOLATION_END.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        state.pop();
      }
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), InterpolationEnd)
    } else if let Some(captures) = NAME.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Name)
    } else if let Some(captures) = EOL.captures(rest) {
      if state.last().unwrap() == &State::Interpolation {
        return error!(CompilationErrorKind::InternalError {
          message: "hit EOL while still in interpolation state".to_string()
        });
      }
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Eol)
    } else if let Some(captures) = BACKTICK.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Backtick)
    } else if let Some(captures) = COLON.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Colon)
    } else if let Some(captures) = AT.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), At)
    } else if let Some(captures) = PLUS.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Plus)
    } else if let Some(captures) = EQUALS.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Equals)
    } else if let Some(captures) = COMMENT.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), Comment)
    } else if let Some(captures) = RAW_STRING.captures(rest) {
      (captures.get(1).unwrap().as_str(), captures.get(2).unwrap().as_str(), RawString)
    } else if UNTERMINATED_RAW_STRING.is_match(rest) {
      return error!(CompilationErrorKind::UnterminatedString);
    } else if let Some(captures) = STRING.captures(rest) {
      let prefix = captures.get(1).unwrap().as_str();
      let contents = &rest[prefix.len()+1..];
      if contents.is_empty() {
        return error!(CompilationErrorKind::UnterminatedString);
      }
      let mut len = 0;
      let mut escape = false;
      for c in contents.chars() {
        if c == '\n' || c == '\r' {
          return error!(CompilationErrorKind::UnterminatedString);
        } else if !escape && c == '"' {
          break;
        } else if !escape && c == '\\' {
          escape = true;
        } else if escape {
          escape = false;
        }
        len += c.len_utf8();
      }
      let start = prefix.len();
      let content_end = start + len + 1;
      if escape || content_end >= rest.len() {
        return error!(CompilationErrorKind::UnterminatedString);
      }
      (prefix, &rest[start..content_end + 1], StringToken)
    } else if rest.starts_with("#!") {
      return error!(CompilationErrorKind::OuterShebang)
    } else {
      return error!(CompilationErrorKind::UnknownStartOfToken)
    };

    tokens.push(Token {
      index:  index,
      line:   line,
      column: column,
      prefix: prefix,
      text:   text,
      lexeme: lexeme,
      kind:   kind,
    });

    let len = prefix.len() + lexeme.len();

    if len == 0 {
      let last = tokens.last().unwrap();
      match last.kind {
        Eof => {},
        _ => return Err(last.error(CompilationErrorKind::InternalError{
          message: format!("zero length token: {:?}", last)
        })),
      }
    }

    match tokens.last().unwrap().kind {
      Eol => {
        line += 1;
        column = 0;
      }
      Eof => {
        break;
      }
      RawString => {
        let lexeme_lines = lexeme.lines().count();
        line += lexeme_lines - 1;
        if lexeme_lines == 1 {
          column += len;
        } else {
          column = lexeme.lines().last().unwrap().len();
        }
      }
      _ => {
        column += len;
      }
    }

    rest = &rest[len..];
    index += len;
  }

  Ok(tokens)
}

#[cfg(test)]
mod test {  
  use super::*;
  use testing::parse_error;

  fn tokenize_success(text: &str, expected_summary: &str) {
    let tokens = tokenize(text).unwrap();
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

  fn tokenize_error(text: &str, expected: CompilationError) {
    if let Err(error) = tokenize(text) {
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
fn tokenize_comment_before_variable() {
  let text = "
#
A='1'
echo:
  echo {{A}}
  ";
  tokenize_success(text, "$#$N='$N:$>^_{N}$<.");
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
  tokenize_error(text, CompilationError {
    text:   text,
    index:  9,
    line:   3,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::InconsistentLeadingWhitespace{expected: " ", found: "\t"},
  });
}

#[test]
fn tokenize_tabs_then_tab_space() {
  let text = "a:
\t\t0
\t\t 1
\t  2
";
  tokenize_error(text, CompilationError {
    text:   text,
    index:  12,
    line:   3,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::InconsistentLeadingWhitespace{expected: "\t\t", found: "\t  "},
  });
}

#[test]
fn tokenize_outer_shebang() {
  let text = "#!/usr/bin/env bash";
  tokenize_error(text, CompilationError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::OuterShebang
  });
}

#[test]
fn tokenize_unknown() {
  let text = "~";
  tokenize_error(text, CompilationError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::UnknownStartOfToken
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
fn unterminated_string() {
  let text = r#"a = ""#;
  parse_error(text, CompilationError {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  None,
    kind:   CompilationErrorKind::UnterminatedString,
  });
}

#[test]
fn unterminated_string_with_escapes() {
  let text = r#"a = "\n\t\r\"\\"#;
  parse_error(text, CompilationError {
    text:   text,
    index:  3,
    line:   0,
    column: 3,
    width:  None,
    kind:   CompilationErrorKind::UnterminatedString,
  });
}
#[test]
fn unterminated_raw_string() {
  let text = "r a='asdf";
  parse_error(text, CompilationError {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  None,
    kind:   CompilationErrorKind::UnterminatedString,
  });
}


#[test]
fn mixed_leading_whitespace() {
  let text = "a:\n\t echo hello";
  parse_error(text, CompilationError {
    text:   text,
    index:  3,
    line:   1,
    column: 0,
    width:  None,
    kind:   CompilationErrorKind::MixedLeadingWhitespace{whitespace: "\t "}
  });
}

}
