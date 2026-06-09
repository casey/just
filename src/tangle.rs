#[derive(Clone, Copy)]
enum State {
  Just { fence: char, len: usize },
  Other { fence: char, len: usize },
  Prose,
}

pub(crate) fn tangle(markdown: &str) -> String {
  let mut output = String::new();

  let mut state = State::Prose;

  for line in markdown.lines() {
    match state {
      State::Just { fence, len } => {
        if closes(line, fence, len) {
          state = State::Prose;
          output.push('\n');
        } else {
          output.push_str(line);
          output.push('\n');
        }
      }
      State::Other { fence, len } => {
        if closes(line, fence, len) {
          state = State::Prose;
        }
        output.push('\n');
      }
      State::Prose => {
        if let Some((fence, len, info)) = opening_fence(line) {
          state = if info.split_whitespace().next() == Some("just") {
            State::Just { fence, len }
          } else {
            State::Other { fence, len }
          };
        }
        output.push('\n');
      }
    }
  }

  output
}

fn opening_fence(line: &str) -> Option<(char, usize, &str)> {
  let fence = line.chars().next().filter(|c| matches!(c, '`' | '~'))?;

  let len = line.chars().take_while(|&c| c == fence).count();

  if len < 3 {
    return None;
  }

  let info = &line[len..];

  if fence == '`' && info.contains('`') {
    return None;
  }

  Some((fence, len, info))
}

fn closes(line: &str, fence: char, len: usize) -> bool {
  let count = line.chars().take_while(|&c| c == fence).count();
  count >= len && line[count..].trim().is_empty()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[track_caller]
  fn case(markdown: &str, expected: &str) {
    assert_eq!(tangle(markdown), expected);
  }

  #[test]
  fn blocks() {
    case("", "");
    case("foo\nbar\n", "\n\n");
    case("foo", "\n");
    case(
      "# foo\n\n```just\nbar:\n echo bar\n```\nbaz\n",
      "\n\n\nbar:\n echo bar\n\n\n",
    );
    case(
      "```just\nfoo := 'bar'\n```\nbaz\n```just\nbob:\n echo {{ foo }}\n```\n",
      "\nfoo := 'bar'\n\n\n\nbob:\n echo {{ foo }}\n\n",
    );
    case("```sh\nfoo\n```\n", "\n\n\n");
  }

  #[test]
  fn info_strings() {
    case("```just foo\nbar:\n```\n", "\nbar:\n\n");
    case("``` just\nfoo:\n```\n", "\nfoo:\n\n");
    case("```justfile\nfoo\n```\n", "\n\n\n");
    case("```JUST\nfoo\n```\n", "\n\n\n");
    case("```just`\nfoo\n", "\n\n");
  }

  #[test]
  fn fences() {
    case("~~~just\nfoo:\n~~~\n", "\nfoo:\n\n");
    case("``just\nfoo\n", "\n\n");
    case("  ```just\nfoo\n", "\n\n");
    case("```just\nfoo:\n```  \n", "\nfoo:\n\n");
    case("```just\nfoo:\n`````\n", "\nfoo:\n\n");
    case("```just\nfoo:\n~~~\nbar\n```\n", "\nfoo:\n~~~\nbar\n\n");
    case("````just\nfoo:\n```\n````\n", "\nfoo:\n```\n\n");
    case("```just\nfoo:\nbar\n", "\nfoo:\nbar\n");
  }

  #[test]
  fn nested() {
    case("~~~\n```just\nfoo\n```\n~~~\n", "\n\n\n\n\n");
    case("````\n```just\nfoo\n```\n````\n", "\n\n\n\n\n");
    case("```\n```just\nfoo\n", "\n\n\n");
  }
}
