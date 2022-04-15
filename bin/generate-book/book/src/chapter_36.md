### Running Recipes in the Middle of a Recipe

`just` doesn’t support running recipes in the middle of another recipe, but you can call `just` recursively in the middle of a recipe. Given the following `justfile`:

````make
a:
  echo 'A!'

b: a
  echo 'B start!'
  just c
  echo 'B end!'

c:
  echo 'C!'
````

…running *b* prints:

````sh
$ just b
echo 'A!'
A!
echo 'B start!'
B start!
echo 'C!'
C!
echo 'B end!'
B end!
````

This has limitations, since recipe `c` is run with an entirely new invocation of `just`: Assignments will be recalculated, dependencies might run twice, and command line arguments will not be propagated to the child `just` process.