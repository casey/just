just
====

[![crates.io version](https://img.shields.io/crates/v/just.svg)](https://crates.io/crates/just)

`just` is a handy way to save and run commands.

Commands are stored in a file called `justfile` or `Justfile` with syntax inspired by `make`:

```make
test-all: build
    ./test --all

test TEST: build
    ./test --test {{TEST}}

build:
    cc *.c -o main
```

`just` produces detailed error messages and avoids `make`'s idiosyncrasies, so debugging a justfile is easier and less surprising than debugging a makefile.

If you need help with `just` please feel free to send me an email. Feature requests and bug reports are also always welcome!


getting started
---------------

`just` should run on any system with a reasonable `sh` and can be installed with `cargo`, the [rust language](https://www.rust-lang.org) package manager:

1. Install rust and cargo by following the instructions at [rustup.rs](https://www.rustup.rs)
2. Run `cargo install just`
3. Add `~/.cargo/bin` to your PATH

Optionally, you can also `alias j=just` for lighting fast command running.


How do I just?
--------------

Once `just` is working, create a file called `justfile` in the root of your project and start adding recipes to it.

Recipes look like this:

```make
recipe-name:
    echo 'This is a recipe!'

another-recipe:
    @echo 'Another recipe.'
```

Running `just` with no arguments runs the first recipe in the `justfile`:

```sh
$ just
echo 'This is a recipe!'
This is a recipe!
```

When you invoke `just` it looks for a `justfile` in the current directory and upwards, so you can invoke `just` from any subdirectory of your project.

One or more arguments specify the recipes to run:

```sh
$ just another-recipe
Another recipe.
```

`just` prints each command to standard error before running it, which is why `echo 'This is a recipe!'` was printed. Lines starting with `@` will not be printed which is why `echo 'Another recipe.'` was not. printed.

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

Assignment, strings, concatination, and substitution with `{{...}}` are supported:

```make
version = "0.2.7"
tardir  = "awesomesauce-" + version
tarball = tardir + ".tar.gz"

publish:
    rm -f {{tarball}}
    mkdir {{tardir}}
    cp README.md *.c {{tardir}}
    tar zcvf {{tarball}} {{tardir}}
    scp {{tarball}} me@server.com:release/
    rm -rf {{tarball}} {{tardir}}
```

Double-quoted strings support escape sequences:

```make
string-with-tab             = "\t"
string-with-newline         = "\n"
string-with-carriage-return = "\r"
string-with-double-quote    = "\""
string-with-slash           = "\\"
```

```sh
$ just --evaluate
"tring-with-carriage-return = "
string-with-double-quote = """
string-with-newline = "
"
string-with-slash = "\"
string-with-tab = "     "
```

Single-quoted strings do not support any escape sequences:

```make
raw-string = '\t\n\r\"\\'
```

```sh
$ just --evaluate
raw-string = "\t\n\r\"\\"
```

Recipes may take arguments. Here recipe `build` takes an argument called `target`:

```make
build target:
    @echo 'Building {{target}}...'
    cd {{target}} && make
```

Only one recipe that takes arguments may given on the command line, and other recipes may not depend on it. To pass arguments put them after the recipe name:

```sh
$ just build my-awesome-project
Building my-awesome-project...
cd my-awesome-project && make
```

Variables can be exported to recipes as environment variables:

```make
export RUST_BACKTRACE = "1"

test:
    # will print a stack trace if it crashes
    cargo test
```

Variables can also be overridden from the command line:

```make
os = "linux"

test: build
    ./test --test {{os}}

build:
    ./build {{os}}
```

```sh
$ just
./build linux
./test --test linux
```

You can pass any number of arguments of the form `NAME=VALUE` before recipes:

```sh
$ just os=plan9
./build plan9
./test --test plan9
```

Or you can use the `--set` flag:

```sh
$ just --set os bsd
./build bsd
./test --test bsd
```

Backticks can be used to store the result of commands:

```make
localhost = `dumpinterfaces | cut -d: -f2 | sed 's/\/.*//' | sed 's/ //g'`

serve:
    ./serve {{localhost}} 8080
```


Recipes that start with a `#!` are executed as scripts, so you can write recipes in other languages:

```make
polyglot: python js perl sh ruby

python:
    #!/usr/bin/env python3
    print('Hello from python!')

js:
    #!/usr/bin/env node
    console.log('Greetings from JavaScript!')

perl:
    #!/usr/bin/env perl
    print "Larry Wall says Hi!\n";

sh:
    #!/usr/bin/env sh
    hello='Yo'
    echo "$hello from a shell script!"

ruby:
    #!/usr/bin/env ruby
    puts "Hello from ruby!"
```

```sh
$ just polyglot
Hello from python!
Greetings from JavaScript!
Larry Wall says Hi!
Yo from a shell script!
Hello from ruby!
```

`just` also supports a number of useful command line options for listing, dumping, and debugging recipes and variable:

```sh
$ just --list
js perl polyglot python ruby
$ just --show perl
perl:
    #!/usr/bin/env perl
    print "Larry Wall says Hi!\n";
$ just --show polyglot
polyglot: python js perl sh ruby
```

Run `just --help` to see all the options.


miscellanea
-----------

### syntax hilighting

`justfile` syntax is close enough to `make` that you may want to tell your editor to use make syntax hilighting for just.

For vim, you can put the following in `~/.vim/filetype.vim`:

```vimscript
if exists("did_load_filetypes")
  finish
endif

augroup filetypedetect
  au BufNewFile,BufRead justfile setf make
augroup END
```

Feel free to send me the commands necessary to get syntax hilighting working in your editor of choice so that I may include them here.

### justfile grammar

A description of the grammar of justfiles can be found in [grammar.md](grammar.md).

### just.sh

Before `just` was a bloated rust program it was a tiny shell script that called `make`. If you can't or would rather not rust you can find the old version in [extras/just.sh](extras/just.sh).

### non-project specific justfile

If you want some commands to be available everwhere, put them in `~/.justfile` and add the following to your shell's initialization file:

```sh
alias .j='just --justfile ~/.justfile --working-directory ~'
```

Or, if you'd rather they run in the current directory:

```sh
alias .j='just --justfile ~/.justfile --working-directory .'
```


further ramblings
-----------------

`just` is a trivial program, but I personally find it very useful to write a `justfile` for almost every project, big or small.

On a big project with multiple contributers, it's very useful to have a file with all the commands needed to work on the project close at hand.

There are probably different commands to test, build, lint, deploy, and the like, and having them all in one place is useful and cuts down on the time you have to spend telling people which commands to run and how to type them.

And, with an easy place to put commands, it's likely that you'll come up with other useful things which are part of the project's collective wisdom, but which aren't written down anywhere, like the arcane commands needed for some part of your revision control workflow, install all your project's dependencies, or all the random flags you might need to pass to the build system.

Some ideas for recipes:

* Deploying/publishing the project
* Building in release mode vs debug mode
* Running in debug mode or with logging enabled
* Complex git workflows
* Updating dependencies
* Running different sets of tests, for example fast tests vs slow tests, or running them with verbose output
* Any complex set of commands that you really should write down somewhere, if only to be able to remember them

Even for small, personal projects it's nice to be able to remember commands by name instead of ^Reverse searching your shell history, and it's a huge boon to be able to go into an old project written in a random language with a mysterious build system and know that all the commands you need to do whatever you need to do are in the `justfile`, and that if you type `just` something useful (or at least interesting!) will probably happen.

Anyways, I think that's about it for this incredibly long-winded README.

I hope you enjoy using `just` and find great success and satisfaction in all your computational endeavors!

😸
