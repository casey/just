### What are the idiosyncrasies of Make that Just avoids?

`make` has some behaviors which are confusing, complicated, or make it unsuitable for use as a general command runner.

One example is that under some circumstances, `make` won’t actually run the commands in a recipe. For example, if you have a file called `test` and the following makefile:

````make
test:
  ./test
````

`make` will refuse to run your tests:

````sh
$ make test
make: `test' is up to date.
````

`make` assumes that the `test` recipe produces a file called `test`. Since this file exists and the recipe has no other dependencies, `make` thinks that it doesn’t have anything to do and exits.

To be fair, this behavior is desirable when using `make` as a build system, but not when using it as a command runner. You can disable this behavior for specific targets using `make`’s built-in [`.PHONY` target name](https://www.gnu.org/software/make/manual/html_node/Phony-Targets.html), but the syntax is verbose and can be hard to remember. The explicit list of phony targets, written separately from the recipe definitions, also introduces the risk of accidentally defining a new non-phony target. In `just`, all recipes are treated as if they were phony.

Other examples of `make`’s idiosyncrasies include the difference between `=` and `:=` in assignments, the confusing error messages that are produced if you mess up your makefile, needing `$$` to use environment variables in recipes, and incompatibilities between different flavors of `make`.