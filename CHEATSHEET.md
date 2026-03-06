# `just` Cheat Sheet

Quick reference for `just` syntax and features. For full docs, see the [book](https://just.systems/man/en/).

## Recipes

```just
# Run with: just build
build:
  cargo build

# Parameters
test target tests="all":
  cargo test {{target}} -- {{tests}}

# Variadic: one or more (+) or zero or more (*)
backup +files:
  scp {{files}} server:

commit msg *flags:
  git commit {{flags}} -m "{{msg}}"

# Export parameter as env var
serve $port:
  node server.js

# Default recipe
default:
  @just --list
```

## Variables & Expressions

```just
version := "1.0.0"
export DB_URL := env("DATABASE_URL", "sqlite://local.db")

# Backtick evaluation
git_hash := `git rev-parse --short HEAD`

# String types
a := "escaped\n"       # double-quoted: \n \t \\ \"
b := 'literal\n'       # single-quoted: no escapes
c := '''               # indented: common whitespace stripped
  multi
  line
'''

# Conditionals
mode := if env("CI", "") != "" { "release" } else { "debug" }

# Concatenation and path joining
full := version + "-beta"
path := "src" / "main.rs"
```

## Dependencies

```just
# Prior dependencies (run before)
all: build test lint

# Subsequent dependencies (run after, with &&)
deploy: test && notify clean

# Pass arguments to dependencies
default: (build "release")

build target:
  cargo build --profile {{target}}
```

## Attributes

```just
[private]                    # hide from --list
[confirm("Delete everything?")]  # require confirmation
[no-cd]                      # don't cd to justfile dir
[no-exit-message]            # suppress error output
[linux] [macos] [windows]    # OS-specific recipes
[group('dev')]               # group in --list output
[script('python3')]          # run as script

[doc("Run the full test suite")]
test:
  cargo test
```

## Shebang Recipes

```just
analyze:
  #!/usr/bin/env python3
  import json
  print(json.dumps({"status": "ok"}))

setup:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "ready"
```

## Settings

```just
set shell := ["bash", "-uc"]
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
set dotenv-load                  # load .env file
set export                       # export all variables
set positional-arguments         # $1, $2, $@ in recipes
set quiet                        # don't echo recipe lines
set fallback                     # search parent dirs
set ignore-comments              # ignore # in recipe bodies
set allow-duplicate-recipes      # later recipes override
set working-directory := "/tmp"
set tempdir := "/tmp"
```

## Modules & Imports

```just
# Import: inline another file
import 'ci.just'
import? 'local.just'       # optional (no error if missing)

# Module: namespaced subcommand
mod deploy                  # loads deploy.just or deploy/mod.just
mod? staging                # optional

# Run with: just deploy build
# Or: just deploy::build
```

## Line Prefixes

| Prefix | Effect |
|--------|--------|
| `@` | Suppress echo (quiet line) |
| `-` | Ignore errors |
| `-@` / `@-` | Both |

## Common Built-in Functions

**System:** `arch()` `os()` `os_family()` `num_cpus()`

**Environment:** `env(key)` `env(key, default)`

**Paths:** `justfile()` `justfile_dir()` `invocation_dir()` `source_file()` `source_dir()`

**Path ops:** `absolute_path(p)` `canonicalize(p)` `extension(p)` `file_name(p)` `file_stem(p)` `parent_dir(p)` `without_extension(p)` `join(a, b)` `clean(p)`

**Strings:** `replace(s, from, to)` `replace_regex(s, re, to)` `trim(s)` `trim_start(s)` `trim_end(s)` `quote(s)` `uppercase(s)` `lowercase(s)` `capitalize(s)`

**Case:** `snakecase(s)` `kebabcase(s)` `uppercamelcase(s)` `lowercamelcase(s)` `titlecase(s)` `shoutysnakecase(s)`

**Hashing:** `sha256(s)` `sha256_file(p)` `blake3(s)` `blake3_file(p)`

**Misc:** `uuid()` `choose(n, alphabet)` `datetime(fmt)` `datetime_utc(fmt)` `error(msg)` `shell(cmd, args...)` `require(name)` `path_exists(p)` `read(p)` `is_dependency()`

**Constants:** `HEX` `HEXLOWER` `HEXUPPER` `PATH_SEP` `BOLD` `RED` `GREEN` `YELLOW` `BLUE` `NORMAL` (and more ANSI codes)

## CLI Quick Reference

```sh
just                     # run default recipe
just recipe arg1 arg2    # run recipe with args
just --list              # list recipes
just --list --unsorted   # list in justfile order
just --summary           # recipe names only
just --show recipe       # print recipe source
just --evaluate          # print all variables
just --evaluate var      # print one variable
just --fmt               # format justfile
just --check             # syntax check only
just --dry-run recipe    # print without running
just --choose            # interactive picker (fzf)
just --groups            # list recipe groups
just --dump              # print parsed justfile
just --yes recipe        # auto-confirm [confirm]
just VAR=val recipe      # override variable
just dir/recipe          # run from subdirectory
just mod::recipe         # run module recipe
```

## Aliases

```just
alias b := build
alias t := test

[private]
alias c := check
```
