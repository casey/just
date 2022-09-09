use super::{Ast, Item, Name, Set, Setting, Token, TokenKind};
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

fn kind_lexeme<'src>(
  token_kind: TokenKind,
  lexeme: &'src str,
) -> impl Parser<Token<'src>, Token<'src>, Error = Simple<Token<'src>>> {
  filter(move |tok: &Token| tok.kind == token_kind && tok.lexeme() == lexeme)
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
  choice((parse_setting().map(Some), parse_eol()))
}

fn parse_setting<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
  kind_lexeme(TokenKind::Identifier, "set")
    .map(|token| {
      assert_eq!(token.kind, TokenKind::Identifier);
      Name::from_identifier(token)
    })
    .then_ignore(kind(TokenKind::Whitespace))
    .then(parse_setting_name())
    .then_ignore(kind(TokenKind::Whitespace).ignored().or_not().then(kind(TokenKind::Eol))) // TODO if this were instead .then_ignore(parse_eol()) it
                                       // would be possible to support comments at the end of a
                                       // line
    .map(|(name, value)| Item::Set(Set { name, value }))
}

fn parse_set_expr<'src, T>(
  parser: impl Parser<Token<'src>, T, Error = Simple<Token<'src>>>,
) -> impl Parser<Token<'src>, T, Error = Simple<Token<'src>>> {
  kind(TokenKind::ColonEquals)
    .padded_by(filter(|tok: &Token<'_>| tok.kind == TokenKind::Whitespace))
    .ignore_then(parser)
}

fn parse_set_bool<'src>() -> impl Parser<Token<'src>, bool, Error = Simple<Token<'src>>> {
  // (kind(TokenKind::ColonEquals)
  //   .padded_by(filter(|tok: &Token<'_>| tok.kind == TokenKind::Whitespace))
  //   .ignore_then(
  //
  let true_or_false = kind_lexeme(TokenKind::Identifier, "true")
    .to(true)
    .or(kind_lexeme(TokenKind::Identifier, "false").to(false));
  parse_set_expr(true_or_false)
    .or_not()
    .map(|maybe_bool| maybe_bool.unwrap_or(true))
}

fn parse_setting_name<'src>() -> impl Parser<Token<'src>, Setting<'src>, Error = Simple<Token<'src>>>
{
  choice((
    kind_lexeme(TokenKind::Identifier, "allow-duplicate-recipes")
      .ignore_then(parse_set_bool())
      .map(|value| Setting::AllowDuplicateRecipes(value)),
    kind_lexeme(TokenKind::Identifier, "dotenv-load")
      .ignore_then(parse_set_bool())
      .map(|value| Setting::DotenvLoad(value)),
    kind_lexeme(TokenKind::Identifier, "export")
      .ignore_then(parse_set_bool())
      .map(|value| Setting::Export(value)),
    kind_lexeme(TokenKind::Identifier, "positional-arguments")
      .ignore_then(parse_set_bool())
      .map(|value| Setting::PositionalArguments(value)),
    kind_lexeme(TokenKind::Identifier, "windows-powershell")
      .ignore_then(parse_set_bool())
      .map(|value| Setting::WindowsPowerShell(value)),
  ))
}

// fn parse_alias<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
//   kind_lexeme(TokenKind::Identifier, "alias")

// }

fn parse_eol<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  parse_comment().or_not().then_ignore(kind(TokenKind::Eol))
}

fn parse_comment<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
  kind(TokenKind::Comment).map(|tok| Item::Comment(tok.lexeme()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Lexer;

  fn debug_tokens<'a>(tokens: Vec<Token<'a>>) {
    for item in tokens.iter() {
      println!("{} {}", item.kind, item.lexeme());
    }
  }

  #[test]
  fn new_parser_test3() {
    let src = "alias b := build";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = parse_ast().parse(tokens).unwrap();

  }

  #[test]
  fn new_parser_test2() {
    let src = "set dotenv-load   \nset windows-powershell := false\n# some stuff";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = parse_ast().parse(tokens).unwrap();
    assert_matches!(
      &ast.items[0],
      Item::Set(Set {
        name: _,
        value: Setting::DotenvLoad(true)
      })
    );
    assert_matches!(
      &ast.items[1],
      Item::Set(Set {
        name: _,
        value: Setting::WindowsPowerShell(false)
      })
    );
    // assert_matches!(&ast.items[2], Item::Comment("# some stuff"));
  }

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
