just
====

[![crates.io version](https://img.shields.io/crates/v/just.svg)](https://crates.io/crates/just)

`just` is a handy way to run project-specific commands.

getting started
---------------

Get `just` from cargo, the [rust language](https://www.rust-lang.org) package manager:

1. Get rust and cargo from [rustup.rs](https://www.rustup.rs)
2. Run `cargo install just`
3. Add `~/.cargo/bin` to your PATH

`just` depends on `make` or `gmake` to actually run commands, but if you're using unix, a make is probably already installed on your system. If not, you can get it from your friendly local package manager.

Unfortunately, the dependency on `make` makes `just` difficult to run on windows. A possible future goal is to stop depending on make and use a custom file format, discussed in [issue #1](https://github.com/casey/just/issues/1).

Once you can run `just`, create a file called `justfile` in the root of your project, and start adding recipes to it. See an example `justfile` [here](https://github.com/casey/j/blob/master/justfile).

The first recipe in the `justfile` will be run when `just` is called with no arguments, which makes it a good candidate for the command that you run most often, for example building and testing your project. Other recipes can be run by supplying their name as an argument, for example `just build` to run the `build` recipe.

After that, the sky is the limit!

Ideas for recipes include:

* Deploying/publishing the project
* Building in release mode vs debug mode
* Running in debug mode/with logging enabled
* Complex git workflows
* Updating dependencies
* Running different sets of tests, for example fast tests vs slow tests
* Any complex set of commands that you really should write down somewhere, if only to be able to remember them

how it works
------------

`just` looks upward from the current directory for a file called `justfile` and then runs `make` with that file as the makefile. `just` also sets the current working directory to where it found the justfile, so your commands are executed from the root of your project and not from whatever subdirectory you happen to be in.

Makefile targets are called "recipes", and are simply lists of commands to run in sequence, making them very concise. Recipes stop if a command fails, like if you do `set -e` in a shell script. Recipes also print each command before running it. If you would like to supress this, you can prepend a line in a recipe with `@`.

With no arguments `just` runs the default recipe:

`just`

Adding one argument specifies the recipe:

`just compile`

Multiple recipes can be run in order:

`just lint compile test`

Arguments after `--` are exported to environment variables`ARG0`, `ARG1`, ..., `ARGN`, which can be used in the justfile. To run recipe `compile` and export `ARG0=bar` and `ARG1=baz`:

`just compile -- bar baz`

further ramblings
-----------------

`just` is a trivial program, but I personally find it enormously useful and write a `justfile` for almost every project, big or small.

For one, `just` is a full 5 characters shorter than `./main`, and 3 characters shorter than `make`.

On a big project with multiple contributers, it's very useful to have a file with all the commands needed to work on the project. There are probably different commands to test, build, lint, deploy, and the like, and having them all in one place is useful and cuts down on the time you have to spend telling people which commands to run and how to type them. And, with an easy place to put commands, it's likely that you'll come up with other useful things which are part of the project's collective wisdom, but which aren't written down anywhere, like the arcane commands needed for your project's revision control workflow, for updating dependencies, or all the random flags you might need to pass to the build system.

Even for small, personal projects, it's nice to be able to go into an old project written in some random language with some mysterious build system and know that all the commands you need to do whatever you need to do are in the justfile, and that if you type `just` something useful will probably happen.

If you have a feature request, do open an issue and let me know.

I hope you enjoy using `just`, and find great success and satisfaction in all your computational endeavors!

😸
