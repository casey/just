### Settings

Settings control interpretation and execution. Each setting may be specified at most once, anywhere in the `justfile`.

For example:

````make
set shell := ["zsh", "-cu"]

foo:
  # this line will be run as `zsh -cu 'ls **/*.txt'`
  ls **/*.txt
````

#### Table of Settings

|Name|Value|Description|
|----|-----|-----------|
|`allow-duplicate-recipes`|boolean|Allow recipes appearing later in a `justfile` to override earlier recipes with the same name.|
|`dotenv-load`|boolean|Load a `.env` file, if present.|
|`export`|boolean|Export all variables as environment variables.|
|`positional-arguments`|boolean|Pass positional arguments.|
|`shell`|`[COMMAND, ARGS…]`|Set the command used to invoke recipes and evaluate backticks.|
|`windows-powershell`|boolean|Use PowerShell on Windows as default shell.|

Boolean settings can be written as:

````mf
set NAME
````

Which is equivalent to:

````mf
set NAME := true
````

#### Allow Duplicate Recipes

If `allow-duplicate-recipes` is set to `true`, defining multiple recipes with the same name is not an error and the last definition is used. Defaults to `false`.

````make
set allow-duplicate-recipes

@foo:
  echo foo

@foo:
  echo bar
````

````sh
$ just foo
bar
````

#### Dotenv Load

If `dotenv-load` is `true`, a `.env` file will be loaded if present. Defaults to `false`.

#### Export

The `export` setting causes all `just` variables to be exported as environment variables. Defaults to `false`.

````make
set export

a := "hello"

@foo b:
  echo $a
  echo $b
````

````sh
$ just foo goodbye
hello
goodbye
````

#### Positional Arguments

If `positional-arguments` is `true`, recipe arguments will be passed as positional arguments to commands. For linewise recipes, argument `$0` will be the name of the recipe.

For example, running this recipe:

````make
set positional-arguments

@foo bar:
  echo $0
  echo $1
````

Will produce the following output:

````sh
$ just foo hello
foo
hello
````

When using an `sh`-compatible shell, such as `bash` or `zsh`, `$@` expands to the positional arguments given to the recipe, starting from one. When used within double quotes as `"$@"`, arguments including whitespace will be passed on as if they were double-quoted. That is, `"$@"` is equivalent to `"$1" "$2"`… When there are no positional parameters, `"$@"` and `$@` expand to nothing (i.e., they are removed).

This example recipe will print arguments one by one on separate lines:

````make
set positional-arguments

@test *args='':
  bash -c 'while (( "$#" )); do echo - $1; shift; done' -- "$@"
````

Running it with *two* arguments:

````sh
$ just test foo "bar baz"
- foo
- bar baz
````

#### Shell

The `shell` setting controls the command used to invoke recipe lines and backticks. Shebang recipes are unaffected.

````make
# use python3 to execute recipe lines and backticks
set shell := ["python3", "-c"]

# use print to capture result of evaluation
foos := `print("foo" * 4)`

foo:
  print("Snake snake snake snake.")
  print("{{foos}}")
````

`just` passes the command to be executed as an argument. Many shells will need an additional flag, often `-c`, to make them evaluate the first argument.

##### Windows PowerShell

`just` uses `sh` on Windows by default. To use PowerShell instead, set `windows-powershell` to true.

````make
set windows-powershell := true

hello:
  Write-Host "Hello, world!"
````

##### Python 3

````make
set shell := ["python3", "-c"]
````

##### Bash

````make
set shell := ["bash", "-uc"]
````

##### Z Shell

````make
set shell := ["zsh", "-uc"]
````

##### Fish

````make
set shell := ["fish", "-c"]
````