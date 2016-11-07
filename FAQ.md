Frequently Asked Questions
==========================

### What are the idiosyncrasies of make that just avoids?

Make has some behaviors which are either confusing, complicated, or make it unsuitable for use as a general command runner.

One example is that sometimes make won't run the commands in a recipe. For example, if you have a file called `test` and the the following makefile that runs it:

```make
test:
  ./test
```

Make will actually refuse to run it:

```sh
$ make test
make: `test' is up to date.
```

Make see the recipe `test` and assumes that it produces a file called `test`. It then sees that this file exists and thus assumes that the recipe doesn't need to be run.

To be fair, this behavior is desirable when using make as a build system, but not when using it as a command runner.

Some other examples include having to understand the difference between `=` and `:=` assignment, the confusing error messages that can be produced if you mess up your makefile, having to use `$$` to write recipes that use environment variables, and incompatibilites between different flavors of make.

### What's the relationship between just and cargo build scripts?

[Cargo build scripts](http://doc.crates.io/build-script.html) have a pretty specific use, which is to control how cargo builds your rust project. This might include adding flags to `rustc` invocations, building an external dependency, or running some kind of codegen step.

`just`, on the other hand, is for all the other miscellaneous commands you might run as part of development. Things like running tests in different configurations, linting your code, pushing build artifacts to a server, removing temporary files, and the like.

Also, although `just` is written in rust, it can be used regardless of the language or build system your project uses.
