# Environment Variables

Assignments prefixed with the `export` keyword will be exported to recipes as environment variables:

```make
export RUST_BACKTRACE := "1"

test:
    # will print a stack trace if it crashes
    cargo test
```