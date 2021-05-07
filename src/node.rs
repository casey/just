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

impl<'src> Node<'src> for Expression<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      Expression::Concatination { lhs, rhs } => Tree::atom("+").push(lhs.tree()).push(rhs.tree()),
      Expression::Conditional {
        lhs,
        rhs,
        then,
        otherwise,
        inverted,
      } => {
        let mut tree = Tree::atom(Keyword::If.lexeme());
        tree.push_mut(lhs.tree());
        if *inverted {
          tree.push_mut("!=")
        } else {
          tree.push_mut("==")
        }
        tree.push_mut(rhs.tree());
        tree.push_mut(then.tree());
        tree.push_mut(otherwise.tree());
        tree
      },
      Expression::Call { thunk } => {
        let mut tree = Tree::atom("call");

        use Thunk::*;
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
        }

        tree
      }
      Expression::Variable { name } => Tree::atom(name.lexeme()),
      Expression::StringLiteral {
        string_literal: StringLiteral { cooked, .. },
      } => Tree::string(cooked),
      Expression::Backtick { contents, .. } => Tree::atom("backtick").push(Tree::string(contents)),
      Expression::Group { contents } => Tree::List(vec![contents.tree()]),
      Expression::FormatString { fragments, .. } => {
        Tree::atom("f").extend(fragments.iter().map(|f| f.tree()))
      }
      Expression::FormatBacktick { fragments, .. } => {
        Tree::atom("fbacktick").extend(fragments.iter().map(|f| f.tree()))
      }
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

      for dependency in &self.dependencies {
        let mut d = Tree::atom(dependency.recipe.lexeme());

        for argument in &dependency.arguments {
          d.push_mut(argument.tree());
        }

        dependencies.push_mut(d);
      }

      t.push_mut(dependencies);
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

impl<'src> Node<'src> for StringFragment<'src> {
  fn tree(&self) -> Tree<'src> {
    match self {
      StringFragment::Text { cooked, .. } => Tree::string(cooked),
      StringFragment::Interpolation { expression } => Tree::List(vec![expression.tree()]),
    }
  }
}

impl<'src> Node<'src> for Set<'src> {
  fn tree(&self) -> Tree<'src> {
    let mut set = Tree::atom(Keyword::Set.lexeme());

    set.push_mut(self.name.lexeme().replace('-', "_"));

    use Setting::*;
    match &self.value {
      DotenvLoad(value) | Export(value) | PositionalArguments(value) =>
        set.push_mut(value.to_string()),
      Shell(setting::Shell { command, arguments }) => {
        set.push_mut(Tree::string(&command.cooked));
        for argument in arguments {
          set.push_mut(Tree::string(&argument.cooked));
        }
      },
    }

    set
  }
}

impl<'src> Node<'src> for Warning {
  fn tree(&self) -> Tree<'src> {
    unreachable!()
  }
}
