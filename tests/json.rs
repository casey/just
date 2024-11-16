use super::*;

fn case<F: Fn(&Path) -> Value>(justfile: &str, value: F) {
  Test::new()
    .justfile(justfile)
    .args(["--dump", "--dump-format", "json"])
    .stdout_with_tempdir(|dir| format!("{}\n", serde_json::to_string(&value(dir.path())).unwrap()))
    .run();
}

#[test]
fn alias() {
  let test = Test::new().justfile(
    "
      alias f := foo

      foo:
    ",
  );
  let source = test
    .tempdir
    .path()
    .join("justfile")
    .to_str()
    .unwrap()
    .to_owned();
  let json = json!({
      "first": "foo",
      "doc": null,
      "aliases": {
        "f": {
          "name": "f",
          "target": "foo",
          "attributes": [],
        }
      },
      "assignments": {},
      "groups": [],
      "modules": {},
      "recipes": {
        "foo": {
          "attributes": [],
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "namepath": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "ignore_comments": false,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
  });
  test
    .args(["--dump", "--dump-format", "json"])
    .stdout(format!("{}\n", serde_json::to_string(&json).unwrap()))
    .run();
}

#[test]
fn assignment() {
  case("foo := 'bar'", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "assignments": {
        "foo": {
          "export": false,
          "name": "foo",
          "value": "bar",
          "private": false,
        }
      },
      "first": null,
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {},
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source":  source,
    })
  });
}

#[test]
fn private_assignment() {
  case(
    "
      _foo := 'foo'
      [private]
      bar := 'bar'
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {
          "_foo": {
            "export": false,
            "name": "_foo",
            "value": "foo",
            "private": true,
          },
          "bar": {
            "export": false,
            "name": "bar",
            "value": "bar",
            "private": true,
          },
        },
        "first": null,
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {},
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn body() {
  case(
    "
      foo:
        bar
        abc{{ 'xyz' }}def
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "attributes": [],
            "body": [
              ["bar"],
              ["abc", ["xyz"], "def"],
            ],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn dependencies() {
  case(
    "
      foo:
      bar: foo
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "bar": {
            "attributes": [],
            "doc": null,
            "name": "bar",
            "namepath": "bar",
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
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
            "attributes": [],
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn dependency_argument() {
  case(
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
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "first": "foo",
        "doc": null,
        "assignments": {
          "x": {
            "export": false,
            "name": "x",
            "value": "foo",
            "private": false,
          },
        },
        "groups": [],
        "modules": {},
        "recipes": {
          "bar": {
            "doc": null,
            "name": "bar",
            "namepath": "bar",
            "body": [],
            "dependencies": [{
              "arguments": [
                "baz",
                "baz",
                ["concatenate", "a", "b"],
                ["evaluate", "echo"],
                ["variable", "x"],
                ["if", ["==", "a", "b"], "c", "d"],
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
            "attributes": [],
          },
          "foo": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
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
            "attributes": [],
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn duplicate_recipes() {
  case(
    "
      set allow-duplicate-recipes
      alias f := foo

      foo:
      foo bar:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "first": "foo",
        "doc": null,
        "aliases": {
          "f": {
            "attributes": [],
            "name": "f",
            "target": "foo",
          }
        },
        "assignments": {},
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [
              {
                "name": "bar",
                "export": false,
                "default": null,
                "kind": "singular",
              },
            ],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
            "attributes": [],
          }
        },
        "settings": {
          "allow_duplicate_recipes": true,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn duplicate_variables() {
  case(
    "
      set allow-duplicate-variables
      x := 'foo'
      x := 'bar'
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {
          "x": {
            "export": false,
            "name": "x",
            "value": "bar",
            "private": false,
          }
        },
        "first": null,
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {},
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": true,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn doc_comment() {
  case("# hello\nfoo:", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "first": "foo",
      "doc": null,
      "assignments": {},
      "groups": [],
      "modules": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": "hello",
          "name": "foo",
          "namepath": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
          "attributes": [],
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
    })
  });
}

#[test]
fn empty_justfile() {
  case("", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "assignments": {},
      "first": null,
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {},
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
    })
  });
}

#[test]
fn parameters() {
  case(
    "
      a:
      b x:
      c x='y':
      d +x:
      e *x:
      f $x:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "first": "a",
        "doc": null,
        "assignments": {},
        "groups": [],
        "modules": {},
        "recipes": {
          "a": {
            "attributes": [],
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "a",
            "namepath": "a",
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
            "namepath": "b",
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
            "attributes": [],
          },
          "c": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "c",
            "namepath": "c",
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
            "attributes": [],
          },
          "d": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "d",
            "namepath": "d",
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
            "attributes": [],
          },
          "e": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "e",
            "namepath": "e",
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
            "attributes": [],
          },
          "f": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "f",
            "namepath": "f",
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
            "attributes": [],
          },
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn priors() {
  case(
    "
      a:
      b: a && c
      c:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "a",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "a": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "a",
            "namepath": "a",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
            "attributes": [],
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
            "namepath": "b",
            "private": false,
            "quiet": false,
            "shebang": false,
            "attributes": [],
            "parameters": [],
            "priors": 1,
          },
          "c": {
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "c",
            "namepath": "c",
            "parameters": [],
            "private": false,
            "quiet": false,
            "shebang": false,
            "attributes": [],
            "parameters": [],
            "priors": 0,
          },
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn private() {
  case("_foo:", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "assignments": {},
      "first": "_foo",
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {
        "_foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "_foo",
          "namepath": "_foo",
          "parameters": [],
          "priors": 0,
          "private": true,
          "quiet": false,
          "shebang": false,
          "attributes": [],
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
    })
  });
}

#[test]
fn quiet() {
  case("@foo:", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "namepath": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": true,
          "shebang": false,
          "attributes": [],
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
    })
  });
}

#[test]
fn settings() {
  case(
    "
      set dotenv-load
      set dotenv-filename := \"filename\"
      set dotenv-path := \"path\"
      set export
      set fallback
      set positional-arguments
      set quiet
      set ignore-comments
      set shell := ['a', 'b', 'c']
      foo:
        #!bar
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "body": [["#!bar"]],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": true,
            "attributes": [],
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": "filename",
          "dotenv_load": true,
          "dotenv_path": "path",
          "dotenv_required": false,
          "export": true,
          "fallback": true,
          "ignore_comments": true,
          "positional_arguments": true,
          "quiet": true,
          "shell": {
            "arguments": ["b", "c"],
            "command": "a",
          },
          "tempdir": null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn shebang() {
  case(
    "
      foo:
        #!bar
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "body": [["#!bar"]],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": true,
            "attributes": [],
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir": null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn simple() {
  case("foo:", |dir| {
    let source = dir.join("justfile").to_str().unwrap().to_owned();
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {
        "foo": {
          "body": [],
          "dependencies": [],
          "doc": null,
          "name": "foo",
          "namepath": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
          "attributes": [],
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir": null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
      "source": source,
    })
  });
}

#[test]
fn attribute() {
  case(
    "
      [no-exit-message]
      foo:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "attributes": ["no-exit-message"],
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "ignore_comments": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn module() {
  Test::new()
    .justfile(
      "
      # hello
      mod foo
    ",
    )
    .tree(tree! {
      "foo.just": "bar:",
    })
    .args(["--dump", "--dump-format", "json"])
    .stdout_with_tempdir(|dir| {
      let source = dir.path().join("justfile").to_str().unwrap().to_owned();
      let foo_just = dir.path().join("foo.just").to_str().unwrap().to_owned();
      format!(
        "{}\n",
        serde_json::to_string(&json!({
          "aliases": {},
          "assignments": {},
          "first": null,
          "doc": null,
          "groups": [],
          "modules": {
            "foo": {
              "aliases": {},
              "assignments": {},
              "first": "bar",
              "doc": "hello",
              "groups": [],
              "modules": {},
              "recipes": {
                "bar": {
                  "attributes": [],
                  "body": [],
                  "dependencies": [],
                  "doc": null,
                  "name": "bar",
                  "namepath": "foo::bar",
                  "parameters": [],
                  "priors": 0,
                  "private": false,
                  "quiet": false,
                  "shebang": false,
                }
              },
              "settings": {
                "allow_duplicate_recipes": false,
                "allow_duplicate_variables": false,
                "dotenv_filename": null,
                "dotenv_load": false,
                "dotenv_path": null,
                "dotenv_required": false,
                "export": false,
                "fallback": false,
                "positional_arguments": false,
                "quiet": false,
                "shell": null,
                "tempdir" : null,
                "unstable": false,
                "ignore_comments": false,
                "windows_powershell": false,
                "windows_shell": null,
                "working_directory" : null,
              },
              "unexports": [],
              "warnings": [],
              "source": foo_just
            },
          },
          "recipes": {},
          "settings": {
            "allow_duplicate_recipes": false,
            "allow_duplicate_variables": false,
            "dotenv_filename": null,
            "dotenv_load": false,
            "dotenv_path": null,
            "dotenv_required": false,
            "export": false,
            "fallback": false,
            "positional_arguments": false,
            "quiet": false,
            "shell": null,
            "tempdir" : null,
            "unstable": false,
            "ignore_comments": false,
            "windows_powershell": false,
            "windows_shell": null,
            "working_directory" : null,
          },
          "unexports": [],
          "warnings": [],
          "source": source
        }))
        .unwrap()
      )
    })
    .run();
}

#[test]
fn module_group() {
  Test::new()
    .justfile(
      "
      [group('alpha')]
      mod foo
    ",
    )
    .tree(tree! {
      "foo.just": "bar:",
    })
    .args(["--dump", "--dump-format", "json"])
    .stdout_with_tempdir(|dir| {
      let source = dir.path().join("justfile").to_str().unwrap().to_owned();
      let foo_just = dir.path().join("foo.just").to_str().unwrap().to_owned();
      format!(
        "{}\n",
        serde_json::to_string(&json!({
          "aliases": {},
          "assignments": {},
          "first": null,
          "doc": null,
          "groups": [],
          "modules": {
            "foo": {
              "aliases": {},
              "assignments": {},
              "first": "bar",
              "doc": null,
              "groups": ["alpha"],
              "modules": {},
              "recipes": {
                "bar": {
                  "attributes": [],
                  "body": [],
                  "dependencies": [],
                  "doc": null,
                  "name": "bar",
                  "namepath": "foo::bar",
                  "parameters": [],
                  "priors": 0,
                  "private": false,
                  "quiet": false,
                  "shebang": false,
                }
              },
              "settings": {
                "allow_duplicate_recipes": false,
                "allow_duplicate_variables": false,
                "dotenv_filename": null,
                "dotenv_load": false,
                "dotenv_path": null,
                "dotenv_required": false,
                "export": false,
                "fallback": false,
                "positional_arguments": false,
                "quiet": false,
                "shell": null,
                "tempdir" : null,
                "unstable": false,
                "ignore_comments": false,
                "windows_powershell": false,
                "windows_shell": null,
                "working_directory" : null,
              },
              "unexports": [],
              "warnings": [],
              "source": foo_just,
            },
          },
          "recipes": {},
          "settings": {
            "allow_duplicate_recipes": false,
            "allow_duplicate_variables": false,
            "dotenv_filename": null,
            "dotenv_load": false,
            "dotenv_path": null,
            "dotenv_required": false,
            "export": false,
            "fallback": false,
            "positional_arguments": false,
            "quiet": false,
            "shell": null,
            "tempdir" : null,
            "unstable": false,
            "ignore_comments": false,
            "windows_powershell": false,
            "windows_shell": null,
            "working_directory" : null,
          },
          "unexports": [],
          "warnings": [],
          "source": source,
        }))
        .unwrap()
      )
    })
    .run();
}

#[test]
fn recipes_with_private_attribute_are_private() {
  case(
    "
      [private]
      foo:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "attributes": ["private"],
            "body": [],
            "dependencies": [],
            "doc": null,
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": true,
            "quiet": false,
            "shebang": false,
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source,
      })
    },
  );
}

#[test]
fn doc_attribute_overrides_comment() {
  case(
    "
      # COMMENT
      [doc('ATTRIBUTE')]
      foo:
    ",
    |dir| {
      let source = dir.join("justfile").to_str().unwrap().to_owned();
      json!({
        "aliases": {},
        "assignments": {},
        "first": "foo",
        "doc": null,
        "groups": [],
        "modules": {},
        "recipes": {
          "foo": {
            "attributes": [{"doc": "ATTRIBUTE"}],
            "body": [],
            "dependencies": [],
            "doc": "ATTRIBUTE",
            "name": "foo",
            "namepath": "foo",
            "parameters": [],
            "priors": 0,
            "private": false,
            "quiet": false,
            "shebang": false,
          }
        },
        "settings": {
          "allow_duplicate_recipes": false,
          "allow_duplicate_variables": false,
          "dotenv_filename": null,
          "dotenv_load": false,
          "dotenv_path": null,
          "dotenv_required": false,
          "export": false,
          "fallback": false,
          "ignore_comments": false,
          "positional_arguments": false,
          "quiet": false,
          "shell": null,
          "tempdir" : null,
          "unstable": false,
          "windows_powershell": false,
          "windows_shell": null,
          "working_directory" : null,
        },
        "unexports": [],
        "warnings": [],
        "source": source
      })
    },
  );
}

#[test]
fn doc_attribute_overrides_comment() {
  case(
    "
      # COMMENT
      [doc('ATTRIBUTE')]
      foo:
    ",
    json!({
      "aliases": {},
      "assignments": {},
      "first": "foo",
      "doc": null,
      "groups": [],
      "modules": {},
      "recipes": {
        "foo": {
          "attributes": [{"doc": "ATTRIBUTE"}],
          "body": [],
          "dependencies": [],
          "doc": "ATTRIBUTE",
          "name": "foo",
          "namepath": "foo",
          "parameters": [],
          "priors": 0,
          "private": false,
          "quiet": false,
          "shebang": false,
        }
      },
      "settings": {
        "allow_duplicate_recipes": false,
        "allow_duplicate_variables": false,
        "dotenv_filename": null,
        "dotenv_load": false,
        "dotenv_path": null,
        "dotenv_required": false,
        "export": false,
        "fallback": false,
        "ignore_comments": false,
        "positional_arguments": false,
        "quiet": false,
        "shell": null,
        "tempdir" : null,
        "unstable": false,
        "windows_powershell": false,
        "windows_shell": null,
        "working_directory" : null,
      },
      "unexports": [],
      "warnings": [],
    }),
  );
}
