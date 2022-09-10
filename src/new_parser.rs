#![allow(dead_code)]
use super::{
  Alias, Assignment, Ast, Expression, Item, Name, Set, Setting, Thunk, Token, TokenKind,
};
use chumsky::prelude::*;

// TODO - maybe we don't even need the NewParser struct to contain tokens?
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
    let p = Self::new(tokens);
    p.ast_parser().parse(tokens).map_err(|_ignoring| ())
  }

  fn ast_parser<'a>(&self) -> impl JustParser<'a, Ast<'a>> {
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
}

trait JustParser<'src, T>: Parser<Token<'src>, T, Error = Simple<Token<'src>>> {}

impl<'src, T, U> JustParser<'src, T> for U where
  U: Parser<Token<'src>, T, Error = Simple<Token<'src>>>
{
}

fn kind<'src>(token_kind: TokenKind) -> impl JustParser<'src, Token<'src>> {
  filter(move |tok: &Token| tok.kind == token_kind)
}

fn kind_lexeme<'src>(token_kind: TokenKind, lexeme: &'src str) -> impl JustParser<Token<'src>> {
  filter(move |tok: &Token| tok.kind == token_kind && tok.lexeme() == lexeme)
}

fn parse_expression<'src>() -> impl JustParser<'src, Expression<'src>> {
  parse_value()
}

fn parse_value<'src>() -> impl JustParser<'src, Expression<'src>> {
  choice((
    parse_name().map(|name| Expression::Variable { name }),
    parse_call().map(|thunk| Expression::Call { thunk }),
  ))
}

fn parse_call<'src>() -> impl JustParser<'src, Thunk<'src>> {
  todo()
}

fn parse_name<'src>() -> impl JustParser<'src, Name<'src>> {
  kind(TokenKind::Identifier).map(Name::from_identifier)
}

fn parse_eof<'src>() -> impl JustParser<'src, Option<Item<'src>>> {
  parse_comment() // A comment might be the last thing in a file with no trailing newline
    .map(Some)
    .or(kind(TokenKind::Eof).map(|_| None)) //TODO the .end() parser makes an explicit Eof token
                                            //unnecessary
}

fn parse_items<'src>() -> impl JustParser<'src, Vec<Item<'src>>> {
  parse_item()
    .repeated()
    .map(|item_or_newline| item_or_newline.into_iter().flatten().collect())
}

fn parse_item<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  // TODO if this were instead .then_ignore(parse_eol()) it would support comments at the end of
  // a line, after a legit item
  fn item_end<'src>() -> impl JustParser<'src, ()> {
    kind(TokenKind::Whitespace)
      .ignored()
      .or_not()
      .then(kind(TokenKind::Eol))
      .to(())
  }

  choice((
    parse_setting().then_ignore(item_end()).map(Some),
    parse_alias().then_ignore(item_end()).map(Some),
    parse_assignment().then_ignore(item_end()).map(Some),
    parse_eol(),
  ))
}

fn parse_assignment<'src>() -> impl JustParser<'src, Item<'src>> {
  (kind_lexeme(TokenKind::Identifier, "export").then_ignore(kind(TokenKind::Whitespace)))
    .or_not()
    .then(parse_name().then(parse_colon_equals(parse_expression())))
    .map(|(maybe_export, (name, value))| {
      Item::Assignment(Assignment {
        export: maybe_export.is_some(),
        name,
        value,
      })
    })
}

fn parse_setting<'src>() -> impl JustParser<'src, Item<'src>> {
  kind_lexeme(TokenKind::Identifier, "set")
    .map(|token| {
      assert_eq!(token.kind, TokenKind::Identifier);
      Name::from_identifier(token)
    })
    .then_ignore(kind(TokenKind::Whitespace))
    .then(parse_setting_name())
    .map(|(name, value)| Item::Set(Set { name, value }))
}

fn parse_colon_equals<'src, T>(parser: impl JustParser<'src, T>) -> impl JustParser<'src, T> {
  kind(TokenKind::ColonEquals)
    .padded_by(filter(|tok: &Token<'_>| tok.kind == TokenKind::Whitespace))
    .ignore_then(parser)
}

fn parse_set_bool<'src>() -> impl JustParser<'src, bool> {
  let true_or_false = kind_lexeme(TokenKind::Identifier, "true")
    .to(true)
    .or(kind_lexeme(TokenKind::Identifier, "false").to(false));
  parse_colon_equals(true_or_false)
    .or_not()
    .map(|maybe_bool| maybe_bool.unwrap_or(true))
}

fn parse_setting_name<'src>() -> impl JustParser<'src, Setting<'src>> {
  choice((
    kind_lexeme(TokenKind::Identifier, "allow-duplicate-recipes")
      .ignore_then(parse_set_bool())
      .map(Setting::AllowDuplicateRecipes),
    kind_lexeme(TokenKind::Identifier, "dotenv-load")
      .ignore_then(parse_set_bool())
      .map(Setting::DotenvLoad),
    kind_lexeme(TokenKind::Identifier, "export")
      .ignore_then(parse_set_bool())
      .map(Setting::Export),
    kind_lexeme(TokenKind::Identifier, "positional-arguments")
      .ignore_then(parse_set_bool())
      .map(Setting::PositionalArguments),
    kind_lexeme(TokenKind::Identifier, "windows-powershell")
      .ignore_then(parse_set_bool())
      .map(Setting::WindowsPowerShell),
  ))
}

fn parse_alias<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
  kind_lexeme(TokenKind::Identifier, "alias")
    .ignore_then(kind(TokenKind::Whitespace))
    .ignore_then(parse_name())
    .then(parse_colon_equals(parse_name()))
    .map(|(name, target)| Item::Alias(Alias { name, target }))
}

fn parse_eol<'src>() -> impl JustParser<'src, Option<Item<'src>>> {
  parse_comment().or_not().then_ignore(kind(TokenKind::Eol))
}

fn parse_comment<'src>() -> impl JustParser<'src, Item<'src>> {
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
    let src = "alias b := build\n";
    let tokens = Lexer::lex(src).unwrap();
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Alias(..))
  }

  #[test]
  fn new_parser_test2() {
    let src = "set dotenv-load   \nset windows-powershell := false\n# some stuff";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = NewParser::parse(&tokens).unwrap();
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
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("# some stuff"));

    let src = "\n# some stuff\n";
    let tokens = Lexer::lex(src).unwrap();
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("# some stuff"));

    let src = "#bongo\n#crayfis\n\n\n# some stuff\nexport tane := rabo\nrusi := kava\n";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Comment("#bongo"));
    assert_matches!(&ast.items[2], Item::Comment("# some stuff"));
    assert_matches!(
      &ast.items[3],
      Item::Assignment(Assignment { export: true, .. })
    );
    assert_matches!(
      &ast.items[4],
      Item::Assignment(Assignment {
        export: false,
        name: _,
        value: Expression::Variable { .. }
      })
    );
  }
}
