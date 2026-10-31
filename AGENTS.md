# Working in this repository

- Keep fixes local to the selected finding. Trace callers before changing a rule;
  test private phases through a child module. Preserve sand → water → fire → smoke
  → wood ordering and copying current `data` into `tmp` before each phase (reuse its allocation).
- For multi-step work, read [PLANS.md](PLANS.md) and update
  [plans/STATUS.md](plans/STATUS.md). Small, low-risk fixes may use a status checklist.
- Historical evidence is in [review-report.html](review-report.html), its completed
  [review-progress.md](review-progress.md), and [reviews/](reviews/). Preserve these
  files unchanged, including when untracked; copy them explicitly across worktrees.
  Record resolutions separately. Future assessments get new dated filenames.
- One verification owner records the active job in `plans/STATUS.md`. Before each
  verification session, inspect `git worktree list --porcelain` and running Cargo,
  Rust, simulator and benchmark processes. Serialize checks across worktrees
  and coordinate other workloads; use the shared lock recorded in status.
- Run `bash check.sh` for native check/build, formatting, strict Clippy, debug/release
  tests and doc tests. It uses jobs 2 and test threads 1. Keep verification serial;
  record exact results and platform limits in the active plan.
- Native desktop builds use the committed Cargo.lock and rust-toolchain.toml.
  Use `--locked`; dependency/toolchain upgrades are separate tasks. Browser delivery
  was removed; historical review records describe the previous web baseline.
- Preserve unrelated edits; never use `git clean`. Keep evidence compact and omit
  binaries, targets, caches, generated WASM, credentials and private runtime data.
  Before a commit, explicitly inventory untracked review/planning files and verify
  they are not ignored. Commit/push/deploy require user authorization.
