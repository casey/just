use super::*;

#[test]
fn only_named_parameters_original_order() {
    Test::new()
      .arg("foo")
      .arg("a=1")
      .arg("b=2")
      .arg("c=3")
      .justfile(
        r#"
      [named-parameters]
      foo a="a" b="b" c="c":
        echo {{ a }}
        echo {{ b }}
        echo {{ c }}
    "#,
      )
      .stdout(
        "
      1
      2
      3
    ",
      )
      .stderr(
        r#"
      echo 1
      echo 2
      echo 3
    "#,
    )
    .run();
}

#[test]
fn only_named_parameters_mixed_order() {
    Test::new()
      .arg("foo")
      .arg("c=3")
      .arg("b=2")
      .arg("a=1")
      .justfile(
        r#"
      [named-parameters]
      foo a="a" b="b" c="c":
        echo {{ a }}
        echo {{ b }}
        echo {{ c }}
    "#,
      )
      .stdout(
        "
      1
      2
      3
    ",
      )
      .stderr(
        r#"
      echo 1
      echo 2
      echo 3
    "#,
    )
    .run();
}

#[test]
fn positional_and_named_parameters() {
    Test::new()
      .arg("foo")
      .arg("\"a=1\"")
      .arg("c=3")
      .arg("b=2")
      .justfile(
        r#"
      [named-parameters]
      foo a b="b" c="c":
        echo {{ a }}
        echo {{ b }}
        echo {{ c }}
    "#,
      )
      .stdout(
        "
      a=1
      2
      3
    ",
      )
      .stderr(
        r#"
      echo a=1
      echo 2
      echo 3
    "#,
    )
    .run();
}

#[test]
fn fail_on_duplicate_assignment_of_named_param() {
    Test::new()
      .arg("foo")
      .arg("b=2")
      .arg("b=2")
      .justfile(
        r#"
      [named-parameters]
      foo a="1" b="b":
        echo {{ a }}
        echo {{ b }}
    "#,
      )
      .stderr(
        r#"
        error: `b` defined multiple times.
    "#,
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn fail_on_unknown_named_param() {
    Test::new()
      .arg("foo")
      .arg("c=2")
      .justfile(
        r#"
      [named-parameters]
      foo a="1" b="b":
        echo {{ a }}
        echo {{ b }}
    "#,
      )
      .stderr(
        r#"
      error: Recipe does not contain parameter `c`.
      Did you mean `a`?
    "#,
    )
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn named_parameters_setting() {
    Test::new()
      .arg("foo")
      .arg("a=1")
      .arg("c=3")
      .justfile(
        r#"
      set named-parameters

      foo a="a" b="b" c="c":
        echo {{ a }}
        echo {{ b }}
        echo {{ c }}
    "#,
      )
      .stdout(
        "
      1
      b
      3
    ",
      )
      .stderr(
        r#"
      echo 1
      echo b
      echo 3
    "#,
    )
    .run();
}
