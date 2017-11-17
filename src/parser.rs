use common::*;

use itertools;
use token::TokenKind::*;
use recipe_resolver::resolve_recipes;
use assignment_resolver::resolve_assignments;

pub struct Parser<'a> {
  text:              &'a str,
  tokens:            itertools::PutBack<vec::IntoIter<Token<'a>>>,
  recipes:           Map<&'a str, Recipe<'a>>,
  assignments:       Map<&'a str, Expression<'a>>,
  assignment_tokens: Map<&'a str, Token<'a>>,
  exports:           Set<&'a str>,
}

impl<'a> Parser<'a> {
  pub fn new(text: &'a str, tokens: Vec<Token<'a>>) -> Parser<'a> {
    Parser {
      text:              text,
      tokens:            itertools::put_back(tokens),
      recipes:           empty(),
      assignments:       empty(),
      assignment_tokens: empty(),
      exports:           empty(),
    }
  }

  fn peek(&mut self, kind: TokenKind) -> bool {
    let next = self.tokens.next().unwrap();
    let result = next.kind == kind;
    self.tokens.put_back(next);
    result
  }

  fn accept(&mut self, kind: TokenKind) -> Option<Token<'a>> {
    if self.peek(kind) {
      self.tokens.next()
    } else {
      None
    }
  }

  fn accept_any(&mut self, kinds: &[TokenKind]) -> Option<Token<'a>> {
    for kind in kinds {
      if self.peek(*kind) {
        return self.tokens.next();
      }
    }
    None
  }

  fn accepted(&mut self, kind: TokenKind) -> bool {
    self.accept(kind).is_some()
  }

  fn expect(&mut self, kind: TokenKind) -> Option<Token<'a>> {
    if self.peek(kind) {
      self.tokens.next();
      None
    } else {
      self.tokens.next()
    }
  }

  fn expect_eol(&mut self) -> Option<Token<'a>> {
    self.accepted(Comment);
    if self.peek(Eol) {
      self.accept(Eol);
      None
    } else if self.peek(Eof) {
      None
    } else {
      self.tokens.next()
    }
  }

  fn unexpected_token(&self, found: &Token<'a>, expected: &[TokenKind]) -> CompilationError<'a> {
    found.error(CompilationErrorKind::UnexpectedToken {
      expected: expected.to_vec(),
      found:    found.kind,
    })
  }

  fn recipe(
    &mut self,
    name:  Token<'a>,
    doc:   Option<Token<'a>>,
    quiet: bool,
  ) -> CompilationResult<'a, ()> {
    if let Some(recipe) = self.recipes.get(name.lexeme) {
      return Err(name.error(CompilationErrorKind::DuplicateRecipe {
        recipe: recipe.name,
        first:  recipe.line_number
      }));
    }

    let mut parsed_parameter_with_default = false;
    let mut parsed_variadic_parameter = false;
    let mut parameters: Vec<Parameter> = vec![];
    loop {
      let plus = self.accept(Plus);

      let parameter = match self.accept(Name) {
        Some(parameter) => parameter,
        None            => if let Some(plus) = plus {
          return Err(self.unexpected_token(&plus, &[Name]));
        } else {
          break
        },
      };

      let variadic = plus.is_some();

      if parsed_variadic_parameter {
        return Err(parameter.error(CompilationErrorKind::ParameterFollowsVariadicParameter {
          parameter: parameter.lexeme,
        }));
      }

      if parameters.iter().any(|p| p.name == parameter.lexeme) {
        return Err(parameter.error(CompilationErrorKind::DuplicateParameter {
          recipe: name.lexeme, parameter: parameter.lexeme
        }));
      }

      let default;
      if self.accepted(Equals) {
        if let Some(string) = self.accept_any(&[StringToken, RawString]) {
          default = Some(CookedString::new(&string)?.cooked);
        } else {
          let unexpected = self.tokens.next().unwrap();
          return Err(self.unexpected_token(&unexpected, &[StringToken, RawString]));
        }
      } else {
        default = None
      }

      if parsed_parameter_with_default && default.is_none() {
        return Err(parameter.error(CompilationErrorKind::RequiredParameterFollowsDefaultParameter{
          parameter: parameter.lexeme,
        }));
      }

      parsed_parameter_with_default |= default.is_some();
      parsed_variadic_parameter = variadic;

      parameters.push(Parameter {
        default:  default,
        name:     parameter.lexeme,
        token:    parameter,
        variadic: variadic,
      });
    }

    if let Some(token) = self.expect(Colon) {
      // if we haven't accepted any parameters, an equals
      // would have been fine as part of an assignment
      if parameters.is_empty() {
        return Err(self.unexpected_token(&token, &[Name, Plus, Colon, Equals]));
      } else {
        return Err(self.unexpected_token(&token, &[Name, Plus, Colon]));
      }
    }

    let mut dependencies = vec![];
    let mut dependency_tokens = vec![];
    while let Some(dependency) = self.accept(Name) {
      if dependencies.contains(&dependency.lexeme) {
        return Err(dependency.error(CompilationErrorKind::DuplicateDependency {
          recipe:     name.lexeme,
          dependency: dependency.lexeme
        }));
      }
      dependencies.push(dependency.lexeme);
      dependency_tokens.push(dependency);
    }

    if let Some(token) = self.expect_eol() {
      return Err(self.unexpected_token(&token, &[Name, Eol, Eof]));
    }

    let mut lines: Vec<Vec<Fragment>> = vec![];
    let mut shebang = false;

    if self.accepted(Indent) {
      while !self.accepted(Dedent) {
        if self.accepted(Eol) {
          lines.push(vec![]);
          continue;
        }
        if let Some(token) = self.expect(Line) {
          return Err(token.error(CompilationErrorKind::Internal{
            message: format!("Expected a line but got {}", token.kind)
          }))
        }
        let mut fragments = vec![];

        while !(self.accepted(Eol) || self.peek(Dedent)) {
          if let Some(token) = self.accept(Text) {
            if fragments.is_empty() {
              if lines.is_empty() {
                if token.lexeme.starts_with("#!") {
                  shebang = true;
                }
              } else if !shebang
                && !lines.last().and_then(|line| line.last())
                  .map(Fragment::continuation).unwrap_or(false)
                && (token.lexeme.starts_with(' ') || token.lexeme.starts_with('\t')) {
                return Err(token.error(CompilationErrorKind::ExtraLeadingWhitespace));
              }
            }
            fragments.push(Fragment::Text{text: token});
          } else if let Some(token) = self.expect(InterpolationStart) {
            return Err(self.unexpected_token(&token, &[Text, InterpolationStart, Eol]));
          } else {
            fragments.push(Fragment::Expression{
              expression: self.expression(true)?
            });
            if let Some(token) = self.expect(InterpolationEnd) {
              return Err(self.unexpected_token(&token, &[InterpolationEnd]));
            }
          }
        }

        lines.push(fragments);
      }
    }

    self.recipes.insert(name.lexeme, Recipe {
      line_number:       name.line,
      name:              name.lexeme,
      doc:               doc.map(|t| t.lexeme[1..].trim()),
      dependencies:      dependencies,
      dependency_tokens: dependency_tokens,
      parameters:        parameters,
      private:           &name.lexeme[0..1] == "_",
      lines:             lines,
      shebang:           shebang,
      quiet:             quiet,
    });

    Ok(())
  }

  fn expression(&mut self, interpolation: bool) -> CompilationResult<'a, Expression<'a>> {
    let first = self.tokens.next().unwrap();
    let lhs = match first.kind {
      Name        => Expression::Variable {name: first.lexeme, token: first},
      Backtick    => Expression::Backtick {
        raw:   &first.lexeme[1..first.lexeme.len()-1],
        token: first
      },
      RawString | StringToken => {
        Expression::String{cooked_string: CookedString::new(&first)?}
      }
      _ => return Err(self.unexpected_token(&first, &[Name, StringToken])),
    };

    if self.accepted(Plus) {
      let rhs = self.expression(interpolation)?;
      Ok(Expression::Concatination{lhs: Box::new(lhs), rhs: Box::new(rhs)})
    } else if interpolation && self.peek(InterpolationEnd) {
      Ok(lhs)
    } else if let Some(token) = self.expect_eol() {
      if interpolation {
        return Err(self.unexpected_token(&token, &[Plus, Eol, InterpolationEnd]))
      } else {
        Err(self.unexpected_token(&token, &[Plus, Eol]))
      }
    } else {
      Ok(lhs)
    }
  }

  fn assignment(&mut self, name: Token<'a>, export: bool) -> CompilationResult<'a, ()> {
    if self.assignments.contains_key(name.lexeme) {
      return Err(name.error(CompilationErrorKind::DuplicateVariable {variable: name.lexeme}));
    }
    if export {
      self.exports.insert(name.lexeme);
    }
    let expression = self.expression(false)?;
    self.assignments.insert(name.lexeme, expression);
    self.assignment_tokens.insert(name.lexeme, name);
    Ok(())
  }

  pub fn justfile(mut self) -> CompilationResult<'a, Justfile<'a>> {
    let mut doc = None;
    loop {
      match self.tokens.next() {
        Some(token) => match token.kind {
          Eof => break,
          Eol => {
            doc = None;
            continue;
          }
          Comment => {
            if let Some(token) = self.expect_eol() {
              return Err(token.error(CompilationErrorKind::Internal {
                message: format!("found comment followed by {}", token.kind),
              }));
            }
            doc = Some(token);
          }
          At => if let Some(name) = self.accept(Name) {
            self.recipe(name, doc, true)?;
            doc = None;
          } else {
            let unexpected = &self.tokens.next().unwrap();
            return Err(self.unexpected_token(unexpected, &[Name]));
          },
          Name => if token.lexeme == "export" {
            let next = self.tokens.next().unwrap();
            if next.kind == Name && self.accepted(Equals) {
              self.assignment(next, true)?;
              doc = None;
            } else {
              self.tokens.put_back(next);
              self.recipe(token, doc, false)?;
              doc = None;
            }
          } else if self.accepted(Equals) {
            self.assignment(token, false)?;
            doc = None;
          } else {
            self.recipe(token, doc, false)?;
            doc = None;
          },
          _ => return Err(self.unexpected_token(&token, &[Name, At])),
        },
        None => return Err(CompilationError {
          text:   self.text,
          index:  0,
          line:   0,
          column: 0,
          width:  None,
          kind:   CompilationErrorKind::Internal {
            message: "unexpected end of token stream".to_string()
          }
        }),
      }
    }

    if let Some(token) = self.tokens.next() {
      return Err(token.error(CompilationErrorKind::Internal {
        message: format!("unexpected token remaining after parsing completed: {:?}", token.kind)
      }))
    }

    resolve_recipes(&self.recipes, &self.assignments, self.text)?;

    for recipe in self.recipes.values() {
      for parameter in &recipe.parameters {
        if self.assignments.contains_key(parameter.token.lexeme) {
          return Err(parameter.token.error(CompilationErrorKind::ParameterShadowsVariable {
            parameter: parameter.token.lexeme
          }));
        }
      }

      for dependency in &recipe.dependency_tokens {
        if !self.recipes[dependency.lexeme].parameters.is_empty() {
          return Err(dependency.error(CompilationErrorKind::DependencyHasParameters {
            recipe: recipe.name,
            dependency: dependency.lexeme,
          }));
        }
      }
    }

    resolve_assignments(&self.assignments, &self.assignment_tokens)?;

    Ok(Justfile {
      recipes:     self.recipes,
      assignments: self.assignments,
      exports:     self.exports,
    })
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use brev;
  use testing::parse_success;
  use testing::parse_error;

fn parse_summary(input: &str, output: &str) {
  let justfile = parse_success(input);
  let s = format!("{:#}", justfile);
  if s != output {
    println!("got:\n\"{}\"\n", s);
    println!("\texpected:\n\"{}\"", output);
    assert_eq!(s, output);
  }
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
fn parse_variadic() {
  parse_summary(r#"

foo +a:


  "#, r#"foo +a:"#);
}

#[test]
fn parse_variadic_string_default() {
  parse_summary(r#"

foo +a="Hello":


  "#, r#"foo +a='Hello':"#);
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
  parse_error(text, CompilationError {
    text:   text,
    index:  5,
    line:   0,
    column: 5,
    width:  Some(1),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![Name, Plus, Colon], found: Eol},
  });
}

#[test]
fn missing_default_eol() {
  let text = "hello arg=\n";
  parse_error(text, CompilationError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(1),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Eol},
  });
}

#[test]
fn missing_default_eof() {
  let text = "hello arg=";
  parse_error(text, CompilationError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(0),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Eof},
  });
}

#[test]
fn missing_default_colon() {
  let text = "hello arg=:";
  parse_error(text, CompilationError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(1),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Colon},
  });
}

#[test]
fn missing_default_backtick() {
  let text = "hello arg=`hello`";
  parse_error(text, CompilationError {
    text:   text,
    index:  10,
    line:   0,
    column: 10,
    width:  Some(7),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![StringToken, RawString], found: Backtick},
  });
}

#[test]
fn parameter_after_variadic() {
  let text = "foo +a bbb:";
  parse_error(text, CompilationError {
    text:   text,
    index:  7,
    line:   0,
    column: 7,
    width:  Some(3),
    kind:   CompilationErrorKind::ParameterFollowsVariadicParameter{parameter: "bbb"}
  });
}

#[test]
fn required_after_default() {
  let text = "hello arg='foo' bar:";
  parse_error(text, CompilationError {
    text:   text,
    index:  16,
    line:   0,
    column: 16,
    width:  Some(3),
    kind:   CompilationErrorKind::RequiredParameterFollowsDefaultParameter{parameter: "bar"},
  });
}

#[test]
fn missing_eol() {
  let text = "a b c: z =";
  parse_error(text, CompilationError {
    text:   text,
    index:  9,
    line:   0,
    column: 9,
    width:  Some(1),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![Name, Eol, Eof], found: Equals},
  });
}

#[test]
fn eof_test() {
  parse_summary("x:\ny:\nz:\na b c: x y z", "a b c: x y z\n\nx:\n\ny:\n\nz:");
}

#[test]
fn duplicate_parameter() {
  let text = "a b b:";
  parse_error(text, CompilationError {
    text:   text,
    index:  4,
    line:   0,
    column: 4,
    width:  Some(1),
    kind:   CompilationErrorKind::DuplicateParameter{recipe: "a", parameter: "b"}
  });
}

#[test]
fn parameter_shadows_varible() {
  let text = "foo = \"h\"\na foo:";
  parse_error(text, CompilationError {
    text:   text,
    index:  12,
    line:   1,
    column: 2,
    width:  Some(3),
    kind:   CompilationErrorKind::ParameterShadowsVariable{parameter: "foo"}
  });
}

#[test]
fn dependency_has_parameters() {
  let text = "foo arg:\nb: foo";
  parse_error(text, CompilationError {
    text:   text,
    index:  12,
    line:   1,
    column: 3,
    width:  Some(3),
    kind:   CompilationErrorKind::DependencyHasParameters{recipe: "b", dependency: "foo"}
  });
}


#[test]
fn duplicate_dependency() {
  let text = "a b c: b c z z";
  parse_error(text, CompilationError {
    text:   text,
    index:  13,
    line:   0,
    column: 13,
    width:  Some(1),
    kind:   CompilationErrorKind::DuplicateDependency{recipe: "a", dependency: "z"}
  });
}

#[test]
fn duplicate_recipe() {
  let text = "a:\nb:\na:";
  parse_error(text, CompilationError {
    text:   text,
    index:  6,
    line:   2,
    column: 0,
    width:  Some(1),
    kind:   CompilationErrorKind::DuplicateRecipe{recipe: "a", first: 0}
  });
}

#[test]
fn duplicate_variable() {
  let text = "a = \"0\"\na = \"0\"";
  parse_error(text, CompilationError {
    text:   text,
    index:  8,
    line:   1,
    column: 0,
    width:  Some(1),
    kind:   CompilationErrorKind::DuplicateVariable{variable: "a"}
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
fn extra_whitespace() {
  let text = "a:\n blah\n  blarg";
  parse_error(text, CompilationError {
    text:   text,
    index:  10,
    line:   2,
    column: 1,
    width:  Some(6),
    kind:   CompilationErrorKind::ExtraLeadingWhitespace
  });

  // extra leading whitespace is okay in a shebang recipe
  parse_success("a:\n #!\n  print(1)");
}
#[test]
fn interpolation_outside_of_recipe() {
  let text = "{{";
  parse_error(text, CompilationError {
    text:   text,
    index:  0,
    line:   0,
    column: 0,
    width:  Some(2),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![Name, At], found: InterpolationStart},
  });
}
#[test]
fn unclosed_interpolation_delimiter() {
  let text = "a:\n echo {{ foo";
  parse_error(text, CompilationError {
    text:   text,
    index:  15,
    line:   1,
    column: 12,
    width:  Some(0),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![Plus, Eol, InterpolationEnd], found: Dedent},
  });
}

#[test]
fn plus_following_parameter() {
  let text = "a b c+:";
  parse_error(text, CompilationError {
    text:   text,
    index:  5,
    line:   0,
    column: 5,
    width:  Some(1),
    kind:   CompilationErrorKind::UnexpectedToken{expected: vec![Name], found: Plus},
  });
}

#[test]
fn readme_test() {
  let mut justfiles = vec![];
  let mut current = None;

  for line in brev::slurp("README.asc").lines() {
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

}
