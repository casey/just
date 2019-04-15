use crate::common::*;

use regex::Regex;
use CompilationErrorKind::*;
use TokenKind::*;

fn re(pattern: &str) -> Regex {
  Regex::new(pattern).unwrap()
}

fn token(pattern: &str) -> Regex {
  re(&format!(r"^(?m)([ \t]*)({})", pattern))
}

fn mixed_whitespace(text: &str) -> bool {
  !(text.chars().all(|c| c == ' ') || text.chars().all(|c| c == '\t'))
}

pub struct Lexer<'a> {
  tokens: Vec<Token<'a>>,
  text: &'a str,
  rest: &'a str,
  offset: usize,
  column: usize,
  line: usize,
  state: Vec<State<'a>>,
}

impl<'a> Lexer<'a> {
  pub fn lex(text: &'a str) -> CompilationResult<Vec<Token<'a>>> {
    /*
    let lexer = Lexer {
      tokens: vec![],
      rest: text,
      offset: 0,
      line: 0,
      column: 0,
      state: vec![State::Normal],
      text,
    };
    */

    // let old = lexer.inner()?;

    let new = crate::new_lexer::lex(text)?;

    Ok(new)
  }

  fn error(&self, kind: CompilationErrorKind<'a>) -> CompilationError<'a> {
    CompilationError {
      text: self.text,
      offset: self.offset,
      line: self.line,
      column: self.column,
      width: None,
      kind,
    }
  }

  fn token(&self, length: usize, kind: TokenKind) -> Token<'a> {
    Token {
      offset: self.offset,
      line: self.line,
      column: self.column,
      text: self.text,
      length,
      kind,
    }
  }

  fn lex_indent(&mut self) -> CompilationResult<'a, Option<Token<'a>>> {
    lazy_static! {
      static ref INDENT: Regex = re(r"^([ \t]*)[^ \t\n\r]");
    }

    let indentation = INDENT
      .captures(self.rest)
      .map(|captures| captures.get(1).unwrap().as_str());

    if self.column == 0 {
      if let Some(kind) = match (self.state.last().unwrap(), indentation) {
        // ignore: was no indentation and there still isn't
        //         or current line is blank
        (&State::Normal, Some("")) | (_, None) => None,
        // indent: was no indentation, now there is
        (&State::Normal, Some(indentation)) => {
          if mixed_whitespace(indentation) {
            return Err(self.error(MixedLeadingWhitespace {
              whitespace: indentation,
            }));
          }
          self.state.push(State::Indented { indentation });
          Some(Indent)
        }
        // dedent: there was indentation and now there isn't
        (&State::Indented { .. }, Some("")) => {
          // indent = None;
          self.state.pop();
          Some(Dedent)
        }
        // was indentation and still is, check if the new indentation matches
        (&State::Indented { indentation }, Some(current)) => {
          if !current.starts_with(indentation) {
            return Err(self.error(InconsistentLeadingWhitespace {
              expected: indentation,
              found: current,
            }));
          }
          None
        }
        // at column 0 in some other state: this should never happen
        (&State::Text, _) | (&State::Interpolation, _) => {
          return Err(self.error(Internal {
            message: "unexpected state at column 0".to_string(),
          }));
        }
      } {
        return Ok(Some(self.token(0, kind)));
      }
    }
    Ok(None)
  }

  pub fn inner(mut self) -> CompilationResult<'a, Vec<Token<'a>>> {
    lazy_static! {
      static ref AT: Regex = token(r"@");
      static ref BACKTICK: Regex = token(r"`[^`\n\r]*`");
      static ref COLON: Regex = token(r":");
      static ref COMMA: Regex = token(r",");
      static ref COMMENT: Regex = token(r"#([^\n\r][^\n\r]*)?\r?$");
      static ref EOF: Regex = token(r"\z");
      static ref EOL: Regex = token(r"\n|\r\n");
      static ref EQUALS: Regex = token(r"=");
      static ref INTERPOLATION_END: Regex = token(r"[}][}]");
      static ref INTERPOLATION_START_TOKEN: Regex = token(r"[{][{]");
      static ref NAME: Regex = token(r"([a-zA-Z_][a-zA-Z0-9_-]*)");
      static ref PAREN_L: Regex = token(r"[(]");
      static ref PAREN_R: Regex = token(r"[)]");
      static ref PLUS: Regex = token(r"[+]");
      static ref RAW_STRING: Regex = token(r#"'[^']*'"#);
      static ref STRING: Regex = token(r#"["]"#);
      static ref UNTERMINATED_RAW_STRING: Regex = token(r#"'[^']*"#);
      static ref INTERPOLATION_START: Regex = re(r"^[{][{]");
      static ref LEADING_TEXT: Regex = re(r"^(?m)(.+?)[{][{]");
      static ref LINE: Regex = re(r"^(?m)[ \t]+[^ \t\n\r].*$");
      static ref TEXT: Regex = re(r"^(?m)(.+)");
    }

    loop {
      if let Some(token) = self.lex_indent()? {
        self.tokens.push(token);
      }

      // insert a dedent if we're indented and we hit the end of the file
      if &State::Normal != self.state.last().unwrap() && EOF.is_match(self.rest) {
        let token = self.token(0, Dedent);
        self.tokens.push(token);
      }

      let (whitespace, lexeme, kind) =
        if let (0, &State::Indented { indentation }, Some(captures)) = (
          self.column,
          self.state.last().unwrap(),
          LINE.captures(self.rest),
        ) {
          let line = captures.get(0).unwrap().as_str();
          if !line.starts_with(indentation) {
            return Err(self.error(Internal {
              message: "unexpected indent".to_string(),
            }));
          }
          self.state.push(State::Text);
          (&line[0..indentation.len()], "", Line)
        } else if let Some(captures) = EOF.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Eof,
          )
        } else if let State::Text = *self.state.last().unwrap() {
          if let Some(captures) = INTERPOLATION_START.captures(self.rest) {
            self.state.push(State::Interpolation);
            ("", captures.get(0).unwrap().as_str(), InterpolationStart)
          } else if let Some(captures) = LEADING_TEXT.captures(self.rest) {
            ("", captures.get(1).unwrap().as_str(), Text)
          } else if let Some(captures) = TEXT.captures(self.rest) {
            ("", captures.get(1).unwrap().as_str(), Text)
          } else if let Some(captures) = EOL.captures(self.rest) {
            self.state.pop();
            (
              captures.get(1).unwrap().as_str(),
              captures.get(2).unwrap().as_str(),
              Eol,
            )
          } else {
            return Err(self.error(Internal {
              message: format!("Could not match token in text state: \"{}\"", self.rest),
            }));
          }
        } else if let Some(captures) = INTERPOLATION_START_TOKEN.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            InterpolationStart,
          )
        } else if let Some(captures) = INTERPOLATION_END.captures(self.rest) {
          if self.state.last().unwrap() == &State::Interpolation {
            self.state.pop();
          }
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            InterpolationEnd,
          )
        } else if let Some(captures) = NAME.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Name,
          )
        } else if let Some(captures) = EOL.captures(self.rest) {
          if self.state.last().unwrap() == &State::Interpolation {
            return Err(self.error(UnterminatedInterpolation));
          }
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Eol,
          )
        } else if let Some(captures) = BACKTICK.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Backtick,
          )
        } else if let Some(captures) = COLON.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Colon,
          )
        } else if let Some(captures) = AT.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            At,
          )
        } else if let Some(captures) = COMMA.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Comma,
          )
        } else if let Some(captures) = PAREN_L.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            ParenL,
          )
        } else if let Some(captures) = PAREN_R.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            ParenR,
          )
        } else if let Some(captures) = PLUS.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Plus,
          )
        } else if let Some(captures) = EQUALS.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Equals,
          )
        } else if let Some(captures) = COMMENT.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            Comment,
          )
        } else if let Some(captures) = RAW_STRING.captures(self.rest) {
          (
            captures.get(1).unwrap().as_str(),
            captures.get(2).unwrap().as_str(),
            StringRaw,
          )
        } else if UNTERMINATED_RAW_STRING.is_match(self.rest) {
          return Err(self.error(UnterminatedString));
        } else if let Some(captures) = STRING.captures(self.rest) {
          let whitespace = captures.get(1).unwrap().as_str();
          let contents = &self.rest[whitespace.len() + 1..];
          if contents.is_empty() {
            return Err(self.error(UnterminatedString));
          }
          let mut len = 0;
          let mut escape = false;
          for c in contents.chars() {
            if c == '\n' || c == '\r' {
              return Err(self.error(UnterminatedString));
            } else if !escape && c == '"' {
              break;
            } else if !escape && c == '\\' {
              escape = true;
            } else if escape {
              escape = false;
            }
            len += c.len_utf8();
          }
          let start = whitespace.len();
          let content_end = start + len + 1;
          if escape || content_end >= self.rest.len() {
            return Err(self.error(UnterminatedString));
          }
          (whitespace, &self.rest[start..=content_end], StringCooked)
        } else {
          return Err(self.error(UnknownStartOfToken));
        };

      if whitespace.len() > 0 {
        self.tokens.push(self.token(whitespace.len(), Whitespace));
        self.column += whitespace.len();
        self.offset += whitespace.len();
      }

      let token = self.token(lexeme.len(), kind);
      self.tokens.push(token);

      let len = whitespace.len() + lexeme.len();

      if len == 0 {
        let last = self.tokens.last().unwrap();
        match last.kind {
          Eof => {}
          _ => {
            return Err(last.error(Internal {
              message: format!("zero length token: {:?}", last),
            }));
          }
        }
      }

      match self.tokens.last().unwrap().kind {
        Eol => {
          self.line += 1;
          self.column = 0;
        }
        Eof => {
          break;
        }
        StringRaw => {
          let lexeme_lines = lexeme.lines().count();
          self.line += lexeme_lines - 1;
          if lexeme_lines == 1 {
            self.column += lexeme.len();
          } else {
            self.column = lexeme.lines().last().unwrap().len();
          }
        }
        _ => {
          self.column += lexeme.len();
        }
      }

      self.rest = &self.rest[len..];
      self.offset += lexeme.len();
    }

    Ok(self.tokens)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use crate::testing::token_summary;

  macro_rules! summary_test {
    ($name:ident, $input:expr, $expected:expr $(,)*) => {
      #[test]
      fn $name() {
        let input = $input;
        let expected = $expected;
        let tokens = crate::lexer::Lexer::lex(input).unwrap();
        let roundtrip = tokens
          .iter()
          .map(Token::lexeme)
          .collect::<Vec<&str>>()
          .join("");
        let actual = token_summary(&tokens);
        if actual != expected {
          panic!(
            "token summary mismatch:\nexpected: {}\ngot:      {}\n",
            expected, actual
          );
        }
        assert_eq!(input, roundtrip);
      }
    };
  }

  macro_rules! error_test {
    (
      name:     $name:ident,
      input:    $input:expr,
      offset:   $offset:expr,
      line:     $line:expr,
      column:   $column:expr,
      width:    $width:expr,
      kind:     $kind:expr,
    ) => {
      #[test]
      fn $name() {
        let input = $input;

        let expected = CompilationError {
          text: input,
          offset: $offset,
          line: $line,
          column: $column,
          width: $width,
          kind: $kind,
        };

        if let Err(error) = Lexer::lex(input) {
          assert_eq!(error.text, expected.text);
          assert_eq!(error.offset, expected.offset);
          assert_eq!(error.line, expected.line);
          assert_eq!(error.column, expected.column);
          assert_eq!(error.kind, expected.kind);
          assert_eq!(error, expected);
        } else {
          panic!("tokenize succeeded but expected: {}\n{}", expected, input);
        }
      }
    };
  }

  summary_test! {
    tokenize_names,
    "\
foo
bar-bob
b-bob_asdfAAAA
test123",
    "N$N$N$N.",
  }

  summary_test! {
    tokenize_indented_line,
    "foo:\n a",
    "N:$>^_<.",
  }

  summary_test! {
    tokenize_indented_block,
    r##"foo:
  a
  b
  c
"##,
    "N:$>^_$ ^_$ ^_$<.",
  }

  summary_test! {
    tokenize_strings,
    r#"a = "'a'" + '"b"' + "'c'" + '"d"'#echo hello"#,
    r#"N = " + ' + " + '#."#,
  }

  summary_test! {
    tokenize_recipe_interpolation_eol,
    "foo: # some comment
 {{hello}}
",
    "N: #$>^{N}$<.",
  }

  summary_test! {
    tokenize_recipe_interpolation_eof,
    "foo: # more comments
 {{hello}}
# another comment
",
    "N: #$>^{N}$<#$.",
  }

  summary_test! {
    tokenize_recipe_complex_interpolation_expression,
    "foo: #lol\n {{a + b + \"z\" + blarg}}",
    "N: #$>^{N + N + \" + N}<.",
  }

  summary_test! {
    tokenize_recipe_multiple_interpolations,
    "foo:,#ok\n {{a}}0{{b}}1{{c}}",
    "N:,#$>^{N}_{N}_{N}<.",
  }

  summary_test! {
    tokenize_junk,
    "bob

hello blah blah blah : a b c #whatever
    ",
    "N$$N N N N : N N N #$ .",
  }

  summary_test! {
    tokenize_empty_lines,
    "
# this does something
hello:
  asdf
  bsdf

  csdf

  dsdf # whatever

# yolo
  ",
    "$#$N:$>^_$ ^_$^$ ^_$^$ ^_$^$<#$ .",
  }

  summary_test! {
    tokenize_comment_before_variable,
    "
#
A='1'
echo:
  echo {{A}}
  ",
    "$#$N='$N:$>^_{N}$ <.",
  }

  summary_test! {
    tokenize_interpolation_backticks,
    "hello:\n echo {{`echo hello` + `echo goodbye`}}",
    "N:$>^_{` + `}<.",
  }

  summary_test! {
    tokenize_empty_interpolation,
    "hello:\n echo {{}}",
    "N:$>^_{}<.",
  }

  summary_test! {
    tokenize_assignment_backticks,
    "a = `echo hello` + `echo goodbye`",
    "N = ` + `.",
  }

  summary_test! {
    tokenize_multiple,
    "
hello:
  a
  b

  c

  d

# hello
bob:
  frank
 \t",

    "$N:$>^_$ ^_$^$ ^_$^$ ^_$^$<#$N:$>^_$ <.",
  }

  summary_test! {
    tokenize_comment,
    "a:=#",
    "N:=#."
  }

  summary_test! {
    tokenize_comment_with_bang,
    "a:=#foo!",
    "N:=#."
  }

  summary_test! {
    tokenize_order,
    r"
b: a
  @mv a b

a:
  @touch F
  @touch a

d: c
  @rm c

c: b
  @mv b c",
    "$N: N$>^_$^$<N:$>^_$ ^_$^$<N: N$>^_$^$<N: N$>^_<.",
  }

  summary_test! {
    tokenize_parens,
    r"((())) )abc(+",
    "((())) )N(+.",
  }

  summary_test! {
    crlf_newline,
    "#\r\n#asdf\r\n",
    "#$#$.",
  }

  summary_test! {
    multiple_recipes,
    "a:\n  foo\nb:",
    "N:$>^_$<N:.",
  }

  error_test! {
    name:  tokenize_space_then_tab,
    input: "a:
 0
 1
\t2
",
    offset:  9,
    line:   3,
    column: 0,
    width:  Some(1),
    kind:   InconsistentLeadingWhitespace{expected: " ", found: "\t"},
  }

  error_test! {
    name:  tokenize_tabs_then_tab_space,
    input: "a:
\t\t0
\t\t 1
\t  2
",
    offset:  12,
    line:   3,
    column: 0,
    width:  Some(2),
    kind:   InconsistentLeadingWhitespace{expected: "\t\t", found: "\t "},
  }

  error_test! {
    name:   tokenize_unknown,
    input:  "~",
    offset: 0,
    line:   0,
    column: 0,
    width:  Some(1),
    kind:   UnknownStartOfToken,
  }

  error_test! {
    name:   unterminated_string,
    input:  r#"a = ""#,
    offset: 4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   UnterminatedString,
  }

  error_test! {
    name:   unterminated_string_with_escapes,
    input:  r#"a = "\n\t\r\"\\"#,
    offset: 4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   UnterminatedString,
  }

  error_test! {
    name:   unterminated_raw_string,
    input:  "r a='asdf",
    offset: 4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   UnterminatedString,
  }

  error_test! {
    name:   unterminated_interpolation,
    input:  "foo:\n echo {{
  ",
    offset: 13,
    line:   1,
    column: 8,
    width:  Some(2),
    kind:   UnterminatedInterpolation,
  }

  error_test! {
    name:   mixed_leading_whitespace,
    input:  "a:\n\t echo hello",
    offset: 3,
    line:   1,
    column: 0,
    width:  Some(2),
    kind:   MixedLeadingWhitespace{whitespace: "\t "},
  }
}
