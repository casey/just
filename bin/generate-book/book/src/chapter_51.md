### Falling back to parent `justfile`s

If a recipe is not found, `just` will look for `justfile`s in the parent
directory and up, until it reaches the root directory.

This feature is currently unstable, and so must be enabled with the
`--unstable` flag.

As an example, suppose the current directory contains this `justfile`:

````make
foo:
  echo foo
````

And the parent directory contains this `justfile`:

````make
bar:
  echo bar
````

````sh
$ just --unstable bar
Trying ../justfile
echo bar
bar
````