# Implementation status

Updated: 2026-10-31. Historical baseline: master at `1792c2f`.
Assessment: [review-report.html](../review-report.html#F04).

## Verification coordination

- Owner: Codex, this chat; sole editor of `src/app/world/mod.rs`.
- Active job: none; F09 final verification complete. Shared lock released; no
  owned simulator/server processes running.
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
| Next native backlog | F13–F16 performance/interaction | Open: rendering work, clone allocation, paused repainting, click painting. |

The user's 2026-10-30 follow-up authorized committing/pushing on a fresh branch
and continuing work; on 2026-10-31 the user requested edge-case regressions.
The simulation correctness sequence F03–F08 is complete;
unimplemented findings retain their historical status. Resolution details belong
in completed plans. F04's four historical evidence files are now tracked unchanged.

Latest verification: 37 tests pass in debug/all-features and release; native check,
desktop binary build, doc tests (0), formatting and shell syntax pass. The 10 new
timing tests cover every supported rate's nanosecond boundary, frame partitioning,
resets, capped stalls and extreme durations. Six historical strict Clippy
diagnostics remain, with no new ones. No GUI/performance claim. Counts in earlier
rows reflect their commit's suite; historical evidence remains unchanged.

2026-10-31: user requested browser/WASM removal followed by the next native task.
Cleanup is committed/pushed as `778f2c8`; F09 is committed/pushed as `cc3c42f`. Browser
findings are retired by removing delivery/server support, not by repairing the
old browser pipeline. The cleanup pruned five unused packages with no remaining
version/checksum changes; F09 uses that unchanged graph. Cargo.lock/target stay
ignored outputs. Completed plans preserve exact commands, graph records and limits.
