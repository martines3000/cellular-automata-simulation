# Implementation status

Updated: 2026-10-31. Historical baseline: master at `1792c2f`.
Assessment: [review-report.html](../review-report.html#F04).

## Verification coordination

- Owner: Codex, this chat; sole editor of `src/app/world/mod.rs`.
- Active job: mixed-material/scaling correctness tests, then local performance
  measurements; Codex owns the shared lock. One worktree, no competing jobs.
- Shared lock for cooperating workers:
  `/tmp/cellular-automata-simulation-verification.lock` (directory; owner recorded
  inside). Inspect process inventory before acquiring; never remove another owner's
  lock or stop their processes. Run one verification command at a time.
- Settings used: installed Rust 1.95.0, build jobs 2, test threads 1.

## Finding resolutions and backlog

| Priority | Finding | Status / next bounded task |
| --- | --- | --- |
| Committed/pushed | F04 water cleanup | `1d407f1` on `codex/simulation-correctness-fixes`. [Completed plan and exact commands](completed/f04-water-cleanup.md). Debug/release: 7/7 each; native checks/tests/format pass; six unchanged baseline Clippy diagnostics. |
| Retired by removal | F10 path traversal | Node server removed with browser delivery; historical evidence unchanged. [Cleanup plan](completed/native-only.md). |
| Committed/pushed | F03 fire indexing | `b99b9de`; 3 fire tests (960 supported configurations), 10 total tests pass. [Completed plan](completed/f03-fire-indexing.md). |
| Committed/pushed | F05/F06 editing state | `6236c6e`; 4 edit tests and 14 total tests pass. [Completed plan](completed/f05-f06-edit-state.md). |
| Committed/pushed | F07 displacement | `7195209`; 19 total tests pass, including 6,561-case mass matrix. [Completed plan](completed/f07-wood-displacement.md). |
| Committed/pushed | F08 direction / F07 follow-up | `01b4340`; 27 total tests pass in debug/release/all-features. [Completed plan](completed/f08-lateral-flow.md). |
| Retired by removal | F01/F02/F11/F12 | Browser entry point, build/hosting scripts and assets removed at user request; no web compatibility claim. [Cleanup plan](completed/native-only.md). |
| Committed/pushed | F09 timing | `cc3c42f`; fixed-step accumulator, 16-step catch-up cap, pause/rate resets and Steps/s label. 37 tests pass. [Completed plan](completed/f09-fixed-step.md). |
| Committed/pushed | F16 input / F15 repaint | `18ff7e5`; primary clicks/dragging, coalesced click handling and idle paused frames; real egui input regressions. [Native follow-up plan](completed/native-polish.md). |
| Committed/pushed | F14 scratch reuse | `64651d4`; original two allocations retained; 100-tick equality against clone reference. |
| Committed/pushed | F13 rendering | `3913e5f`; visible-cell culling, empty-cell omission and shape capacity reuse, geometry regressions. |
| Committed | Display visibility/timing | `d68114d`; collapsed Display keeps active simulation advancing; preserves pause and paint/update order. [Completed plan](completed/display-visibility.md). |

The user's 2026-10-30 follow-up authorized committing/pushing on a fresh branch
and continuing work; on 2026-10-31 the user requested edge-case regressions.
The simulation correctness sequence F03–F08 is complete. Resolution details belong
in completed plans. F04's four historical evidence files are now tracked unchanged.

Latest local verification (2026-10-31): `bash check.sh` passes using pinned Rust
1.95.0 and the committed native Cargo.lock: check/build, fmt, strict Clippy (zero
diagnostics), 53 all-target/all-feature debug tests, 0 doc tests and 53 release
all-feature tests. Mechanical lint cleanup: `3a32a09`. Native UI results/limits:
`611c83b`; macOS startup, painting, zoom, rate, play/pause, Clear/Generate/Smooth
observed. Automated native dragging was unreliable; headless RawInput tests cover
it. No controlled performance or Windows execution claim.

Reproducible native builds: `af94baf`. Existing 325-package graph committed
unchanged, Rust 1.95.0 pinned, all Cargo checks locked. The user subsequently
requested local-only testing: CI run confirmed cancelled, workflow confirmed disabled
and removed, with README/AGENTS updated. No future Actions runs are configured.
Ubuntu's brief CI run passed; it reported a quick-xml 0.22.0 future-compatibility
warning. Dependency migration remains separate. Browser delivery was removed
(`778f2c8`); no browser validation claimed or required.

All sixteen historical findings are implemented or retired by browser removal.
Historical reports and extracted evidence remain byte-for-byte unchanged. Counts
in earlier rows describe those commits; current suite contains 53 tests.

[Completed native follow-up](completed/native-polish.md) records decisions,
commands, evidence and limits. Future work is separate: dependency compatibility,
controlled frame profiling and any engine/rendering redesign. Windows support
is outside the backlog at the user's request.

2026-10-31 follow-up: [Display visibility/timing](completed/display-visibility.md)
reproduced before the fix and passed after it. Three new regressions cover
collapse/reopen, unchanged paused frames and painting before ticking. Local-only
`bash check.sh` passes all 53 tests in debug/release plus strict Clippy.

Active plan: [Mixed materials and local scaling](planned/mixed-material-performance.md).
Windows removed from future work; all validation remains local-only.
