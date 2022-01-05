use crate::common::*;

test! {
  name: identity_closure,
  justfile: r#"
id(s) := s

foo:
  echo {{id("foo")}}
"#,
  stdout: "foo",
  stderr: "echo foo",
}

test! {
  name: custom_strjoin,
  justfile: r#"
join(a,b,sep) := a + sep + b

foo:
  echo '{{join("Hello", "world!", ", ")}}'
"#,
  stdout: "Hello, world!",
  stderr: "echo 'Hello, world!",
}

test! {
  name: closures_capture_context, // alliteration!
  justfile: r#"
var := "bar"
append_var(s) := s + var

foo:
  echo {{append_var("foo")}}
"#,
  stdout: "foobar",
  stderr: "echo foobar",
}

// TODO: if this adds extra complexity, it isn't necessary right now
test! {
  name: backtick_closure,
  justfile: r#"
append_bar(s) := s + `echo bar`

foo:
  echo {{append_bar("foo")}}
"#,
  stdout: "foobar",
  stderr: "echo foobar",
}
