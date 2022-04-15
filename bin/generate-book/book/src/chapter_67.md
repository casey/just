### What’s the relationship between Just and Cargo build scripts?

[`cargo` build scripts](http://doc.crates.io/build-script.html) have a pretty specific use, which is to control how `cargo` builds your Rust project. This might include adding flags to `rustc` invocations, building an external dependency, or running some kind of codegen step.

`just`, on the other hand, is for all the other miscellaneous commands you might run as part of development. Things like running tests in different configurations, linting your code, pushing build artifacts to a server, removing temporary files, and the like.

Also, although `just` is written in Rust, it can be used regardless of the language or build system your project uses.