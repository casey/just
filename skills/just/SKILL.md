---
name: just
description: >
  Reference for `just`, the command runner. Use when working in a project with
  a `justfile` or when the user mentions `just` or `justfile`.
---

Just
====

Discovery
---------

- `just --dump` Print justfile
- `just --evaluate` Print variable values
- `just --help` Print detailed command-line syntax help
- `just --list` Print recipes with descriptions
- `just --show <RECIPE>` Print recipe source
- `just --summary` Print recipes without descriptiosn

Execution
---------

- `just` Run default recipe
- `just <RECIPE>` Run specific recipe
- `just <RECIPE> <ARG1> <ARG2` Run recipe with arguments

Syntax
------

```just
executable := 'main'

# compile main.c
compile:
  cc main.c -o {{ executable }}

# run main
run: compile
  ./{{ executable }}

# run test
test name: compile
  ./bin/test {{ name }}

# start webserver
serve port='8080':
  python -m http.server {{port}}

# publish current tag
publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  tag=`git describe --tags --exact-match`
  ./bin/check-tag $tag
  git push origin $tag
```

Notes
-----

The comment proceeding a recipe is used as its doc-comment, and included in
`just --list`.

By default, each line of a recipe runs in a fresh shell. Recipes whose bodies
start with `#!` are written to a file and executed as a script.

Commonly used commands and scripts should be turned into `just` recipes.
