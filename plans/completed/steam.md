# Conserved Steam transport and condensation

- Status: complete, committed/pushed as 50444a8, 2026-11-01; owner: Codex this chat.
- Start: clean aa71b9b, one worktree, no active simulator/check processes.
- Context: next bounded task from [frozen water](../completed/frozen-water.md) and
  [material research](../../reviews/research/2026-11-01-materials.md).
- Scope: Steam, Vaporize/Condense tools, scripted fire/cold contact, deterministic
  quantity/scheduling/input regressions. No heat grid, pressure solver, renderer
  migration, dependency changes, Lava/Acid or timed benchmarks.

## Contract and decisions

- Cell remains 8 B. Steam.mass is water-equivalent quantity, included in the typed
  water accessor; smoke lifetime remains excluded. Steam does not expire or lose
  sub-threshold quantity. Zero-quantity Steam can become None without loss.
- Painting Steam replaces a cell with 1 water unit (explicit source/sink). Vaporize
  changes positive Water/passive None into Steam; Condense changes Steam to Water
  at >= MIN_MASS, otherwise passive None. Both preserve quantity bit for bit.
- Fire vaporizes adjacent liquid Water once per fire phase. Frozen targets still
  melt to Water, with the phase snapshot preventing a second fire from immediately
  vaporizing the newly melted product in the same phase. No implicit heat budget.
- Steam condenses in place beside positive Ice/Snow; the cold solid is unchanged.
  This is a scripted trigger, not heat exchange. Dirt ceilings do not consume gas.
- Append Steam after the six current phases, copying data to scratch first.
  Steam moves at most one cell per phase: up into empty/Water, diagonal through
  open air, then sideways if blocked. Water swaps and passive residue survive.
  Only snapshot liquid/empty destinations are eligible; scratch checks resolve
  collisions without overwrite or processing an arriving parcel twice.
- No liquid/gas merging/expansion, venting, passive cooling or general powder/gas
  displacement. Existing material eligibility and phase order otherwise remain.

## Steps

- [x] Trace Cell scalar, paint/phase tools, fire, schedules, renderer and tests.
- [x] Implement Steam, transitions, movement and palette/tool integration.
- [x] Complete conservation/blocked/contact/threshold/mixed/input regressions.
- [x] Complete serialized native checks and UI inspection; update docs/limitations.
- [x] Record actual revision, commit/push and hand off (`50444a8`).

## Verification / recovery

Codex is sole verification owner. Inspect worktrees/processes; use the existing
shared directory lock, jobs 2 / test threads 1, installed Rust 1.95.0 and unchanged
lock. Run bash check.sh, non-timing workload smoke and release app build sequentially.
Preserve historical reports/extracts. Revert only this task's reviewed owned hunks;
remove only owned temporary artifacts, stop only its app and release its lock.

## Outcomes

Implemented in `50444a8`. Targeted milestone: 101 debug correctness tests
pass, two performance helpers ignored. Thirteen new regressions (11 world, two UI)
cover quantity bit preservation, tiny/high-pressure values, displaced liquid and
passive residue, blocked/cold boundaries, multi-source targets, no same-phase
melt/vaporize chain, 96 mixed fixtures × 24 ticks with per-phase budgets, 100/1024
stress, editing and coalesced/panned UI input. The trapped-gas case runs 512 full
production ticks to include smoke lifetime processing. Final results follow below.
Timed sampling remains deferred by the user.

## Final verification (2026-11-01)

Codex was the sole verification owner; jobs 2 / test threads 1, pinned Rust 1.95.0,
unchanged Cargo.lock SHA-256
`69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`.
The shared lock covered all checks and native inspection; released after stopping
only the owned release app and removing its temporary bundle.

```sh
bash check.sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1
CARGO_BUILD_JOBS=2 cargo build --locked --release --bin vaja_2
```

PASS: native check/build, formatting, strict all-target/all-feature Clippy,
101 debug and 101 release correctness tests, 0 doc tests. Two performance helpers
ignored normally; all 32 untimed workload cases pass separately. Release app builds.
No baseline check failures. New tests cover full schedules and borders at up to
1024², exact phase quantity transfer/brush conversion, explicit cleanup budgets,
96 seeded mixed fixtures × 24 ticks, compact scrolling and coalesced pointer input.
Original coverage snapshots pass; Cell remains 8 B and two 1024² grids 16 MiB.

Native release inspection verified Steam palette/description, four phase tools,
Steam placement and Condense rendering blue Water after moving the cursor away.
A later Vaporize UI action returned noWindowsAvailable while its process remained
alive; no native Vaporize success is claimed. Both directions are covered by real
egui input regressions and bit-preserving world tests. Owned PID stopped afterward.
No frame/GPU/performance timing, CI, browser or Windows checks; no dependencies or
toolchain changes. Historical assessment/evidence files remain unchanged.

## Delivery and next task

Source in `50444a8`: Steam type/color/quantity accessor, new steam.rs, phase brushes,
fire snapshot rule and seven-phase schedule. App palette/tools and independent
phase oracles updated. README explains sources/sinks, no lifetime and scripted
limits. PERFORMANCE records the added work and limits of historical comparisons.

Next bounded material work is Lava with a distinct Stone cooling product and an
explicit conversion budget. It requires its own transport/contact design; do not
reuse Water.mass as lava or silently erase reaction products. Acid and real heat
remain later increments. Source, tests and documentation committed/pushed as
`50444a8` on `codex/simulation-correctness-fixes`.
