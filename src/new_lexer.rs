use crate::common::*;

pub struct NewLexer<'a> {
  text: &'a str,
  tokens: Vec<Token<'a>>,
  state: Vec<State<'a>>,
  offset: usize,
  column: usize,
  line: usize,
}

impl<'a> NewLexer<'a> {
  pub fn new(text: &'a str) -> NewLexer<'a> {
    NewLexer {
      state: vec![State::Start],
      tokens: Vec::new(),
      offset: 0,
      column: 0,
      line: 0,
      text,
    }
  }

  pub fn lex(self) -> CompilationResult<'a, Vec<Token<'a>>> {
    Ok(self.tokens)
  }

  // pub fn token(&mut self, length: usize, token_kind: TokenKind) {
  //   self.tokens.push( Token {
  //     offset: self.offset,
  //     column:
  //   });
  // }
}

// #[derive(Debug, PartialEq, Clone)]
// pub struct Token<'a> {
//   pub index: usize,
//   pub length: usize,
//   pub line: usize,
//   pub column: usize,
//   pub text: &'a str,
//   pub kind: TokenKind,
// }
