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

fn kind<'src>(
  token_kind: TokenKind,
) -> impl Parser<Token<'src>, Token<'src>, Error = Simple<Token<'src>>> {
  filter(move |tok: &Token| tok.kind == token_kind)
}

fn parse_ast<'src>() -> impl Parser<Token<'src>, Ast<'src>, Error = Simple<Token<'src>>> {
  parse_items()
    .then(parse_eof())
    .map(|(mut items, maybe_final_comment)| {
      if let Some(comment) = maybe_final_comment {
        items.push(comment);
      }

      Ast {
        items,
        warnings: vec![],
      }
    })
}

fn parse_eof<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  parse_comment() // A comment might be the last thing in a file with no trailing newline
    .map(Some)
    .or(kind(TokenKind::Eof).map(|_| None)) //TODO the .end() parser makes an explicit Eof token
                                            //unnecessary
}

fn parse_items<'src>() -> impl Parser<Token<'src>, Vec<Item<'src>>, Error = Simple<Token<'src>>> {
  parse_item()
    .repeated()
    .map(|item_or_newline| item_or_newline.into_iter().flatten().collect())
}

fn parse_item<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  parse_eol()
}

fn parse_eol<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  (parse_comment().then_ignore(kind(TokenKind::Eol)).map(Some))
    .or(kind(TokenKind::Eol).map(|_| None))
}

fn parse_comment<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
  kind(TokenKind::Comment).map(|tok| Item::Comment(tok.lexeme()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Lexer;

  #[test]
  fn new_parser_test() {
    let src = "\n# some stuff";
    let tokens = Lexer::lex(src).unwrap();
    let ast = parse_ast().parse(tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("# some stuff"));

    let src = "\n# some stuff\n";
    let tokens = Lexer::lex(src).unwrap();
    let ast = parse_ast().parse(tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("# some stuff"));

    let src = "#bongo\n#crayfis\n\n\n# some stuff\n";
    let tokens = Lexer::lex(src).unwrap();
    let ast = parse_ast().parse(tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("#bongo"));
    assert_matches!(&ast.items[2], Item::Comment("# some stuff"));
  }
}
