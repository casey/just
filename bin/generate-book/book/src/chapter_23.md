### Documentation Comments

Comments immediately preceding a recipe will appear in `just --list`:

````make
# build stuff
build:
  ./bin/build

# test stuff
test:
  ./bin/test
````

````sh
$ just --list
Available recipes:
    build # build stuff
    test # test stuff
````