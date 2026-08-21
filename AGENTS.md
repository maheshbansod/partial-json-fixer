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

**Branch per issue, off `main`.** Start every task with
`git switch -c <branch> main`. Never stack on or reuse an existing feature
branch, and don't commit to one unless it's the branch you created for the
current task — if the working tree is on another branch, switch away first.
Stacked PRs only when the user explicitly asks.

## Layout

Two independent Rust crates, no root workspace — run cargo per crate:

- `crates/partial-json-fixer`: the core library
- `py/partial-json-fixer`: PyO3 binding wrapping the core crate by path;
  fixes to the core crate reach Python automatically, there is no second copy of the logic

## CI

`.github/workflows/CI.yml` (maturin-generated): the `test` job runs
`cargo test` + `cargo clippy -D warnings` on both crates on every PR.
Nothing runs on plain pushes to `main`; the wheel-build matrix, PyPI
publish (`release` job), and crates.io publish (`publish-crates` job)
run only when a `v*` tag is pushed.

**Local test runs:** only run `cargo test` and
`cargo clippy --all-targets -- -D warnings` inside
`crates/partial-json-fixer`. Skip the `py/partial-json-fixer` crate
locally — its test binary links a Python framework that most dev
machines can't resolve (dyld `Library not loaded ... Python3`, SIGABRT),
so it fails for environment reasons before any test executes. CI covers
both crates on every PR, so nothing is lost by skipping it.

## Releases

Manual, one command from `main`: `./scripts/release.sh <patch|minor|major>`.
It bumps both `Cargo.toml`s in lockstep, refreshes lockfiles, commits,
tags `vX.Y.Z`, and pushes — the tag push triggers publishing to PyPI and
crates.io. Requires `PYPI_API_TOKEN` and `CARGO_REGISTRY_TOKEN` secrets.
Tags are always plain `vX.Y.Z` (no suffixes).
