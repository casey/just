use crate::common::*;

fn test(justfile: &str, value: Value) {
  Test::new()
    .justfile(justfile)
    .args(&["--dump", "--dump-format", "json", "--unstable"])
    .stdout(format!("{}\n", serde_json::to_string(&value).unwrap()))
    .run();
}

#[test]
fn alias() {
  test(
    "
      alias f := foo

      foo:
    ",
    json!({
      "first": "foo",
      "aliases": {
        "f": {
          "name": "f",
          "target": "foo",
        }
      },
      "assignments": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn assignment() {
  test(
    "foo := 'bar'",
    json!({
      "aliases": {},
      "assignments": {
        "foo": {
          "export": false,
          "name": "foo",
          "value": "bar",
        }
      },
      "first": null,
      "recipes": {},
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn body() {
  test(
    "
      foo:
        bar
        abc{{ 'xyz' }}def
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "foo": {
          "body": [
            ["bar"],
            ["abc", ["xyz"], "def"],
          ],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn dependencies() {
  test(
    "
      foo:
      bar: foo
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "bar": {
          "doc": null,
          "name": "bar",
          "body": [],
          "dependencies": [{
            "arguments": [],
            "recipe": "foo"
          }],
          "parameters": [],
          "priors": 1,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn dependency_argument() {
  test(
    "
      x := 'foo'
      foo *args:
      bar: (
        foo
        'baz'
        ('baz')
        ('a' + 'b')
        `echo`
        x
        if 'a' == 'b' { 'c' } else { 'd' }
        arch()
        env_var('foo')
        join('a', 'b')
        replace('a', 'b', 'c')
      )
    ",
    json!({
      "aliases": {},
      "first": "foo",
      "assignments": {
        "x": {
          "export": false,
          "name": "x",
          "value": "foo",
        },
      },
      "recipes": {
        "bar": {
          "doc": null,
          "name": "bar",
          "body": [],
          "dependencies": [{
            "arguments": [
              "baz",
              "baz",
              ["concatinate", "a", "b"],
              ["evaluate", "echo"],
              ["variable", "x"],
              ["if", "==", "a", "b", "c", "d"],
              ["call", "arch"],
              ["call", "env_var", "foo"],
              ["call", "join", "a", "b"],
              ["call", "replace", "a", "b", "c"],
            ],
            "recipe": "foo"
          }],
          "parameters": [],
          "priors": 1,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [
            {
              "name": "args",
              "export": false,
              "default": null,
              "kind": "star",
            }
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn duplicate_recipes() {
  test(
    "
      set allow-duplicate-recipes
      alias f := foo

      foo:
      foo bar:
    ",
    json!({
      "first": "foo",
      "aliases": {
        "f": {
          "name": "f",
          "target": "foo",
        }
      },
      "assignments": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": true,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn doc_comment() {
  test(
    "# hello\nfoo:",
    json!({
      "aliases": {},
      "first": "foo",
      "assignments": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": "hello",
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn empty_justfile() {
  test(
    "",
    json!({
      "aliases": {},
      "assignments": {},
      "first": null,
      "recipes": {},
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn parameters() {
  test(
    "
      a:
      b x:
      c x='y':
      d +x:
      e *x:
      f $x:
    ",
    json!({
      "aliases": {},
      "first": "a",
      "assignments": {},
      "recipes": {
        "a": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "a",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "b": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "b",
          "parameters": [
            {
              "name": "x",
              "export": false,
              "default": null,
              "kind": "singular",
            },
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "c": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "c",
          "parameters": [
            {
              "name": "x",
              "export": false,
              "default": "y",
              "kind": "singular",
            }
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "d": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "d",
          "parameters": [
            {
              "name": "x",
              "export": false,
              "default": null,
              "kind": "plus",
            }
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "e": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "e",
          "parameters": [
            {
              "name": "x",
              "export": false,
              "default": null,
              "kind": "star",
            }
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "f": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "f",
          "parameters": [
            {
              "name": "x",
              "export": true,
              "default": null,
              "kind": "singular",
            }
          ],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn priors() {
  test(
    "
      a:
      b: a && c
      c:
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "a",
      "recipes": {
        "a": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "a",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        },
        "b": {
          "body": [],
          "dependencies": [
            {
              "arguments": [],
              "recipe": "a",
            },
            {
              "arguments": [],
              "recipe": "c",
            }
          ],
          "doc": null,
          "name": "b",
          "private": false,
          "quiet": false,
          "shebang": false,
          "parameters": [],
          "priors": 1,
        },
        "c": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "c",
          "parameters": [],
          "private": false,
          "quiet": false,
          "shebang": false,
          "parameters": [],
          "priors": 0,
        },
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn private() {
  test(
    "_foo:",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "_foo",
      "recipes": {
        "_foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "_foo",
          "parameters": [],
          "priors": 0,
          "private": true,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn quiet() {
  test(
    "@foo:",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": true,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn requires_unstable() {
  Test::new()
    .justfile("foo:")
    .args(&["--dump", "--dump-format", "json"])
    .stderr("error: The JSON dump format is currently unstable. Invoke `just` with the `--unstable` flag to enable unstable features.\n")
    .status(EXIT_FAILURE)
    .run();
}

#[test]
fn settings() {
  test(
    "
      set dotenv-load
      set export
      set positional-arguments
      set shell := ['a', 'b', 'c']

      foo:
        #!bar
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "foo": {
          "body": [["#!bar"]],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": true,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": true,
        "export": true,
        "positional_arguments": true,
        "shell": {
          "arguments": ["b", "c"],
          "command": "a",
        },
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn shebang() {
  test(
    "
      foo:
        #!bar
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "foo": {
          "body": [["#!bar"]],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": true,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}

#[test]
fn simple() {
  test(
    "foo:",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "dotenv_load": null,
        "export": false,
        "positional_arguments": false,
        "shell": null,
        "windows_powershell": false,
      },
      "warnings": [],
    }),
  );
}
