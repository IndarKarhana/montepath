# Contributing

## Working Agreements

- Follow `docs/repository-rules.md`.
- Follow architecture and design docs in `docs/`.
- Keep `roadmap.md` current.
- Use TDD as default workflow.
- Follow `docs/competitive-benchmark-policy.md`.
- Treat user-friendliness as a core quality requirement (not optional polish).

## Development Workflow

1. Sync understanding with architecture docs.
2. Add or update failing tests for the target behavior.
3. Implement the smallest correct change.
4. Refactor for clarity and performance.
5. Run formatting and tests.
6. Update docs and roadmap.

## Quality Checklist

- Is behavior validated by tests?
- Are errors actionable and typed?
- Are dependencies justified?
- Is hot-path behavior allocation-conscious?
- Are docs and roadmap updated?

## Scope Discipline

- Keep changes focused and reviewable.
- Avoid speculative complexity.
- Prefer stable interfaces and incremental capability growth.
