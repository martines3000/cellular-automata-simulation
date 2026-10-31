# Keep simulation running when Display is collapsed

- Owner: Codex, this chat; start `084881b`, clean checkout, one worktree.
- Scope: application update orchestration and deterministic egui regressions.
  Preserve paint → tick → render order, phase order, pause semantics and timing cap.
  No dependency upgrades, CI, renderer redesign or new simulation rules.

## Progress

- [x] Reproduce with a collapsed Display window through real egui input.
- [x] Ensure one update opportunity per running frame whether body is shown or skipped.
- [x] Verify collapsed/expanded transitions, pause and painting order.
- [ ] Run local check.sh, preserve evidence, commit/push and close records.

## Decisions and validation

World::update has one application caller, inside Window::show's body. egui 0.17
skips that body after collapse animation. Use the returned optional inner response
to update only when the body was skipped; preserve existing visible-frame order.
Tests reuse the egui harness from a World child module for private clock access.
An overdue clock exercises the existing 16-tick cap deterministically without sleeps.

Verification owner holds /tmp/cellular-automata-simulation-verification.lock.
No competing verification processes; Rust 1.95.0, jobs 2, test threads 1. All checks
are local-only. Dependency warning remains a separate migration task.

## Recovery and outcomes

Revert only this task's owned hunks/commit; preserve historical evidence and other
work. Release only the owned lock. Implementation and verification pending.

Reproduction: `CARGO_BUILD_JOBS=2 cargo test --locked --lib display_visibility -- --test-threads=1`
failed before the fix (running transition test, simulation frame 3); the pause and
paint-order cases passed. After the fallback update: 3/3 pass. The test clicks the
real collapse arrow, checks that the canvas actually disappears/reappears, compares
full data/tmp against 16 reference ticks on every transition frame and checks that
paused frames remain idle. Painting still occurs before ticking in visible frames.
Production change: retain Window::show's response and advance outside its closure
only if the closure did not execute. Implemented in the working tree.

Final local `bash check.sh`: PASS. Rust 1.95.0; locked graph unchanged; native
check/build, formatting, strict all-target/all-feature Clippy (zero diagnostics),
53 debug/all-feature tests, 0 doc tests and 53 release/all-feature tests. Three new
regressions ran without sleeps or stochastic fixtures. No GUI or Windows rerun;
these tests drive the real App update using egui RawInput. No performance claim.
