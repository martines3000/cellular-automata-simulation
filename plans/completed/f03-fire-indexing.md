# F03: signed fire-neighbor indexing

- Status: complete, committed/pushed as `b99b9de`.
- Owner: Codex, this chat; sole editor of the world module and verification owner.
- Authorization: user requested a fresh branch, commit/push, and continued work
  on 2026-10-30. F04 is committed/pushed as `1d407f1`; F03 is the next bounded
  simulation item in status.
- Context: [historical F03](../../review-report.html#F03) and
  [saved probe](../../reviews/evidence/2026-10-30-rule-probes.rs.txt).
- Scope: signed neighbor arithmetic in supported fire, deterministic regressions.
  Preserve movement, ignition eligibility, smoke values, borders and phase order.
  No other material rule or dependency changes.

## Progress

- [x] Trace callers: `World::update` clones data into tmp before `update_fire`;
  there are no other production callers. Reuse the historical confirmed diagnosis.
- [x] Reproduce in a focused debug regression before editing the rule.
  `cargo +1.95.0 test --lib fire_indexing -- --test-threads=1` with build jobs 2:
  one pass, one failure at mod.rs:592:42, `attempt to add with overflow`.
- [x] Fix signed arithmetic and verify debug/release fire regressions.
- [x] Complete native checks and record outcomes; commit/push authorized.

## Decisions and assumptions

Valid worlds have solid Dirt borders and fire in the interior. Negative unit
offsets must be added while signed, then converted to indices, matching the
existing neighborhood arithmetic used by Smooth. Fixing arbitrary externally
injected fire on the border is outside this finding's reproduction and scope.
The saved probe already isolates the cause; a new broad diagnosis/audit is not
needed. Tests assert smoke type/lifetime, four-way wood ignition, unaffected
diagonals/materials/borders, and the unchanged unsupported-fire movement branch.

## Verification

Use installed Rust 1.95.0 and the existing ignored validation lock, build jobs 2,
test threads 1. Serialize all checks using the status record and shared lock.
Another repository's Cargo job was active at initial inventory; no checks start
here until it exits. No GUI/server, benchmarks or web toolchain work.

## Recovery

Revert only F03 indexing hunks and its added tests after inspecting the diff;
preserve committed F04, historical evidence and unrelated edits. Release only
this chat's verification lock. Never clean the checkout.

## Outcome

Implemented signed addition before converting to ndarray indices; no phase or
material eligibility changes. Baseline debug overflow was reproduced before the
fix. On 2026-10-31 the expanded tests cover 960 supported-fire configurations:
all interior positions in sizes 3, 4, 7; both fire types; all 16 cardinal
wood/solid patterns. Whole-grid equality verifies unaffected diagonals/borders.
Additional cases cover Water/Sand/Smoke/DarkSmoke neighbors and unsupported fire.

Sequential validation, `CARGO_BUILD_JOBS=2` for Cargo build/test/check commands:

| Exact command | Result |
| --- | --- |
| `cargo +1.95.0 test --lib fire_indexing -- --test-threads=1` | 3 passed after fix; original baseline 1 passed / 1 overflow failure. |
| `cargo +1.95.0 test --release --lib fire_indexing -- --test-threads=1` | 3 passed. |
| `cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 10 passed, 0 binary tests. |
| `cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all` | Applied formatting only. |
| `cargo +1.95.0 fmt --all -- --check` | PASS after formatting. |
| `cargo +1.95.0 clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all` | Same six historical diagnostics before matrix expansion; final comparison recorded with subsequent fix. |

The F04 validation graph is unchanged; no dependency edits. Historical evidence
unchanged. No web/GUI/cross-platform/performance claim. Fire on externally altered
borders remains outside valid-world inputs; UI painting excludes the border.
All edge cases here are finite deterministic matrices, not an exhaustive proof
of every possible world. Commit identity is recorded in status/history after
creation. Recovery: revert this commit alone once committed.
