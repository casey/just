# Test Refactoring Plan

## Goal

Reduce the number of separate tests and verbosity while still exercising the code in the same way.

## Strategies

### 1. Parameterized / data-driven tests (biggest win)

Many tests differ only in input/output data. Use loop-in-a-single-test:

```rust
#[test]
fn string_functions() {
  for (func, input, expected) in [
    ("uppercase", "bar", "BAR"),
    ("lowercase", "BAR", "bar"),
    // ...
  ] {
    Test::new()
      .justfile(&format!("foo:\n  echo {{{{ {func}('{input}') }}}}"))
      .stdout(&format!("{expected}\n"))
      .stderr(&format!("echo {expected}\n"))
      .success();
  }
}
```

Applies to:
- String function tests in `functions.rs` (~20+ tests → 1-2)
- Path function error tests in `functions.rs` (~6+ tests → 1)
- Case conversion tests throughout

### 2. Merge tests that exercise the same feature with slight variations

Tests like `test_os_arch_functions_in_interpolation`, `_in_expression`, and `_in_default` repeat identical stdout/stderr assertions — only the justfile differs. These could be a single test with a loop over justfile variants.

Similarly in `options.rs`, tests sharing the `LONG_SHORT` constant justfile could be one test that runs multiple arg combinations.

### 3. Macro-generated test functions (preserves individual test names)

For cases where individual test names are important:

```rust
macro_rules! test_function {
  ($name:ident, $func:expr, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      Test::new()
        .justfile(&format!("foo:\n  echo {{{{ {}('{}') }}}}", $func, $input))
        .stdout(&format!("{}\n", $expected))
        .stderr(&format!("echo {}\n", $expected))
        .success();
    }
  };
}

test_function!(uppercase, "uppercase", "bar", "BAR");
test_function!(lowercase, "lowercase", "BAR", "bar");
```

### 4. Consolidate multi-assertion unit tests

Group related assertions into fewer test functions, similar to what the inline `#[cfg(test)]` unit tests already do.

### 5. Extract shared error-format helpers

The "broken function" error tests construct error messages with the same format. A helper function would cut significant duplication.

## Priority Areas

| Area | Current tests | Could become | Technique |
|------|--------------|-------------|-----------|
| String functions (`functions.rs`) | ~20 | 1-2 | Data-driven loop |
| Path error tests (`functions.rs`) | ~6 | 1 | Data-driven loop |
| OS/arch placement tests | 3 | 1 | Loop over justfile variants |
| Options arg-passing tests | ~10 | 2-3 | Merge by shared justfile |
| `misc.rs` listing variants | many groups of 3-5 | fewer | Merge closely related ones |
