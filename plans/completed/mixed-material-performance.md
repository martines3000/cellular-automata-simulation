# Mixed-material correctness and local scaling setup

- Complete: 2026-10-31. Owner: Codex, this chat; sole verification/source owner.
- Start: `c737ec8`, clean `codex/simulation-correctness-fixes`, one worktree.
- Commits pushed: `563cfb2` (water preservation, seeded fixtures, local harness),
  `449ed04` (compact cells, merged geometry, larger-world controls).
- User scope: mixed-material edge cases, useful performance tools, larger worlds,
  improvements and GPU options. Local-only; Windows removed from backlog.

## Completed steps and decisions

- [x] Add seeded fixtures and per-phase/full-tick invariants.
- [x] Reproduce/fix passive-water loss in diagonal sand, falling fire and rising smoke.
- [x] Research primary-source tools; add exact dev-only Divan 0.1.21.
- [x] Validate all 32 benchmark workload/size cases without timing sampling.
- [x] Reduce exact cell-storage and dense-geometry budgets; preserve coverage.
- [x] Add paused-only 100/256/512/1024 presets, explicit Resize and clear and full pan range.
- [x] Complete local checks and native control check; preserve historical evidence.
- [x] Defer timing runs to [the sampling plan](../planned/local-performance-measurements.md)
  per the user's explicit "Continue setup; measure later" instruction.

Generation/sand now use entropy-initialized StdRng, reset to fixed fixture seeds.
No UI seed control or cross-version random sequence guarantee. Preserve material
phase order, coefficients, pressure rules and smoke lifetime meaning. Only the
water phase deliberately discards old sub-threshold Water sources; f32 transfer
rounding is bounded separately in conservation checks. Moves into None cells
relocate passive water to the vacated source instead of overwriting/contaminating it.

Cell coordinates were unused outside initialization and test helpers. ndarray
indices are now position truth; protection tests save index/cell pairs. Cell
storage is 24→8 bytes, two 1024² grids 48→16 MiB. Renderer merges immediately
neighboring equal-color/equal-height rectangles, preserving append/painter order,
gaps, colors and fractional water/wood surfaces. Twelve pre-merge opaque-coverage
snapshots match; they do not model GPU edge antialiasing. See exact counts below.

Resize resets both grids, Dirt border, pan and timing, preserving brush/zoom/rate/
generation controls. Controls are disabled while running and require an explicit
clear action. Default size remains 100. GPU compute is a later design decision;
the existing Glow backend already rasterizes on the GPU.

## Reproduction and validation

`CARGO_BUILD_JOBS=2 cargo test --locked --lib mixed_material -- --test-threads=1`:
all 5 new cases failed before movement fixes and passed afterward. Focused exact
0.005/0.01-unit cases catch sand/fire/smoke losses; a 100-side smoke phase lost
0.08 units before the fix. 384 seeded fixtures ×24 ticks verify per-phase budgets,
finite/nonnegative scalars, material counts, Dirt/borders and production schedule.
Larger 100/256/512/1024 fixtures pass two ticks each.

Storage-budget and dense-geometry tests failed before their respective changes,
then passed. Rendering/resize/paint-state tests and original regressions all pass.
`bash check.sh`: native check/build, fmt, strict all-target/all-feature Clippy,
65 debug/all-feature tests and 65 release/all-feature tests pass; 2 performance
helpers stay ignored; 0 doc tests. Rust 1.95.0, jobs 2, test threads 1.

`CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1`:
PASS, all 32 registered cases execute once without timing sampling; Cell prints 8 B.
Eight new dev-only registry packages; all pre-existing registry entries unchanged
(325→333 total lock packages). Lock SHA256:
`69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`.

Native owned debug app: observed four presets, 1024 resize, pan limits 1024 and
initial merged-border rendering. Further input automation returned
noWindowsAvailable while process stayed alive; subsequent native generation/
visual checks were not completed. Owned process/wrapper removed. No timed Divan,
FPS, GPU timing or pixel-perfect GPU antialiasing equivalence claim.

## Evidence, coordination and recovery

- [Exact dated scaling evidence](../../reviews/evidence/2026-10-31-scaling.md)
- [Primary-source tools/GPU research](../../reviews/research/2026-10-31-performance-tools.md)
- [Runnable commands and workload/reset policy](../../PERFORMANCE.md)

Only the parent ran verification; research agent wrote one note without checks.
No overlapping compiler/test/GUI/benchmark jobs. Shared verification lock released
at closure. Four original historical evidence files compare byte-for-byte with
`1d407f1`; all supporting files tracked/not ignored. No unrelated edits present.
No CI, Windows validation, target outputs, binaries or large traces added.
Revert only owned commits/hunks, preserving unrelated work and historical evidence.
