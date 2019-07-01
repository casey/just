# Listing Available Recipes

Recipes can be listed with `just --list` :

```sh
$ just --list
Available recipes:
  build
  test
  deploy
  lint
```

`just --summary` is more concise:

```sh
$ just --summary
build test deploy lint