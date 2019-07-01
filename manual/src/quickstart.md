# Quick Start

See [Installation](install/index.md) for how to install `just` on your computer. Try running `just --version` to make sure that it's installed correctly.

Once `just` is installed and working, create a file named `justfile` in the root of your project with the following contents:

```make
recipe-name:
    echo 'This is a recipe!'

# this is a comment
another-recipe:
    @echo 'This is another recipe.'
```

When you invoke `just` it looks for file `justfile` in the current directory and upwards, so you can invoke it from any subdirectory of your project.

The search for a `justfile` is case insensitive, so any case, like `Justfile`, `JUSTFILE`, or `JuStFiLe`, will work.

Running `just` with no arguments runs the first recipe in the `justfile`:

```sh
$ just
echo 'This is a recipe!'
This is a recipe!
```

One or more arguments specify the recipe(s) to run:

```sh
$ just another-recipe
This is another recipe.
```

`just` prints each command to standard error before running it, which is why `echo 'This is a recipe!'` was printed. This is suppressed for lines starting with `@`, which is why `echo 'Another recipe.'` was not printed.

Recipes stop running if a command fails. Here `cargo publish` will only run if `cargo test` succeeds:

```make
publish:
    cargo test
    # tests passed, time to publish!
    cargo publish
```

Recipes can depend on other recipes. Here the `test` recipe depends on the `build` recipe, so `build` will run before `test`:

```make
build:
    cc main.c foo.c bar.c -o main

test: build
    ./test

sloc:
    @echo "`wc -l *.c` lines of code"
```

```sh
$ just test
cc main.c foo.c bar.c -o main
./test
testing... all tests passed!
```

Recipes without dependencies will run in the order they're given on the command line:

```sh
$ just build sloc
cc main.c foo.c bar.c -o main
1337 lines of code
```

Dependencies will always run first, even if they are passed after a recipe that depends on them:

```sh
$ just test build
cc main.c foo.c bar.c -o main
./test
testing... all tests passed!
```
