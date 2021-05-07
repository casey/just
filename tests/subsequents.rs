test! {
  name: single_and_subsequent_success,
  justfile: "
    foo: && bar
      echo foo

    bar:
      echo bar
  ",
  stdout: "
    foo
    bar
  ",
  stderr: "
    echo foo
    echo bar
  ",
}

test! {
  name: single_and_subsequent_failure,
  justfile: "
    foo: && bar
      false

    bar:
      echo bar
  ",
  stdout: "",
  stderr: "
    false
  ",
}
