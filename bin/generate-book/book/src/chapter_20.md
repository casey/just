### Listing Available Recipes

Recipes can be listed in alphabetical order with `just --list`:

````sh
$ just --list
Available recipes:
    build
    test
    deploy
    lint
````

`just --summary` is more concise:

````sh
$ just --summary
build test deploy lint
````

Pass `--unsorted` to print recipes in the order they appear in the `justfile`:

````make
test:
  echo 'Testing!'

build:
  echo 'Building!'
````

````sh
$ just --list --unsorted
Available recipes:
    test
    build
````

````sh
$ just --summary --unsorted
test build
````

If you’d like `just` to default to listing the recipes in the `justfile`, you can use this as your default recipe:

````make
default:
  @just --list
````

The heading text can be customized with `--list-heading`:

````sh
$ just --list --list-heading $'Cool stuff…\n'
Cool stuff…
    test
    build
````

And the indentation can be customized with `--list-prefix`:

````sh
$ just --list --list-prefix ····
Available recipes:
····test
····build
````

The argument to `--list-heading` replaces both the heading and the newline following it, so it should contain a newline if non-empty. It works this way so you can suppress the heading line entirely by passing the empty string:

````sh
$ just --list --list-heading ''
    test
    build
````