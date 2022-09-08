use super::{Ast, Item, Token, TokenKind};
use chumsky::prelude::*;

/// New parser
/// This uses the chumsky library to do parsing
pub(crate) struct NewParser<'tokens, 'src> {
  tokens: &'tokens [Token<'src>],
}

impl<'tokens, 'src> NewParser<'tokens, 'src> {
  fn new(tokens: &'tokens [Token<'src>]) -> NewParser<'tokens, 'src> {
    NewParser { tokens }
  }

  pub(crate) fn parse(tokens: &'tokens [Token<'src>]) -> Result<Ast<'src>, ()> {
    Self::new(tokens).parse_ast()
  }

  /// Parse a justfile, consumes self
  fn parse_ast(mut self) -> Result<Ast<'src>, ()> {
    Err(())
  }
}

fn parse_ast<'src>() -> impl Parser<Token<'src>, Ast<'src>, Error = Simple<Token<'src>>> {
  filter(|tok: &Token| matches!(tok.kind, TokenKind::Comment)).map(|tok| Ast {
    items: vec![Item::Comment(tok.lexeme())],
    warnings: vec![],
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Lexer;

  #[test]
  fn new_parser_test() {
    let src = "# some stuff";
    let tokens = Lexer::lex(src).unwrap();
    println!("TOK: {:?}", tokens);
    let ast = parse_ast().parse(tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("# some stuff"));
  }
}
