
# `just`

![crates.io version](https://img.shields.io/crates/v/just.svg)
![build status](https://github.com/casey/just/workflows/Build/badge.svg)
![downloads](https://img.shields.io/github/downloads/casey/just/total.svg)
![chat on discord](https://img.shields.io/discord/695580069837406228?logo=discord)
![say thanks](https://img.shields.io/badge/Say%20Thanks-!-1EAEDB.svg)

`just` is a handy way to save and run project-specific commands.

(非官方中文文档,[这里](https://github.com/chinanf-boy/just-zh),快看过来!)

Commands, called recipes, are stored in a file called `justfile` with syntax inspired by `make`:

![screenshot](screenshot.png)

You can then run them with `just RECIPE`:

````sh
$ just test-all
cc *.c -o main
./test --all
Yay, all your tests passed!
````

`just` has a ton of useful features, and many improvements over `make`:

* `just` is a command runner, not a build system, so it avoids much of [`make`’s complexity and idiosyncrasies](#what-are-the-idiosyncrasies-of-make-that-just-avoids). No need for `.PHONY` recipes!

* Linux, MacOS, and Windows are supported with no additional dependencies. (Although if your system doesn’t have an `sh`, you’ll need to [choose a different shell](#shell).)

* Errors are specific and informative, and syntax errors are reported along with their source context.

* Recipes can accept [command line arguments](#recipe-parameters).

* Wherever possible, errors are resolved statically. Unknown recipes and circular dependencies are reported before anything runs.

* `just` [loads `.env` files](#dotenv-integration), making it easy to populate environment variables.

* Recipes can be [listed from the command line](#listing-available-recipes).

* Command line completion scripts are [available for most popular shells](#shell-completion-scripts).

* Recipes can be written in [arbitrary languages](#writing-recipes-in-other-languages), like Python or NodeJS.

* `just` can be invoked from any subdirectory, not just the directory that contains the `justfile`.

* And [much more](#manual)!

If you need help with `just` please feel free to open an issue or ping me on [Discord](https://discord.gg/ezYScXR). Feature requests and bug reports are always welcome!