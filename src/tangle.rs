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
  use {super::*, crate::unindent::unindent};

  #[track_caller]
  fn case(markdown: &str, expected: &str) {
    assert_eq!(tangle(&unindent(markdown)), unindent(expected));
  }

  #[test]
  fn blocks() {
    case("", "");

    case(
      "foo", "

      ",
    );

    case(
      "
        foo
        bar
      ",
      "


      ",
    );

    case(
      "
        # foo

        ```just
        bar:
         echo bar
        ```
        baz
      ",
      "



        bar:
         echo bar


      ",
    );

    case(
      "
        ```just
        foo := 'bar'
        ```
        baz
        ```just
        bob:
         echo {{ foo }}
        ```
      ",
      "

        foo := 'bar'



        bob:
         echo {{ foo }}

      ",
    );

    case(
      "
        ```sh
        foo
        ```
      ",
      "



      ",
    );
  }

  #[test]
  fn info_strings() {
    case(
      "
        ```just foo
        bar:
        ```
      ",
      "

        bar:

      ",
    );

    case(
      "
        ``` just
        foo:
        ```
      ",
      "

        foo:

      ",
    );

    case(
      "
        ```justfile
        foo
        ```
      ",
      "



      ",
    );

    case(
      "
        ```JUST
        foo
        ```
      ",
      "



      ",
    );

    case(
      "
        ```just`
        foo
      ",
      "


      ",
    );
  }

  #[test]
  fn fences() {
    case(
      "
        ~~~just
        foo:
        ~~~
      ",
      "

        foo:

      ",
    );

    case(
      "
        ``just
        foo
      ",
      "


      ",
    );

    case(
      "
          ```just
        foo
        ```
      ",
      "



      ",
    );

    case(
      "
        ```just
        foo:
        ```\x20\x20
      ",
      "

        foo:

      ",
    );

    case(
      "
        ```just
        foo:
        `````
      ",
      "

        foo:

      ",
    );

    case(
      "
        ```just
        foo:
        ~~~
        bar
        ```
      ",
      "

        foo:
        ~~~
        bar

      ",
    );

    case(
      "
        ````just
        foo:
        ```
        ````
      ",
      "

        foo:
        ```

      ",
    );

    case(
      "
        ```just
        foo:
        bar
      ",
      "

        foo:
        bar
      ",
    );
  }

  #[test]
  fn nested() {
    case(
      "
        ~~~
        ```just
        foo
        ```
        ~~~
      ",
      "





      ",
    );

    case(
      "
        ````
        ```just
        foo
        ```
        ````
      ",
      "





      ",
    );

    case(
      "
        ```
        ```just
        foo
      ",
      "



      ",
    );
  }

  #[test]
  fn invisible() {
    case(
      "
        <!--
        ```just
        foo:
        ```
        -->
      ",
      "





      ",
    );

    case(
      "
        > ```just
        > foo:
        > ```
      ",
      "



      ",
    );

    case(
      "
        - foo

          ```just
          bar:
          ```
      ",
      "





      ",
    );
  }
}
