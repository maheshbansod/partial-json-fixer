# partial-json-fixer

## Agent skills

### Issue tracker

Issues are tracked on GitHub (maheshbansod/partial-json-fixer) via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` + `docs/adr/` at repo root. See `docs/agents/domain.md`.

## Workflow

Incoming GitHub issues go through `/triage` before any implementation; a triaged
issue carries an agent-brief comment, which is the contract `/implement` builds
from. Ship changes as a branch + PR — don't commit directly to `main`.

## Layout

Two independent Rust crates, no root workspace — run cargo per crate:

- `crates/partial-json-fixer`: the core library
- `py/partial-json-fixer`: PyO3 binding wrapping the core crate by path;
  fixes to the core crate reach Python automatically, there is no second copy of the logic

## CI

`.github/workflows/CI.yml` (maturin-generated): the `test` job runs
`cargo test` + `cargo clippy -D warnings` on both crates on every push and PR;
the wheel-build matrix and the PyPI `release` job run only on pushes to
`main` and tags.
