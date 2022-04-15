### The Default Recipe

When `just` is invoked without a recipe, it runs the first recipe in the `justfile`. This recipe might be the most frequently run command in the project, like running the tests:

````make
test:
  cargo test
````

You can also use dependencies to run multiple recipes by default:

````make
default: lint build test

build:
  echo Building…

test:
  echo Testing…

lint:
  echo Linting…
````

If no recipe makes sense as the default recipe, you can add a recipe to the beginning of your `justfile` that lists the available recipes:

````make
default:
  just --list
````