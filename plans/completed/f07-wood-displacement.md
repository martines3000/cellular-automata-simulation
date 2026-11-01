# F07: preserve water displaced by lifted wood

- Owner: Codex, this chat; sole world editor and verification owner.
- Status: complete, committed/pushed as `7195209` on 2026-10-31.
- Historical evidence: [F07](../../review-report.html#F07) and saved probe source.
- Scope: lift_wood preserves current destination fluid in the vacated wood cell.
  Preserve flow/pressure coefficients, schedule and directional preference. F08
  directional behavior remains separate. Tests explicitly assert conservation,
  wood counts, borders, finite/nonnegative quantities, stacks and blocked cases.

## Progress

- [x] Trace callers: pressure and lateral water displacement call lift_wood after
  wood_stuck finds an empty/water cell above the contiguous stack.
- [x] Reproduce saved 25-unit case and deterministic edge fixtures.
  Baseline `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib wood_displacement
  -- --test-threads=1`: 1 pass (blocked stack), 3 failures. Direct lift erased
  passive 0.005; saved pressure and lateral-lift fixtures violated 25-unit budget.
- [x] Preserve displaced quantity and test debug/release.
- [x] Complete native checks; commit/push authorized independently.

## Contract and decisions

The top destination can contain water, including passive sub-threshold mass under
None. Moving wood there must carry that water into the vacated bottom cell,
preserving any already accumulated quantity there. Wood at the new top has zero
mass. The vacated cell is Water at or above MIN_MASS, otherwise None with its
passive retained quantity. Read tmp rather than old data so transfers earlier in
the phase remain accounted for. A full water phase may still discard old
sub-threshold Water residue under the documented F04 sink; direct lift tests
separate that sink from displacement conservation.

## Verification and recovery

Use Rust 1.95.0, existing F04 lock graph, jobs 2 / test threads 1. Serialize checks
after process inventory and shared-lock ownership. Revert only this commit's
lift/tests hunks; preserve prior fixes/evidence. No web/GUI/performance claim.

## Outcomes

lift_wood now transfers current destination mass into the vacated wood cell,
adding to any existing accumulation before zeroing the new Wood destination.
Direct helper tests cover 28 stack/destination fixtures (heights 1–4, empty and
water destinations, 0, sub-threshold, at-threshold, ordinary and compressed mass).
The saved 25-unit pressure fixture conserves its water and one Wood for 16 phases;
lateral lifting conserves 25 units; blocked stacks remain byte-for-byte unchanged.

Additional F04 edge coverage, requested by the user: all 6,561 four-cell mass
combinations from 9 boundary values, each for 5 phases in a closed 4×4 world
(32,805 phase checks). Every phase separately budgets the exact old small-source
sink, checks finite/nonnegative scalars, types, and solid borders. This is bounded
state-space enumeration, not a proof over arbitrary floating-point inputs.

Sequential verification, existing F04 graph and shared lock; jobs 2 / threads 1:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib wood_displacement -- --test-threads=1` | 4 passed after fix; baseline 1 passed / 3 failed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib water_cleanup_closed_mass_matrix -- --test-threads=1` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib -- --test-threads=1` | 19 passed, including full mass matrix. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 19 passed; 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all` then `cargo +1.95.0 fmt --all -- --check` | PASS. |

No dependency/web changes; historical evidence unchanged. Six baseline Clippy
findings remain (latest comparison in F05/F06; next comparison after F08).
At this commit F08 left-side eligibility, zero-flow movement and lateral bias
remained open. See [F08](f08-lateral-flow.md) for subsequent resolution and the
additional falling-wood/passive-water regression.
Commit identity is recorded in status/history after creation.
