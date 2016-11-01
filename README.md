just
====

[![crates.io version](https://img.shields.io/crates/v/just.svg)](https://crates.io/crates/just)

`just` is a handy way to save and run commands.

Commands are stored in a file called `justfile` with a syntax inspired by `make`:

```make
# test everything. must build first
test-all: build
  test --all

# run a specific test by passing it as an argument: `just test server-test`
test TEST: build
  test --test {{TEST}}

# build the binary
build:
  cc *.c -o main

version = "0.2.0"
tardir  = "awesomesauce-" + version
tarball = tardir + ".tar.gz"

build-tarball:
	rm -f {{tarball}}
	mkdir {{tardir}}
	cp README.md *.c {{tardir}}
	tar zcvf {{tarball}} {{tardir}}
	rm -rf {{tardir}}

publish: test build-tarball
  scp {{tarball}} me@server.com:release/

# recipes can be written in any language
serve-docs:
	#!/usr/bin/env python3
	import os, http.server, socketserver
	PORT = 8000
	Handler = http.server.SimpleHTTPRequestHandler
  os.chdir('docs')
	httpd = socketserver.TCPServer(("", PORT), Handler)
	print("serving at port", PORT)
	httpd.serve_forever()
```

`just` avoids `make`'s idiosyncrasies and produces excellent error messages, so debugging a `justfile` is easier and less suprising than debugging a makefile.

getting started
---------------

`just` should run on any system with a reasonable `sh`, and can be installed with `cargo`, the [rust language](https://www.rust-lang.org) package manager:

1. Get rust and cargo from [rustup.rs](https://www.rustup.rs)
2. Run `cargo install just`
3. Add `~/.cargo/bin` to your PATH

Then, create a file called `justfile` in the root of your project and start adding recipes to it.

Optionally, you can `alias j=just` for lighting fast command running.
