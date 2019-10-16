use crate::common::*;

/// Methods commmon to all AST nodes. Currently only used in parser unit tests.
pub(crate) trait Node<'src> {
  /// Construct an untyped tree of atoms representing this Node. This function,
  /// and `Tree` type, are only used in parser unit tests.
  fn tree(&self) -> Tree<'src>;
}

impl<'src> Node<'src> for Module<'src> {
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
      Item::Recipe(recipe) => recipe.tree(),
    }
  }
}

impl<'src> Node<'src> for Alias<'src> {
  fn tree(&self) -> Tree<'src> {
    Tree::atom(keyword::ALIAS)
      .push(self.name.lexeme())
      .push(self.target.lexeme())
  }
}

impl<'src> Node<'src> for Assignment<'src> {
  fn tree(&self) -> Tree<'src> {
    if self.export {
      Tree::atom("assignment").push("#").push(keyword::EXPORT)
    } else {
      Tree::atom("assignment")
    }
    .push(self.name.lexeme())
    .push(self.expression.tree())
  }
}

impl<'src> Node<'src> for Expression<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Expression::Concatination { lhs, rhs } => Tree::atom("+").push(lhs.tree()).push(rhs.tree()),
      Expression::Call {
        function,
        arguments,
      } => Tree::atom("call")
        .push(function.lexeme())
        .extend(arguments.iter().map(|argument| argument.tree())),
      Expression::Variable { name } => Tree::atom(name.lexeme()),
      Expression::StringLiteral {
        string_literal: StringLiteral { cooked, .. },
      } => Tree::string(cooked),
      Expression::Backtick { contents, .. } => Tree::atom("backtick").push(Tree::string(contents)),
      Expression::Group { contents } => Tree::List(vec![contents.tree()]),
    }
  }
}

impl<'src> Node<'src> for Recipe<'src> {
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
        if parameter.variadic {
          params.push_mut("+");
        }

        params.push_mut(parameter.tree());
      }

      t.push_mut(params);
    }

    if !self.dependencies.is_empty() {
      t = t.push(
        Tree::atom("deps").extend(
          self
            .dependencies
            .iter()
            .map(|dependency| dependency.lexeme()),
        ),
      );
    }

    if !self.body.is_empty() {
      t.push_mut(Tree::atom("body").extend(self.body.iter().map(|line| line.tree())));
    }

    t
  }
}

impl<'src> Node<'src> for Parameter<'src> {
  fn tree(&self) -> Tree<'src> {
    let mut children = Vec::new();
    children.push(Tree::atom(self.name.lexeme()));

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

impl<'src> Node<'src> for Warning<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Warning::DeprecatedEquals { .. } => Tree::atom("warning").push("deprecated_equals"),
    }
  }
}
