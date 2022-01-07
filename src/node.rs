use crate::common::*;

/// Methods commmon to all AST nodes. Currently only used in parser unit tests.
pub(crate) trait Node<'src> {
  /// Construct an untyped tree of atoms representing this Node. This function,
  /// and `Tree` type, are only used in parser unit tests.
  fn tree(&self) -> Tree<'src>;
}

impl<'src> Node<'src> for Ast<'src> {
  fn tree(&self) -> Tree<'src> {
    Tree::atom("justfile")
      .extend(self.items.iter().map(|item| item.tree()))
      .extend(self.warnings.iter().map(|warning| warning.tree()))
  }
}

impl<'src> Node<'src> for Item<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Item::Alias(alias) => alias.tree(),
      Item::Assignment(assignment) => assignment.tree(),
      Item::Closure(closure) => closure.tree(),
      Item::Comment(comment) => comment.tree(),
      Item::Recipe(recipe) => recipe.tree(),
      Item::Set(set) => set.tree(),
    }
  }
}

impl<'src> Node<'src> for Alias<'src, Name<'src>> {
  fn tree(&self) -> Tree<'src> {
    Tree::atom(Keyword::Alias.lexeme())
      .push(self.name.lexeme())
      .push(self.target.lexeme())
  }
}

impl<'src> Node<'src> for Assignment<'src> {
  fn tree(&self) -> Tree<'src> {
    if self.export {
      Tree::atom("assignment")
        .push("#")
        .push(Keyword::Export.lexeme())
    } else {
      Tree::atom("assignment")
    }
    .push(self.name.lexeme())
    .push(self.value.tree())
  }
}

impl<'src> Node<'src> for NamedClosure<'src> {
  fn tree(&self) -> Tree<'src> {
    let params = self
      .value
      .params
      .iter()
      .fold(Tree::atom("params"), |t, x| t.push(x.lexeme()));

    Tree::atom("closure")
      .push(self.name.lexeme())
      .push(params)
      .push(self.value.rule.tree())
  }
}

impl<'src> Node<'src> for Expression<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Expression::Concatination { lhs, rhs } => Tree::atom("+").push(lhs.tree()).push(rhs.tree()),
      Expression::Conditional {
        lhs,
        rhs,
        then,
        otherwise,
        operator,
      } => {
        let mut tree = Tree::atom(Keyword::If.lexeme());
        tree.push_mut(lhs.tree());
        tree.push_mut(operator.to_string());
        tree.push_mut(rhs.tree());
        tree.push_mut(then.tree());
        tree.push_mut(otherwise.tree());
        tree
      }
      Expression::Call { thunk } => {
        use Thunk::*;

        let mut tree = Tree::atom("call");

        match thunk {
          Nullary { name, .. } => tree.push_mut(name.lexeme()),
          Unary { name, arg, .. } => {
            tree.push_mut(name.lexeme());
            tree.push_mut(arg.tree());
          }
          Binary {
            name, args: [a, b], ..
          } => {
            tree.push_mut(name.lexeme());
            tree.push_mut(a.tree());
            tree.push_mut(b.tree());
          }
          BinaryPlus {
            name,
            args: ([a, b], rest),
            ..
          } => {
            tree.push_mut(name.lexeme());
            tree.push_mut(a.tree());
            tree.push_mut(b.tree());
            for arg in rest {
              tree.push_mut(arg.tree());
            }
          }
          Ternary {
            name,
            args: [a, b, c],
            ..
          } => {
            tree.push_mut(name.lexeme());
            tree.push_mut(a.tree());
            tree.push_mut(b.tree());
            tree.push_mut(c.tree());
          }
          User { name, args } => {
            tree.push_mut(name.lexeme());
            for arg in args {
              tree.push_mut(arg.tree());
            }
          }
        }

        tree
      }
      Expression::Variable { name } => Tree::atom(name.lexeme()),
      Expression::StringLiteral {
        string_literal: StringLiteral { cooked, .. },
      } => Tree::string(cooked),
      Expression::Backtick { contents, .. } => Tree::atom("backtick").push(Tree::string(contents)),
      Expression::Group { contents } => Tree::List(vec![contents.tree()]),
    }
  }
}

impl<'src> Node<'src> for UnresolvedRecipe<'src> {
  fn tree(&self) -> Tree<'src> {
    let mut t = Tree::atom("recipe");

    if self.quiet {
      t.push_mut("#");
      t.push_mut("quiet");
    }

    if let Some(doc) = self.doc {
      t.push_mut(Tree::string(doc));
    }

    t.push_mut(self.name.lexeme());

    if !self.parameters.is_empty() {
      let mut params = Tree::atom("params");

      for parameter in &self.parameters {
        if let Some(prefix) = parameter.kind.prefix() {
          params.push_mut(prefix);
        }

        params.push_mut(parameter.tree());
      }

      t.push_mut(params);
    }

    if !self.dependencies.is_empty() {
      let mut dependencies = Tree::atom("deps");
      let mut subsequents = Tree::atom("sups");

      for (i, dependency) in self.dependencies.iter().enumerate() {
        let mut d = Tree::atom(dependency.recipe.lexeme());

        for argument in &dependency.arguments {
          d.push_mut(argument.tree());
        }

        if i < self.priors {
          dependencies.push_mut(d);
        } else {
          subsequents.push_mut(d);
        }
      }

      if let Tree::List(_) = dependencies {
        t.push_mut(dependencies);
      }

      if let Tree::List(_) = subsequents {
        t.push_mut(subsequents);
      }
    }

    if !self.body.is_empty() {
      t.push_mut(Tree::atom("body").extend(self.body.iter().map(|line| line.tree())));
    }

    t
  }
}

impl<'src> Node<'src> for Parameter<'src> {
  fn tree(&self) -> Tree<'src> {
    let mut children = vec![Tree::atom(self.name.lexeme())];

    if let Some(default) = &self.default {
      children.push(default.tree());
    }

    Tree::List(children)
  }
}

impl<'src> Node<'src> for Line<'src> {
  fn tree(&self) -> Tree<'src> {
    Tree::list(self.fragments.iter().map(|fragment| fragment.tree()))
  }
}

impl<'src> Node<'src> for Fragment<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Fragment::Text { token } => Tree::string(token.lexeme()),
      Fragment::Interpolation { expression } => Tree::List(vec![expression.tree()]),
    }
  }
}

impl<'src> Node<'src> for Set<'src> {
  fn tree(&self) -> Tree<'src> {
    use Setting::*;

    let mut set = Tree::atom(Keyword::Set.lexeme());
    set.push_mut(self.name.lexeme().replace('-', "_"));

    match &self.value {
      DotenvLoad(value) | Export(value) | PositionalArguments(value) => {
        set.push_mut(value.to_string());
      }
      Shell(setting::Shell { command, arguments }) => {
        set.push_mut(Tree::string(&command.cooked));
        for argument in arguments {
          set.push_mut(Tree::string(&argument.cooked));
        }
      }
    }

    set
  }
}

impl<'src> Node<'src> for Warning {
  fn tree(&self) -> Tree<'src> {
    unreachable!()
  }
}

impl<'src> Node<'src> for str {
  fn tree(&self) -> Tree<'src> {
    Tree::atom("comment").push(["\"", self, "\""].concat())
  }
}
