# F04: preserve incoming water during cleanup

- Status: complete, implemented and verified in the working tree; uncommitted.
- Owner: Codex, this chat (including all verification).
- Context: [historical F04](../../review-report.html#F04) and
  [saved probe source](../../reviews/evidence/2026-10-30-rule-probes.rs.txt).
- Baseline verified: master / `1792c2f7dc735be4663fc2f2683cf43fdc473839`,
  subject `Update README.md`, one worktree, no tracked/staged changes; all 26
  tracked files match HEAD. Four untracked historical evidence files preserved.
- Scope: water cleanup and focused child-module tests, plus requested planning
  support. No other finding, flow constants, phase ordering or dependency changes.

## Progress and implementation steps

- [x] Establish reproduction with the new regression against unchanged rules.
- [x] Trace callers and choose bounded fix.
- [x] Implement fix and complete debug/release targeted tests.
  Both profiles: 7 passed, 0 failed.
- [x] Complete final verification, preservation checks and handoff.

## Decisions, discoveries and assumptions

`App::update` calls `World::update`; it clones `data` into `tmp` before the water
phase. `update_water` reads old sources from `data` and accumulates transfers in
`tmp`. Cleanup must subtract only the old source residue, rather than clear the
accumulator. A Water source with old mass below `MIN_MASS` does not flow this phase;
its original residue is deliberately discarded, even if incoming water rescues it.
Incoming mass stays in `tmp`. At cleanup, retained mass at least `MIN_MASS` is
Water; smaller retained incoming mass is passive under None (as already possible
for small incoming transfers elsewhere). Zero incoming mass leaves None/zero.

Existing active sources can drain to zero and keep Water type until the next
phase; this fix does not normalize all cells or change destination flow thresholds.
Tests budget discarded old residues explicitly and use only rounding-scale
tolerances where flow arithmetic needs them. Fixtures are deterministic and call
only the private water phase, rebuilding `tmp` each iteration. One focused
accumulator test explicitly credits `tmp` after cloning to cover retained incoming
quantities immediately below, at and above the active threshold.

## Validation

User reported the competing benchmark finished; process inventory confirmed an
idle verification slot. This chat held the shared lock throughout verification,
then released it at completion. All commands ran sequentially. Installed Rust
1.95.0 is used, with build jobs 2 and test threads 1. No Cargo.lock or target
existed initially. Current validation generated an ignored lock from a fresh
resolution (330 packages including vaja_2). Its SHA-256 is
`9b977bdaf27db3dad861f8183364d7b043b8fd97c8a8e559bbfa3fd31eea857f`.
Compared with the report's embedded historical graph, the only package/version
change is log 0.4.34 → 0.4.35. Direct/relevant resolutions remain egui/eframe 0.17.0,
ndarray 0.15.6, rand 0.8.8, direct getrandom 0.2.17, instant 0.1.13, serde 1.0.229,
wasm-bindgen 0.2.130 and web-sys 0.3.107. This is a current validation graph,
not a proposed lockfile or dependency upgrade policy.

Before the production fix:
`CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib water_cleanup -- --test-threads=1`
failed (exit 101): 1 passed, 6 failed. Exact reproduction retained 0 instead of
1.0; zero-source incoming also became 0. Supported basin retained 4.0009765625
instead of the explicit budget 5.00390625 - 0.00390625 = 5.0. Isolated residue
cleanup passed. Only the child-module hook/tests had changed at that point.

After the fix:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib water_cleanup -- --test-threads=1` | PASS: 7 tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib water_cleanup -- --test-threads=1` | PASS: 7 tests. |
| `cargo +1.95.0 fmt --all` | PASS: formatted the new tests; no unrelated source changed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | PASS: 7 library tests, 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS: 0 doctests. |
| `cargo +1.95.0 fmt --all -- --check` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all` | FAIL (exit 101): same six historical diagnostics; no new diagnostics. |
| `git diff --check` | PASS. |

Clippy comparison against the saved report log: redundant field names (now line
78), unnecessary cast (147), needless returns (187, 709), identical branches
(239), collapsible condition (562), all in `src/app/world/mod.rs`. No unrelated
lint repairs included.

WASM pipeline and web scripts: not rerun; the historical dependency/type failures
remain documented in [the assessment](../../review-report.html). No browser/GUI,
cross-platform, benchmark or performance validation claimed. These focused water
tests do not establish conservation for wood displacement or other known findings.

## Recovery

Owned production changes: only the F04 cleanup hunk and child test-module hook in
`src/app/world/mod.rs`; new tests in `src/app/world/tests.rs`. Review the diff and
undo those hunks alone. Planning files are new and may be removed only if still
wholly owned. Preserve the four original review/evidence files and all unrelated
work. Release only this chat's verification lock; never use git clean.

## Outcomes and supporting evidence

F04 is resolved in the working tree. Affected code: the small-source cleanup in
`src/app/world/mod.rs`, with private test access through `tests.rs`. The exact
1.0-over-0.005 reproduction now retains exactly 1.0 rather than 0; only the old
0.005 residue is discarded. Zero-source incoming is preserved. The supported
incoming/outgoing fixture retains exactly 5.0 after its explicit 1/256 residue
sink; 16 phases stay within f32 rounding bounds of that budget. Tests check
finite/nonnegative scalars, destination types, solid borders and unchanged
ineligible cells (including Sand and Smoke).

SHA-256 comparison confirms all four pre-existing historical files unchanged;
the other 25 original tracked files still match HEAD. All review/planning/test
files are not ignored. For a future commit, explicitly inventory/stage these new
files and decide whether to include the four pre-existing untracked historical
artifacts; they do not automatically follow a worktree. Generated Cargo.lock and
target remain ignored local validation outputs, not proposed commit content.

New planning support: `AGENTS.md`, `PLANS.md`, `plans/TEMPLATE.md`,
`plans/STATUS.md`, `plans/planned/README.md`, and this completed plan. No commit,
push, merge or deployment performed. [Status](../STATUS.md) records remaining
priorities: F10 before public serving; F03 is the next bounded simulation fix;
F05/F06 and F07/F08 follow. Web delivery F01/F02/F11/F12 requires its own graph,
toolchain and artifact decision. No further finding was implemented.
