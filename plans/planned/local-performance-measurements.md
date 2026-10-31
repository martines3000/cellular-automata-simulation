# Local performance measurements (deferred)

- Status: deferred at the user's explicit "Continue setup; measure later" request.
- Scope: run the existing PERFORMANCE.md commands in a quiet exclusive local slot;
  no CI, Windows validation or dependency upgrades. Reserve the shared verification
  lock and inspect all worktrees/processes first; jobs 2 / test threads 1.
- Functional benchmark baseline: `563cfb2`; updated source revision is recorded in
  STATUS. Private Divan cases cover copying, complete ticks, shapes, tessellation
  and one headless tick+UI at 100/256/512/1024. Validate fixtures once, then sample.
- [ ] Repeat equivalent workload runs on baseline/updated code sequentially,
  recording actual samples, median/spread, toolchain/lock and power/background state.
- [ ] Use core copy/tick/shape cases to attribute improvements. The end-to-end UI
  case includes newly added size controls, so it is not an isolated renderer test.
- [ ] Profile native frames separately if measurements justify instrumentation;
  headless CPU output is not FPS/GPU time. Use the CPU model/tests as an oracle
  before any GPU compute/rule redesign.
- Keep outputs compact in a new dated evidence note; preserve historical files,
  unrelated work and generated artifacts. Release only the owned verification lock.
