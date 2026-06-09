use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

pub(crate) fn tangle(markdown: &str) -> String {
  let mut ranges = Vec::new();

  let mut keep = false;

  for (event, range) in Parser::new(markdown).into_offset_iter() {
    match event {
      Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
        keep = info.split_whitespace().next() == Some("just")
          && (range.start == 0 || markdown.as_bytes()[range.start - 1] == b'\n');
      }
      Event::End(TagEnd::CodeBlock) => keep = false,
      Event::Text(_) if keep => ranges.push(range),
      _ => {}
    }
  }

  let mut ranges = ranges.into_iter().peekable();

  let mut output = String::new();

  let mut offset = 0;

  for line in markdown.split_inclusive('\n') {
    let end = offset + line.len();

    while ranges.next_if(|range| range.end <= offset).is_some() {}

    if ranges
      .peek()
      .is_some_and(|range| range.start <= offset && end <= range.end)
    {
      output.push_str(line.strip_suffix('\n').unwrap_or(line));
    }

    output.push('\n');

    offset = end;
  }

  output
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
    case("  ```just\nfoo\n```\n", "\n\n\n");
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

  #[test]
  fn invisible() {
    case("<!--\n```just\nfoo:\n```\n-->\n", "\n\n\n\n\n");
    case("> ```just\n> foo:\n> ```\n", "\n\n\n");
    case("- foo\n\n  ```just\n  bar:\n  ```\n", "\n\n\n\n\n");
  }
}
