# F08: consistent lateral water/wood rules

- Owner: Codex, this chat; sole world editor and verification owner.
- Status: complete and verified in working tree, 2026-10-31; committing separately.
- Evidence: [F08](../../review-report.html#F08), including the saved 10-unit
  symmetric basin result (left 1.875, right 2.5) and left-side wood exclusion.
- Scope: compute both lateral flows from the same available source quantity,
  share eligibility/wood handling, and require positive flow before displacement.
  Preserve /3 and smoothing coefficients, pressure rules and phase ordering.

## Progress

- [x] Trace the two branches and their wood helpers; F07 already preserves
  vertically displaced water. Horizontal wood motion must also preserve passive
  destination mass rather than leave water hidden under Wood.
- [x] Establish failing regressions, including cutoff and blocked-side cases.
  Baseline `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib lateral_flow
  -- --test-threads=1`: all 4 fail, including 0 vs 0.16666667 at MIN_FLOW.
- [x] Apply localized shared side-flow rule and water-preserving wood moves.
- [x] Verify all regressions debug/release and native checks; commit/push authorized.

## Decisions and assumptions

Evaluate both sides using the mass remaining after downflow. Each requested
lateral flow is at most one third of that quantity for finite nonnegative inputs,
so their sum fits the source budget. A blocked side requests zero and must not
skip the other side or upward pressure. Preserve deterministic phase scheduling;
no seeded RNG, new material, constant tuning or broader engine refactor.
Edge discovery: falling wood also wrote its new type over passive water, hiding
the quantity under Wood. A new regression failed before the fix (lost 0.005
fluid units); update_wood now uses the same water-preserving move helper. This is
a documented F07 follow-up needed to make displacement consistent across phases.
The original falling eligibility and phase position are unchanged.
Full-world mirror symmetry is not claimed: row scan order still affects multiple
interacting sources. Test reflection for isolated supported sources/material
arrangements and retain the independent mass matrix for multi-source safety.

## Verification and recovery

Rust 1.95.0, unchanged F04 graph, build jobs 2 / test threads 1, serialized under
the shared lock after process inventory. Revert only this bounded commit and its
tests; preserve other fixes and historical evidence. No web/GUI/performance claim.

## Outcome

F08 now shares one lateral eligibility/displacement rule for both directions.
Both use the post-downflow source mass; all original /3 and smoothing constants
remain. Positive flow is required before moving wood or promoting a destination.
A blocked direction returns zero rather than skipping the other direction or
pressure. The former lift helper is now move_wood with source/destination index
pairs, reused for vertical lifting, horizontal displacement and falling. Every
move returns destination fluid to the vacated cell before zeroing the new Wood.
This fixes the separately reproduced F07 falling/passive-water edge as well.

The 10-unit saved symmetric fixture now sends 2.5 units to each side, replacing
left 1.875 / right 2.5. Tests cover MIN_FLOW's adjacent f32 values, a 100-case
left/right material/source-mass matrix and its mirrors, blocked/free wood, zero
flow with stale Wood scalar, passive destination water, every interior source
column near borders, pressure continuation despite blocked sides, and 32 full
sand → water → fire → smoke → wood ticks conserving 25 water units and one Wood.
Each phase rebuilds tmp exactly as production does; no sleep/RNG seed is used.
All four initial lateral tests failed before the fix; the falling-water test
independently failed before its additional fix and then passed.

Final sequential verification under this chat's shared lock, unchanged F04 graph:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib lateral_flow -- --test-threads=1` | Initial 4 tests passed after fix, failed before; 2 further edge tests pass in full suite. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib wood_displacement_falling -- --test-threads=1` | PASS after fix, failed before (0 vs 0.005). |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib -- --test-threads=1` | 27 passed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib -- --test-threads=1` | 27 passed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 27 passed, 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all` then `cargo +1.95.0 fmt --all -- --check` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all` | Six historical diagnostics, no new ones. |

Historical evidence unchanged; no toolchain/dependency/web/server changes. No
browser/GUI/cross-platform/performance claim. Reflection is tested for isolated
supported sources, not promised for arbitrary multi-source scan order. The mass
matrix covers 6,561 representative finite states, not all possible f32/world
states. Invalid externally modified borders and nonfinite input are outside this
work's valid-world contract. Commit identity follows in status/history.
