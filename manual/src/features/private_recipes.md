# Private Recipes

Recipes and aliases whose name starts with a `_` are omitted from `just --list`:

```make
test: _test-helper
  ./bin/test

_test-helper:
  ./bin/super-secret-test-helper-stuff
```

```sh
$ just --list
Available recipes:
  test
```

And from `just --summary`:

```sh
$ just --summary
test
```

This is useful for helper recipes which are only meant to be used as dependencies of other recipes.