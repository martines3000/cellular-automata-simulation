# F09: simulation timing independent of repaint cadence

- Owner: Codex, this chat; sole world editor and verification owner.
- Status: complete and verified in working tree (2026-10-31); committing separately.
- Authorization: user requested native-only cleanup followed by the next task;
  the cleanup is committed as `778f2c8`. F09 is next in status.
- Evidence: [historical F09](../../review-report.html#F09).
- Scope: fixed simulation steps, bounded catch-up, fractional time retention,
  explicit pause/rate-change behavior, and a Steps/s control label. Preserve the
  five material phases, coefficients and scratch setup. No renderer/performance,
  new materials, seeded RNG, browser delivery or measured-FPS feature.

## Progress

- [x] Trace callers: App calls World::update only while running; Toggle currently
  changes only the App flag, and the FPS slider calls update_speed.
- [x] Establish regression against the original one-tick update.
  `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib timing_ -- --test-threads=1`
  failed before the fix (exit 101): 100 ms at 60 steps/s decayed smoke only
  7.0 → 6.99, exactly one tick. The same entry-point regression passes now.
- [x] Implement fixed-step scheduling and deterministic elapsed-time regressions.
  All 10 targeted timing tests pass.
- [x] Complete native validation; independent commit/push authorized.

## Contract and decisions

Accumulate elapsed nanoseconds × requested steps/s in integer credits; one billion
credits makes a tick. This retains fractional intervals (e.g. 60 or 360 steps/s)
without truncating to milliseconds or adding floating-point drift. The remainder
is always less than one tick. Run at most 16 ticks per UI update, enough for
360 steps/s at 30 repaints/s; discard older whole-tick debt after a longer stall,
retaining fractional credit. This is a bounded responsiveness policy, not a
performance guarantee. Keep the existing rate range 1–360 and clamp invalid
programmatic values to that range when processing elapsed time.

Pause/resume and rate changes reset time/credit, so paused/background time is not
replayed after resume and a new rate starts a fresh interval. Call reset on both
Toggle transitions. A long stall while running is subject to the catch-up cap.
Tests use supplied Duration values with no sleeps or stochastic fixtures; one
entry-point regression backdates Instant by 100 ms and asserts a lower bound on
ticks, which remains valid under scheduling delays.

## Verification and recovery

Use installed Rust 1.95.0, the post-cleanup lock graph, jobs 2 / test threads 1,
process/worktree inventory and the shared lock. Check rate boundaries, nanosecond
remainders, cadence partitioning, zero/huge elapsed time, cap/debt dropping,
resets, and real tick state. Revert only this bounded commit after reviewing
conflicts with later work; preserve native-only cleanup and historical evidence.

## Outcomes

F09 uses an integer fixed-step accumulator and extracts the unchanged material
schedule into private tick(). Elapsed time can produce multiple ticks per update;
no milliseconds or floating-point interval conversion. App resets timing on both
Toggle transitions and rate changes, and labels the requested rate Steps/s.
The scheduler clamps programmatic rates to 1–360. Excess whole-tick debt after
16 due steps is deliberately dropped; fractional credit survives. At most
Duration::MAX nanoseconds × 360 plus a sub-tick remainder fits u128, covered by
an extreme-duration regression in both profiles.

Baseline real update advanced only one tick after 100 ms (smoke 7.0 → 6.99),
then passed after the fix. Ten timing regressions now cover all 360 rate
boundaries at nanosecond precision; 5 requested rates across 30/60/120/144 frame
partitions; actual grid equality against the same tick count; fractional
accumulation; zero/maximum Duration; capped stalls with retained fractions and
no whole debt; invalid i32 rates; pause/rate resets; and complete material phase
order against an independently invoked five-phase schedule. No sleeps or seeded
RNG dependencies. The real-clock lower-bound regression remains valid if the
process is delayed because catch-up is capped above its required minimum.

Sequential verification under this chat's shared lock, Rust 1.95.0 / jobs 2:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib timing_ -- --test-threads=1` | 10 passed after fix; original 1-test reproduction failed before fix. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 build --bin vaja_2` | PASS, native executable links. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 37 library tests passed, 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib -- --test-threads=1` | 37 passed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all` then `cargo +1.95.0 fmt --all -- --check` | PASS. |
| `bash -n check.sh` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all` | Same six baseline diagnostics, no new diagnostics. |

The post-cleanup ignored lock graph is unchanged; see
[native-only cleanup](native-only.md) for its SHA-256/removals. All four historical
review/evidence files remain unchanged. No GUI interaction, measured frame
performance, cross-platform execution or arbitrary-world proof is claimed.
The scheduler stays on the UI thread; rates exceeding 16 due ticks per update
are intentionally limited by catch-up policy. Actual material stochasticity is
unchanged. Verification lock released at completion. Commit identity follows in
status/history. Recover by reverting this bounded timing commit alone after
reviewing conflicts with later changes, preserving native-only cleanup.
