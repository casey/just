#![allow(dead_code)]
use super::{
  Alias, Assignment, Ast, CompileError, CompileErrorKind, ConditionalOperator, Expression,
  Fragment, Item, Line, Name, Recipe, Set, Setting, Shell, StringKind, StringLiteral, Thunk, Token,
  TokenKind,
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
    let (output, errs) = p.ast_parser().parse_recovery_verbose(tokens);
    for item in errs.iter() {
      println!("ERR: {:#?}", item);
    }
    if let Some(output) = output {
      Ok(output)
    } else {
      Err(())
    }
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

fn kind<'src>(token_kind: TokenKind) -> impl JustParser<'src, Token<'src>> + Clone {
  filter(move |tok: &Token| tok.kind == token_kind)
}

fn ws<'src>() -> impl JustParser<'src, ()> + Clone {
  kind(TokenKind::Whitespace).to(())
}

fn keyword<'src>(lexeme: &'src str) -> impl JustParser<Token<'src>> + Clone {
  filter(move |tok: &Token| tok.kind == TokenKind::Identifier && tok.lexeme() == lexeme)
}

fn parse_expression<'src>() -> impl JustParser<'src, Expression<'src>> {
  recursive(|parse_expression_rec| {
    let parse_group = parse_expression_rec
      .clone()
      .delimited_by(kind(TokenKind::ParenL), kind(TokenKind::ParenR))
      .map(Box::new);

    let parse_sequence = parse_expression_rec
      .clone()
      .separated_by(kind(TokenKind::Comma).padded_by(kind(TokenKind::Whitespace)))
      .or(
        parse_expression_rec
          .clone()
          .then_ignore(kind(TokenKind::Comma).or_not())
          .map(|expr| vec![expr]),
      );

    let parse_call = || {
      parse_name()
        .then(
          parse_sequence
            .clone()
            .delimited_by(kind(TokenKind::ParenL), kind(TokenKind::ParenR)),
        )
        .try_map(|(name, arguments), span| {
          Thunk::resolve(name, arguments).map_err(|err| Simple::custom(span, err))
        })
    };

    let parse_value = || {
      choice((
        parse_name().map(|name| Expression::Variable { name }),
        parse_string().map(|string_literal| Expression::StringLiteral { string_literal }),
        parse_group
          .clone()
          .map(|contents| Expression::Group { contents }),
        parse_call().map(|thunk| Expression::Call { thunk }),
      ))
    };

    let conditional_operator = choice((
      kind(TokenKind::BangEquals).to(ConditionalOperator::Inequality),
      kind(TokenKind::EqualsEquals).to(ConditionalOperator::Equality),
      kind(TokenKind::EqualsTilde).to(ConditionalOperator::RegexMatch),
    ));

    let condition = parse_expression_rec
      .clone()
      .then(conditional_operator.padded_by(ws()))
      .then(parse_expression_rec.clone())
      .map(|((lhs, op), rhs)| CondOutput { lhs, op, rhs });

    let conditional = keyword("if")
      .ignored()
      .then_ignore(ws())
      .then(condition)
      .then_ignore(ws())
      .then(parse_expression_rec.clone().delimited_by(
        kind(TokenKind::BraceL).then(ws().or_not()),
        ws().or_not().then(kind(TokenKind::BraceR)),
      ))
      .then_ignore(ws())
      .then_ignore(keyword("else"))
      .then_ignore(ws())
      .then(parse_expression_rec.clone().delimited_by(
        kind(TokenKind::BraceL).then(ws().or_not()),
        ws().or_not().then(kind(TokenKind::BraceR)),
      ))
      .map(|((((), co), then), otherwise)| Expression::Conditional {
        lhs: Box::new(co.lhs),
        rhs: Box::new(co.rhs),
        then: Box::new(then),
        otherwise: Box::new(otherwise),
        operator: co.op,
      });

    let parse_join = parse_value()
      .or_not()
      .then_ignore(ws().or_not())
      .then_ignore(kind(TokenKind::Slash))
      .then_ignore(ws().or_not())
      .then(parse_expression_rec.clone())
      .map(|(lhs, rhs)| Expression::Join {
        lhs: lhs.map(Box::new),
        rhs: Box::new(rhs),
      });

    let parse_concat = parse_value()
      .then_ignore(ws().or_not())
      .then_ignore(kind(TokenKind::Plus))
      .then_ignore(ws().or_not())
      .then(parse_expression_rec.clone())
      .map(|(lhs, rhs)| Expression::Concatenation {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
      });

    // let join_or_concat = parse_value()
    //   .then_ignore(ws().or_not())
    //   .then(kind(TokenKind::Slash).or(kind(TokenKind::Plus)))
    //   .then_ignore(ws().or_not())
    //   .then(parse_expression_rec.clone())
    //   .map(|((lhs, op), rhs)| match op.kind {
    //     TokenKind::Slash => Expression::Join {
    //       lhs: Box::new(lhs),
    //       rhs: Box::new(rhs),
    //     },
    //     TokenKind::Plus => Expression::Concatenation {
    //       lhs: Box::new(lhs),
    //       rhs: Box::new(rhs),
    //     },
    //     _ => unreachable!(),
    //   });

    choice((conditional, parse_join, parse_concat, parse_value()))
  })
}

struct CondOutput<'a> {
  lhs: Expression<'a>,
  op: ConditionalOperator,
  rhs: Expression<'a>,
}

fn validate_string<'src>(token: Token<'src>) -> Result<StringLiteral<'src>, CompileError<'src>> {
  let kind = StringKind::from_string_or_backtick(token)?;
  let delimiter_len = kind.delimiter_len();
  let raw = &token.lexeme()[delimiter_len..token.lexeme().len() - delimiter_len];
  let unindented = if kind.indented() {
    crate::unindent::unindent(raw)
  } else {
    raw.to_owned()
  };
  let cooked = if kind.processes_escape_sequences() {
    let mut cooked = String::new();
    let mut escape = false;
    for c in unindented.chars() {
      if escape {
        match c {
          'n' => cooked.push('\n'),
          'r' => cooked.push('\r'),
          't' => cooked.push('\t'),
          '\\' => cooked.push('\\'),
          '\n' => {}
          '"' => cooked.push('"'),
          other => {
            return Err(token.error(CompileErrorKind::InvalidEscapeSequence { character: other }));
          }
        }
        escape = false;
      } else if c == '\\' {
        escape = true;
      } else {
        cooked.push(c);
      }
    }
    cooked
  } else {
    unindented
  };
  Ok(StringLiteral { kind, raw, cooked })
}

fn parse_string<'src>() -> impl JustParser<'src, StringLiteral<'src>> {
  kind(TokenKind::StringToken)
    .try_map(|token, span| validate_string(token).map_err(|err| Simple::custom(span, err)))
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
    .debug("parse_items")
}

fn parse_item<'src>() -> impl Parser<Token<'src>, Option<Item<'src>>, Error = Simple<Token<'src>>> {
  fn item_end<'src>() -> impl JustParser<'src, ()> {
    kind(TokenKind::Whitespace)
      .or_not()
      .then(parse_eol())
      .ignored()
  }

  choice((
    parse_setting().then_ignore(item_end()).map(Some),
    parse_alias().then_ignore(item_end()).map(Some),
    parse_assignment().then_ignore(item_end()).map(Some),
    parse_recipe().map(Some),
    parse_eol(),
  ))
}

fn parse_line<'src>() -> impl JustParser<'src, Line<'src>> {
  ws()
    .or_not()
    .ignore_then(
      choice((
        kind(TokenKind::Text).map(|token| Fragment::Text { token }),
        kind(TokenKind::InterpolationStart)
          .ignore_then(parse_expression())
          .then_ignore(kind(TokenKind::InterpolationEnd))
          .map(|expression| Fragment::Interpolation { expression }),
      ))
      .repeated()
      .at_least(1)
      .map(|fragments| Line { fragments }),
    )
    .debug("parse_line")
}

fn parse_recipe_body<'src>() -> impl JustParser<'src, Vec<Line<'src>>> {
  parse_line()
    .separated_by(kind(TokenKind::Eol).repeated().at_least(1))
    .allow_trailing()
    .delimited_by(kind(TokenKind::Indent), kind(TokenKind::Dedent))
    .debug("parse_recipe_body")
}

fn parse_recipe<'src>() -> impl JustParser<'src, Item<'src>> {
  //TODO can this handle doc comments as part of the grammar?

  kind(TokenKind::At)
    .or_not()
    .then(parse_name())
    .then_ignore(kind(TokenKind::Colon))
    .then_ignore(kind(TokenKind::Eol))
    .then(parse_recipe_body())
    .map(|((maybe_quiet, name), body)| {
      Item::Recipe(Recipe {
        body,
        dependencies: vec![],
        doc: None,
        name,
        parameters: vec![],
        priors: 0,
        private: false,
        quiet: maybe_quiet.is_some(),
        shebang: false,
      })
    })
    .debug("parse-recipe")
}

fn parse_assignment<'src>() -> impl JustParser<'src, Item<'src>> {
  (keyword("export").then_ignore(kind(TokenKind::Whitespace)))
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
  keyword("set")
    .map(Name::from_identifier)
    .then_ignore(ws())
    .then(parse_setting_name())
    .map(|(name, value)| Item::Set(Set { name, value }))
}

fn parse_colon_equals<'src, T>(parser: impl JustParser<'src, T>) -> impl JustParser<'src, T> {
  kind(TokenKind::ColonEquals)
    .padded_by(kind(TokenKind::Whitespace))
    .ignore_then(parser)
}

fn parse_set_bool<'src>() -> impl JustParser<'src, bool> {
  let true_or_false = keyword("true").to(true).or(keyword("false").to(false));
  parse_colon_equals(true_or_false)
    .or_not()
    .map(|maybe_bool| maybe_bool.unwrap_or(true))
}

fn parse_set_shell<'src>() -> impl JustParser<'src, Shell<'src>> {
  let string_list = parse_string()
    .padded_by(ws().or_not())
    .separated_by(kind(TokenKind::Comma))
    .at_least(1)
    .allow_trailing()
    .delimited_by(kind(TokenKind::BracketL), kind(TokenKind::BracketR))
    .map(|mut strings: Vec<StringLiteral<'src>>| {
      //TODO strings should always have at least one element because of the .at_least(1) - but
      //have some tests assert that
      let arguments = strings.split_off(1);
      let command = strings.pop().unwrap();
      Shell { arguments, command }
    });
  parse_colon_equals(string_list)
}

fn parse_setting_name<'src>() -> impl JustParser<'src, Setting<'src>> {
  choice((
    keyword("allow-duplicate-recipes")
      .ignore_then(parse_set_bool())
      .map(Setting::AllowDuplicateRecipes),
    keyword("dotenv-load")
      .ignore_then(parse_set_bool())
      .map(Setting::DotenvLoad),
    keyword("export")
      .ignore_then(parse_set_bool())
      .map(Setting::Export),
    keyword("positional-arguments")
      .ignore_then(parse_set_bool())
      .map(Setting::PositionalArguments),
    keyword("windows-powershell")
      .ignore_then(parse_set_bool())
      .map(Setting::WindowsPowerShell),
    keyword("shell")
      .ignore_then(parse_set_shell())
      .map(Setting::Shell),
    keyword("windows-shell")
      .ignore_then(parse_set_shell())
      .map(Setting::WindowsShell),
  ))
}

fn parse_alias<'src>() -> impl Parser<Token<'src>, Item<'src>, Error = Simple<Token<'src>>> {
  keyword("alias")
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
    let src = "set dotenv-load   \nset windows-powershell := false\n\nset export  := true #comment after line\n\n# some stuff";
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
  }

  #[test]
  fn new_parser_test_25() {
    let src = "set windows-shell := ['a']\n";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(
      &ast.items[0],
      Item::Set(Set {
        name: _,
        value: Setting::WindowsShell(Shell {
            arguments, command
        })
      }) if arguments.len() == 0 &&
      command.cooked == "a"
    );

    let src = "set windows-shell := []\n";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = NewParser::parse(&tokens);
    assert!(ast.is_err());
  }

  #[test]
  fn new_parser_test_expressions2() {
    let src = "q := if a =~ 'f' { b } else { c }\n";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let output = NewParser::parse(&tokens);
    let ast = output.unwrap();
    assert_matches!(&ast.items[0],
        Item::Assignment(Assignment {
            value: Expression::Conditional { lhs, rhs, then: _, otherwise: _, operator },
            ..
        }) if matches!(**lhs, Expression::Variable { .. }) &&
              matches!(**rhs, Expression::StringLiteral { string_literal: StringLiteral { .. }}) &&
              *operator == ConditionalOperator::RegexMatch
    );
  }

  #[test]
  fn new_parser_test_expressions() {
    let src = "alpha := \'a string\'\n# some stuff\na := ('str')\n";
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let output = NewParser::parse(&tokens);
    let ast = output.unwrap();
    assert_matches!(
      &ast.items[0],
      Item::Assignment(Assignment {
        value: Expression::StringLiteral {
          string_literal: StringLiteral { .. }
        },
        ..
      })
    );

    assert_matches!(&ast.items[2],
        Item::Assignment(Assignment {
            value: Expression::Group { contents },
            ..
        }) if matches!(**contents, Expression::StringLiteral { .. })
    );
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

  #[test]
  fn new_parser_test_recipe() {
    let src = r#"
@some-recipe:
    echo "hello"

    some-cmd
garbanzo:
    echo no"#;
    let tokens = Lexer::lex(src).unwrap();
    debug_tokens(tokens.clone());
    let ast = NewParser::parse(&tokens).unwrap();
    assert_matches!(&ast.items[0], Item::Recipe(Recipe {
        body, quiet: true, ..
    }) if matches!(&body[0], Line { fragments } if matches!(fragments[0], Fragment::Text { token } if token.lexeme() == "echo \"hello\"")) &&
    matches!(&body[1], Line { fragments } if matches!(fragments[0], Fragment::Text { token } if token.lexeme() == "some-cmd"))
    );

    assert_matches!(&ast.items[1], Item::Recipe(Recipe {
        body, quiet: false, ..
    }) if matches!(&body[0], Line { fragments } if matches!(fragments[0], Fragment::Text { token } if token.lexeme() == "echo no"))
    );
  }
}
