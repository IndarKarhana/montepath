# Release Checklist

## Versioning

This project follows semantic versioning while the Python package is pre-1.0:

- patch: docs, packaging, or compatible helper additions
- minor: new public helpers, new workload configs, or compatible manifest fields
- major: reserved for future stable 1.0 breaking changes

Update:

- `pyproject.toml`
- `CHANGELOG.md`
- `docs/site/migration-notes.md` when behavior changes
- `docs/function-catalog.md` for public surfaces

## Local Checks

```bash
cargo fmt --all --check
cargo test -q
PYTHONPATH=python python -m pytest -q
python -m build
```

## Benchmark Artifact Refresh

```bash
cargo run -p mc-bench --release -- --output benchmarks/release-results.json
```

Use the benchmark table format requested by project convention: include
`(ours)` next to our implementation and compare timing plus accuracy metrics.

## Publish Readiness

- All new public functions are in `docs/function-catalog.md`.
- Unsupported behavior is documented explicitly.
- Benchmark claims match committed artifacts.
- Wheel and source distribution build successfully.
- Changelog entry exists.

