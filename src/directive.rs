use crate::common::*;

/// A specific instruction inside a recipe
#[derive(Debug, Default)]
pub(crate) struct Directive {
  evaluated:     String,
  continuations: Vec<usize>,
}

/// A companion structure for parsing directives
pub(crate) struct DirectiveParser<'src, 'run> {
  evaluator:      Evaluator<'src, 'run>,
  lines_consumed: usize,
}

impl Directive {
  const QUIET: char = '@';

  pub(crate) fn parser<'src, 'run>(
    evaluator: Evaluator<'src, 'run>,
  ) -> DirectiveParser<'src, 'run> {
    DirectiveParser {
      evaluator,
      lines_consumed: 0,
    }
  }

  pub(crate) fn is_quiet(&self) -> bool {
    self.evaluated.starts_with(Self::QUIET)
  }

  pub(crate) fn as_evaluated_str(&self) -> &str {
    if self.is_quiet() {
      &self.evaluated[1..]
    } else {
      &self.evaluated
    }
  }
}

impl Display for Directive {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let mut rest = self.as_evaluated_str();
    if self.continuations.is_empty() {
      write!(f, "{}", rest)
    } else {
      let mut consumed = 0;
      for continuation in self.continuations.iter().copied() {
        let (part, tail) = rest.split_at(continuation - consumed - 1);
        rest = tail;
        consumed += part.len();
        write!(f, "{} ", part.trim())?;
      }
      write!(f, "{}", rest.trim())
    }
  }
}

impl<'src, 'run> DirectiveParser<'src, 'run> {
  pub(crate) fn parse<'recipe>(
    &mut self,
    lines: impl Iterator<Item = &'recipe Line<'src>>,
  ) -> RunResult<'src, Directive>
  where
    'src: 'recipe,
  {
    let mut lines = lines.peekable();
    let mut directive = Directive::default();
    {
      let Directive {
        evaluated: command,
        continuations,
      } = &mut directive;
      loop {
        if lines.peek().is_none() {
          break;
        }
        let line = lines.next().unwrap();
        self.lines_consumed += 1;
        command.push_str(&self.evaluator.evaluate_line(line)?);
        if line.is_continuation() {
          continuations.push(command.len());
          command.pop();
        } else {
          break;
        }
      }
    }
    Ok(directive)
  }

  pub(crate) fn lines_consumed(&self) -> usize {
    self.lines_consumed
  }

  pub(crate) fn into_inner(self) -> Evaluator<'src, 'run> {
    self.evaluator
  }
}
