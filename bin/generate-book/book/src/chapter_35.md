### Running Recipes at the End of a Recipe

Normal dependencies of a recipes always run before a recipe starts. That is to say, the dependee always runs before the depender. These dependencies are called “prior dependencies”.

A recipe can also have subsequent dependencies, which run after the recipe and are introduced with an `&&`:

````make
a:
  echo 'A!'

b: a && c d
  echo 'B!'

c:
  echo 'C!'

d:
  echo 'D!'
````

…running *b* prints:

````sh
$ just b
echo 'A!'
A!
echo 'B!'
B!
echo 'C!'
C!
echo 'D!'
D!
````