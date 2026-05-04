# Migration Notes

## Current Python Surface

The current Python package is dependency-free and pure Python for UX helpers.
It exposes typed configs, pricing helpers, Greek helpers, benchmark helpers, and
method recommendations.

## Future Compiled Bindings

Compiled bindings should preserve these concepts:

- immutable typed configs
- structured result objects
- `manifest`
- `explain()`
- `reproduce()`
- explicit `McConfigurationError` codes

Timing-sensitive workflows should migrate to compiled bindings when they land.
The pure-Python helpers should remain useful for examples, testing, and agent
preflight.

## Stability Expectations

- Config field names are intended to be stable within the `0.1.x` line.
- Result manifests may add fields, but existing keys should not be removed
  without migration notes.
- Unsupported accelerator behavior must remain explicit.

