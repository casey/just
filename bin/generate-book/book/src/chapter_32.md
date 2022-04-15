### Setting Variables from the Command Line

Variables can be overridden from the command line.

````make
os := "linux"

test: build
  ./test --test {{os}}

build:
  ./build {{os}}
````

````sh
$ just
./build linux
./test --test linux
````

Any number of arguments of the form `NAME=VALUE` can be passed before recipes:

````sh
$ just os=plan9
./build plan9
./test --test plan9
````

Or you can use the `--set` flag:

````sh
$ just --set os bsd
./build bsd
./test --test bsd
````