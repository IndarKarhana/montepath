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

`uv` equivalent:

```bash
uv pip install -e ".[dev]"
uv run cargo fmt --all --check
uv run cargo test -q
uv run python -m pytest -q
uv run python -m build
```

Smoke the built distributions:

```bash
python -m venv /tmp/montepath-wheel-smoke
/tmp/montepath-wheel-smoke/bin/python -m pip install dist/*.whl
/tmp/montepath-wheel-smoke/bin/python python/tests/installed_package_smoke.py

python -m venv /tmp/montepath-sdist-smoke
/tmp/montepath-sdist-smoke/bin/python -m pip install dist/*.tar.gz
/tmp/montepath-sdist-smoke/bin/python python/tests/installed_package_smoke.py
```

## Benchmark Artifact Refresh

```bash
cargo run -p mc-bench --release -- --output benchmarks/release-results.json
```

Use the benchmark table format requested by project convention: include
`(ours)` next to our implementation and compare timing plus accuracy metrics.

## Publish Readiness

- PyPI project name `montepath` is still available immediately before
  publishing.
- Hosted repository URLs point at the renamed GitHub repository.
- PyPI has a pending trusted publisher for:
  - PyPI project name: `montepath`
  - owner: `IndarKarhana`
  - repository: `montepath`
  - workflow: `publish-pypi.yml`
  - environment: `pypi`
- All new public functions are in `docs/function-catalog.md`.
- Unsupported behavior is documented explicitly.
- Benchmark claims match committed artifacts.
- Wheel and source distribution build successfully.
- Wheel and source distribution install smokes pass.
- Changelog entry exists.
- Public alpha limitations are current in `docs/public-alpha.md`.
- uv/MCP install guidance is current in `docs/uv-and-agent-install.md`.

## Publishing

Preferred publishing path is PyPI trusted publishing from
`.github/workflows/publish-pypi.yml`. Create a GitHub release or run the
workflow manually after the PyPI pending publisher is configured. The workflow
builds the source distribution plus manylinux x86_64, macOS universal2, and
Windows x64 wheels before uploading through trusted publishing. The publish step
checks PyPI for existing files so trusted-publishing reruns can upload missing
supplemental wheels without republishing artifacts that already exist.

If publishing manually, use a scoped PyPI token from a verified environment and
publish only after the checks above pass:

```bash
python -m build
uv publish
```

Do not publish local dirty or unverified artifacts.

## Post-Publish Verification

```bash
uv pip install montepath
uvx --from montepath montepath-mcp
```

Also verify that `uv add montepath` works inside a fresh project environment.
