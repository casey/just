### Selecting Recipes to Run With an Interactive Chooser

The `--choose` subcommand makes `just` invoke a chooser to select which recipes to run. Choosers should read lines containing recipe names from standard input and print one or more of those names separated by spaces to standard output.

Because there is currently no way to run a recipe that requires arguments with `--choose`, such recipes will not be given to the chooser. Private recipes and aliases are also skipped.

The chooser can be overridden with the `--chooser` flag. If `--chooser` is not given, then `just` first checks if `$JUST_CHOOSER` is set. If it isn’t, then the chooser defaults to `fzf`, a popular fuzzy finder.

Arguments can be included in the chooser, i.e. `fzf --exact`.

The chooser is invoked in the same way as recipe lines. For example, if the chooser is `fzf`, it will be invoked with `sh -cu 'fzf'`, and if the shell, or the shell arguments are overridden, the chooser invocation will respect those overrides.

If you’d like `just` to default to selecting recipes with a chooser, you can use this as your default recipe:

````make
default:
  @just --choose
````