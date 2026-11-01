# Local performance measurements (complete)

- Status: complete; implementation `fa8ce77`. The 2026-11-01 continued-work request supersedes the earlier
  "Continue setup; measure later" deferral. Coordinated by [workbench plan](game-workbench.md).
- Scope: run the existing PERFORMANCE.md commands in a quiet exclusive local slot;
  no CI, Windows validation or dependency upgrades. Reserve the shared verification
  lock and inspect all worktrees/processes first; jobs 2 / test threads 1.
- Original harness milestone: `563cfb2`. Measured baseline: `237ebc5` with the
  current seven-phase engine; implementation revision is recorded in STATUS. Private Divan cases cover copying, complete ticks, shapes, tessellation
  and one headless tick+UI at 100/256/512/1024. Validate fixtures once, then sample.
- [x] Repeat equivalent workload runs on baseline/updated code sequentially,
  recording actual samples, median/spread, toolchain/lock and power/background state.
- [x] Use core copy/tick/shape cases to attribute improvements. The end-to-end UI
  case includes newly added size controls, so it is not an isolated renderer test.
- [x] Add scoped CPU diagnostics; native GPU/presentation profiling stays separate;
  headless CPU output is not FPS/GPU time. Use the CPU model/tests as an oracle
  before any GPU compute/rule redesign.
- Keep outputs compact in a new dated evidence note; preserve historical files,
  unrelated work and generated artifacts. Release only the owned verification lock.

Outcome: two initial and two final repeat runs collected on the same pinned graph;
[dated evidence](../../reviews/evidence/2026-11-01-workbench.md) records medians,
spread, actual counts and desktop background limitations. Final 76-case smoke
passes. CPU texture preparation supports the overview path; no FPS/GPU claim.
Detailed geometry was also batched after native seam inspection. Native review
completed separately with the app stopped before sampling. See the completed
workbench plan for checks, ownership, recovery and subsequent priorities.
