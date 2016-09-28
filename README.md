j
=

`j` is a handy way to run project-specific commands.

`j` looks upward from the current directory for a file called `justfile` and then runs make with that file as the makefile. `j` also sets the current working directory to where it found the justfile, so your commands are executed from the root of your project and not from whatever subdirectory you happen to be in.

With no arguments it runs the default recipe:

`j`

Adding one argument specifies the recipe:

`j compile`

Multiple recipes can be run in order:

`j lint compile test`

Arguments after `--` are exported as `ARG0, ARG1, ..., ARGN`, which can be used in the justfile. To run recipe `compile` and export `ARG0=bar` and `ARG1=baz`:

`just compile -- bar baz`

By way of example, the included justfile has a pinch of fanciful fluff.

getting j
---------

J is distributed via `cargo`, rust's package manager.

1. Get cargo at [rustup.rs](https://www.rustup.rs)
2. Run `cargo install j`
3. Add `~/.cargo/bin` to your PATH

`j` depends on make to actually run commands, but hopefully if you're on a unix, make is already installed. If not, you can get it from friendly local package manager.
