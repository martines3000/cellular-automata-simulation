# Mixed-material correctness and local performance scaling

- Owner: Codex, this chat; one source editor and verification owner. Research agent
  owns only a new tools research note; no builds/checks/benchmarks delegated.
- Start: `c737ec8`, clean branch `codex/simulation-correctness-fixes`, one worktree.
- User scope: mixed-material edge cases, useful benchmark tools, larger worlds,
  measured improvements and GPU tradeoffs. Local-only, no Windows backlog or CI.

## Progress

- [x] Add reproducible mixed-material fixtures and conservation/state invariants.
- [x] Reproduce and fix confirmed defects with focused regression cases.
- [x] Research primary-source benchmark/profiler tools and GPU tradeoffs.
- [x] Add a repeatable local benchmark for ticking, copying and drawing larger grids.
- [x] Verify exact memory/geometry budgets and optimize them; defer wall-time
  sampling at the user's explicit request to measure later.
- [x] Add paused-only 100/256/512/1024 world controls with explicit resize/clear.
- [ ] Complete local checks, summarize compact evidence, commit/push and close records.

## Decisions and assumptions

Preserve material phase order, fluid constants, deliberate cleanup budgets and
smoke lifetime meaning. Account for tiny water cleanup before the water phase;
fire/smoke scalar changes are distinct from water. Fixtures must reproduce failing
cases without sleeps; test isolated material interactions and repeated full ticks.
Keep benchmarks separate from ordinary tests and compare equal starting states.
Sizes include the existing 100×100 plus 256/512/1024 where machine resources allow.
CPU update/shape/tessellation times are not GPU frame times. Treat GPU simulation
as a separate engine design if evidence warrants it; don't silently change rules.

## Verification and recovery

Shared lock: /tmp/cellular-automata-simulation-verification.lock. Parent Codex owns
all serialized local verification: Rust 1.95.0, build jobs 2, test threads 1.
Inspect worktrees/processes before each session; no overlapping app/benchmark/check
jobs. Record hardware, workload, sample/reset policy and noise/limitations.
Preserve original historical report/progress/research/probes byte-for-byte. New
notes use dated filenames. No targets, binaries or large traces tracked. Preserve
unrelated work and revert only owned commits/hunks; release only owned lock.

## Outcomes

Pending; no performance claim or dependency migration yet.

Correctness milestone: seeded StdRng (entropy by default) now supplies generation
and sand choices; tests reset it to fixed seeds. No UI seed control or cross-version
random-sequence guarantee. Five new tests reproduced water loss in diagonal sand,
falling fire and rising smoke moving into None cells holding passive water. Exact
0.005/0.01-unit fixtures failed before the fix; large smoke phase lost 0.08 units
at size 100, seed 42, tick 0. Movement now relocates passive water to the vacated
source and separates smoke lifetime from water. Existing phase rules/caps retained.
`CARGO_BUILD_JOBS=2 cargo test --locked --lib mixed_material -- --test-threads=1`:
5 pass after fix. Small matrix: 384 initial fixtures, 24 ticks each, every phase's
quantity/solid/count invariants plus equality to the production tick schedule.
Large fixtures: 100/256/512/1024, 2 ticks each. Cleanup quantities explicitly
budgeted; bounded f32 rounding is separate. All historical evidence preserved.

Tools note: [primary-source research](../../reviews/research/2026-10-31-performance-tools.md).
Choose exact Divan 0.1.21 dev-only, private benchmark functions + ignored local
runner (no CLI argument parsing, no production engine methods exposed). Main
application dependencies remain fixed. Benchmarks must be excluded from check.sh.

Harness milestone: Divan 0.1.21 default features disabled; 8 new registry packages,
all pre-existing registry entries unchanged; root adds a dev dependency
(325→333 total lock packages).
Lock SHA256 `69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`.
`bash check.sh`: check/build/fmt/strict Clippy pass; 58 debug/release tests pass,
2 explicit performance tests ignored; 0 doc tests. Non-timing harness smoke command
in PERFORMANCE.md passes all 32 cases. No simulation methods exposed for benches.
Host: Apple M3 Pro, 11 cores, 18 GiB, macOS 26.7.1. Sustained background browser
CPU activity prevents controlled wall-time claims; user was asked for a quiet slot.
Continue exact memory/geometry checks while timing remains deferred.

Scaling milestone: baseline storage-budget test failed at 50,331,648 bytes for two
1024² grids. Cell x/y coordinates were unused outside initialization and test
helpers; use ndarray indices as position truth. Removing them reduces Cell 24→8
bytes and two grids 48→16 MiB. Existing solid-protection tests now save index/cell
pairs, preserving their assertions instead of relying on redundant coordinates.
Dense-basin geometry before/after: 100: 5198→247; 256: 33278→637;
512: 132094→1277; 1024: 526334→2557. Merge only immediately adjacent generated
rectangles with equal color and vertical bounds; preserve caller shapes, gaps and
fractional heights. Twelve pre-merge quarter-pixel coverage snapshots match,
including pan and all materials. GPU antialiasing is not modeled by these checks.
Three storage tests and nine rendering tests pass. No FPS/speedup claim.

User explicitly chose "Continue setup; measure later". Do not run timed sampling
in this session. Added optional paused world-size presets with explicit Resize and
clear, preserving settings, resetting both buffers, pan and timing; shift sliders
scale with the grid. Resize state and real egui button guard tests pass.

Final setup checks: `bash check.sh` passes native check/build, formatting, strict
Clippy, 65 debug/all-feature tests, 0 doc tests and 65 release/all-feature tests;
2 performance helpers stay ignored. The separate release non-timing harness smoke
passes all 32 cases and prints Cell storage 8 bytes. No timed sampling performed.
Native control check on the owned debug app confirmed all four size presets,
explicit resize to 1024 and both pan sliders reaching 1024. Initial merged-border
rendering appeared correct. Subsequent input automation returned noWindowsAvailable
while the app process remained alive; additional native generation/visual checks
were not completed. Owned process/wrapper stopped/removed. No GPU timing or
pixel-perfect GPU antialiasing equivalence claim.

Exact counts/results: [dated scaling evidence](../../reviews/evidence/2026-10-31-scaling.md).
Timed sampling is deferred into [a separate plan](local-performance-measurements.md)
at the user's request. The current setup/correctness/scaling scope is complete;
remaining work is final reviewable commits, push and coordination closure.
