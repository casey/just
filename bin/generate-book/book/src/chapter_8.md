## Backwards Compatibility

With the release of version 1.0, `just` features a strong commitment to backwards compatibility and stability.

Future releases will not introduce backwards incompatible changes that make existing `justfile`s stop working, or break working invocations of the command-line interface.

This does not, however, preclude fixing outright bugs, even if doing so might break `justfiles` that rely on their behavior.

There will never be a `just` 2.0. Any desirable backwards-incompatible changes will be opt-in on a per-`justfile` basis, so users may migrate at their leisure.

Features that aren’t yet ready for stabilization are gated behind the `--unstable` flag. Features enabled by `--unstable` may change in backwards incompatible ways at any time.