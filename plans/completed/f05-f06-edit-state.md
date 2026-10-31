# F05/F06: editing operations preserve or reset scalar state

- Owner: Codex, this chat; sole world editor and verification owner.
- Status: complete and verified in working tree on 2026-10-31; committing separately.
- Authorization: continued correctness work and edge-case tests requested.
- Historical evidence: [F05](../../review-report.html#F05),
  [F06](../../review-report.html#F06), and saved probe source in `reviews/evidence/`.
- Scope: Smooth starts with current cell state; generation initializes mass for
  every cell. Preserve cave rules, borders, material eligibility and phase order.

## Progress

- [x] Trace callers: Generate/Smooth/Clear buttons call these public world methods
  directly; callers do not initialize tmp. Clear resets data but leaves old tmp.
- [x] Establish regressions against unchanged editing methods. All 3 failed:
  stale live material scalar, generation mass 1 instead of 0, and Clear → Smooth
  restoring old water. Command: `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib
  edit_state -- --test-threads=1` (exit 101).
- [x] Apply bounded fixes and test debug/release.
- [x] Complete native verification and update records; commit/push authorized.

## Decisions and assumptions

Smooth may change Dirt/None types according to its existing neighbor rule; other
materials and their complete cell state must survive. Start its scratch grid from
current data to prevent stale water/smoke mass, including after Clear/Generate.
Generation replaces every material with Dirt/None, so mass must start at zero.
Tests use deterministic generation thresholds 0 and 1 (no seeded-RNG dependency).
Fixtures cover minimum useful worlds, every interior position, water around the
cleanup threshold and above MAX_MASS, smoke lifetimes, all retained materials,
repeated smoothing, and stale-scratch operation sequences. No global mass
normalization or simulation-constant changes are introduced.

## Validation and recovery

Use existing Rust 1.95.0 and F04 validation graph, jobs 2 / test threads 1; inspect
worktrees/processes and own the shared lock before serialized checks. Revert only
this commit's editing hunks/tests after reviewing the diff. Preserve F03/F04 and
historical evidence. No web/GUI/performance claim; no server required.

## Outcomes

F05: Smooth initializes tmp with current data before changing Dirt/None types.
F06: generation sets every cell mass to zero before assigning its new type.
The changes are three production lines plus a comment and test-module hook.
All three original regressions failed before the fix and now pass. The fourth
regression covers every 8-neighbor Dirt/None pattern (256 patterns × 2 center
states = 512 cases), preserving the existing cave rule. Scalar preservation
covers 300 material/location fixtures, each smoothed three times; generation
covers 10 deterministic size/threshold endpoint fixtures, including sizes 1/2.

Sequential verification on the unchanged F04 lock graph, jobs 2 / test threads 1:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib edit_state -- --test-threads=1` | 4 passed after fix; 3 failed before fix. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib edit_state -- --test-threads=1` | 4 passed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 14 library tests passed, 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all` then `cargo +1.95.0 fmt --all -- --check` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all` | Same six historical diagnostics; no new diagnostics, including expanded F03 tests. |

Historical report/evidence files remain unchanged. No GUI, web, benchmark or
cross-platform verification. Generation thresholds between 0/1 remain stochastic;
these tests check deterministic endpoints rather than RNG distribution.
Commit identity is recorded in status/history after creation. Once committed,
recover by reverting this bounded commit alone, preserving the other fixes.
