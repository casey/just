### Getting and Setting Environment Variables

#### Exporting `just` Variables

Assignments prefixed with the `export` keyword will be exported to recipes as environment variables:

````make
export RUST_BACKTRACE := "1"

test:
  # will print a stack trace if it crashes
  cargo test
````

Parameters prefixed with a `$` will be exported as environment variables:

````make
test $RUST_BACKTRACE="1":
  # will print a stack trace if it crashes
  cargo test
````

Exported variables and parameters are not exported to backticks in the same scope.

````make
export WORLD := "world"
# This backtick will fail with "WORLD: unbound variable"
BAR := `echo hello $WORLD`
````

````make
# Running `just a foo` will fail with "A: unbound variable"
a $A $B=`echo $A`:
  echo $A $B
````

When [export](#export) is set, all `just` variables are exported as environment variables.

#### Getting Environment Variables from the environment

Environment variables from the environment are passed automatically to the recipes.

````make
print_home_folder:
  echo "HOME is: '${HOME}'"
````

````sh
$ just
HOME is '/home/myuser'
````

#### Loading Environment Variables from a `.env` File

`just` will load environment variables from a `.env` file if [dotenv-load](#dotenv-load) is set. The variables in the file will be available as environment variables to the recipes. See [dotenv-integration](#dotenv-integration) for more information.

#### Setting `just` Variables from Environments Variables

Environment variables can be propagated to `just` variables using the functions `env_var()` and `env_var_or_default()`.
See [environment-variables](#environment-variables).