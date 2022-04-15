### Aliases

Aliases allow recipes to be invoked with alternative names:

````make
alias b := build

build:
  echo 'Building!'
````

````sh
$ just b
build
echo 'Building!'
Building!
````