# Migration Notes

## Current Python Surface

The current Python package exposes dependency-free UX helpers plus an optional
installed Rust-backed CPU extension at `montepath._native` when installed from
a built wheel. It exposes typed configs, pricing helpers, Greek helpers,
benchmark helpers, and method recommendations.

## Compiled Bindings

Compiled bindings preserve these concepts:

- immutable typed configs
- structured result objects
- `manifest`
- `explain()`
- `reproduce()`
- explicit `McConfigurationError` codes

Timing-sensitive CPU workflows should use the native bridge functions when
`native_runtime_status().available` is true. The pure-Python helpers remain
useful for examples, testing, and agent preflight.

## Stability Expectations

- Config field names are intended to be stable within the `0.1.x` line.
- Result manifests may add fields, but existing keys should not be removed
  without migration notes.
- Unsupported accelerator behavior must remain explicit.
