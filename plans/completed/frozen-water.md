# Frozen water: Ice, Snow and phase brushes

- Status: implemented and verified in the working tree, 2026-11-01; owner: Codex this chat.
- Start: clean tree at eae4ff3; one worktree, no verification/simulator processes.
- Context: continue the [material roadmap](../../reviews/research/2026-11-01-materials.md)
  after the game UI. Timing measurements remain deferred.
- Scope: two material types, conserved freeze/melt editing, first movement/contact
  rules, palette integration and regressions. No temperature/energy solver,
  Steam/Lava/Acid, dependency changes or GPU migration.

## Steps

- [x] Trace paint, phases, scalar uses, snapshots, schedule oracles and UI callers.
- [x] Choose bounded scripted rules and quantity representation below.
- [x] Implement materials, phase brushes and UI.
- [x] Verify conservation, thresholds, conflicts, border/edit integrity and UI input.
- [x] Complete serialized local checks and native inspection.
- [ ] Record outcomes, commit/push and hand off.

## Decisions and assumptions

- Keep Cell at 8 bytes. Its existing mass is explicitly water-equivalent quantity
  for Water/None/Ice/Snow; smoke lifetime stays a separate meaning, excluded from
  a typed quantity accessor. No universal mass reinterpretation or thermal grid.
- Painting Ice places 10 water units; Snow places 1. Painting/erasing replaces cells
  and is an explicit source/sink, as other painting already is. Freeze/Melt tools
  instead preserve every bit of stored water, including passive/sub-threshold
  quantities and amounts above MAX_MASS (a scale, not a cap).
- Freeze applies only to positive water quantities in Water/None; melt changes
  Ice/Snow to Water at >= MIN_MASS, otherwise None with passive quantity retained.
  Brushes ignore unrelated materials and the border. No numeric temperature claim.
- Original Sand → Water → Fire → Smoke → Wood relative order remains. A sixth
  frozen-material phase moves Ice/Snow at most once, only into eligible destinations.
  Ice rises through Water (conservative swaps), falls through empty space and rests
  at the water surface. Snow falls/slides like powder and melts beside Water.
- Fire melts adjacent Ice/Snow during the fire phase before its existing movement/
  combustion; each target changes type once. New liquid flows next water phase.
- Explicit brush/contact conversion has no energy model: this is scripted game
  behavior. Steam and real heat transfer require a separately designed state model.

## Validation / ownership

Codex owns all checks and native inspection. Acquire the existing shared directory
lock after checking worktrees/processes; jobs 2 / test threads 1. Pin/lock unchanged.
Meaningful cases: exact water budgets, threshold and >MAX quantities, blocked and
multi-source motion, minimal borders, old-material compatibility, repeated full
mixed ticks, paused edits, rendered colors, thermal UI routing and short-window
scrolling. Run bash check.sh and the non-timing workload smoke sequentially.

## Recovery

Review the diff and revert only this plan's owned source/test/doc hunks; preserve
historical evidence and unrelated edits. Remove only owned scratch outputs, stop
only this chat's native app and release only this chat's verification lock.

## Outcomes

Implementation and results pending. Record actual revisions only after committing.

Targeted milestone: 88 debug correctness tests pass, 2 performance helpers ignored.
Includes 14 frozen-world tests, two real-egui phase-tool tests, 96 seeded mixed
fixtures × 24 ticks with per-phase cleanup budgets, and 100/1024 full-tick stress.
The resize safety test uses a taller fixture so the whole expanded toolbox is in
view; the separate 900×640 regression exercises scrolling to the same control.
Old material geometry snapshots remain unchanged. Final results follow below.

## Final verification (2026-11-01)

Sole verification owner: Codex this chat; shared directory lock held throughout
checks and native inspection, then released. Installed/pinned Rust 1.95.0, jobs 2,
test threads 1; committed dependency graph unchanged (Cargo.lock SHA-256
`69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`).

```sh
bash check.sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1
CARGO_BUILD_JOBS=2 cargo build --locked --release --bin vaja_2
```

PASS: native check/build, fmt check, strict all-feature/all-target Clippy, 88 debug
and 88 release correctness tests, 0 doc tests. Two performance helpers ignored by
normal checks; separate non-timing smoke passed all 32 cases. Release binary built.
Initial strict check found an import used only by child tests; gated it with
cfg(test), then reran the full sequence successfully. No baseline failures remain.

Native release inspection confirmed palette/phase-tool labels and selected states,
Ice drag placement, Melt rendering Water, Freeze rendering Ice, and selecting Snow
leaving the phase tool. Quantities, Snow transport, ice buoyancy, fire interactions
and compact scrolling are established by deterministic headless regressions, not
screen-color observations. Owned app stopped and temporary bundle removed.

Cell remains 8 bytes; two 1024² grids remain 16 MiB. Original coverage snapshots
pass unchanged. A tick now has six phases/copies; benchmark comparison notes updated.
No new timing/FPS/GPU claim. No CI, Windows/browser checks or dependency upgrades.
Historical reports/extracts remain unchanged; no unrelated starting changes existed.

## Outcome and next bounded task

Working-tree implementation: Cell types/quantity accessor, frozen movement module,
scripted fire contact, Freeze/Melt paint routing and palette descriptions. Shared
paint hit testing and phase oracles updated; 16 additional regressions (14 world,
2 UI), including threshold bit neighbors, multi-source targets, repeated mixed
phase budgets, minimal borders and larger worlds. README explains quantities and
the source/sink distinction between painting and phase tools.

Next: design/implement Steam with a conserved water-equivalent quantity and explicit
condensation/blocked-gas rules, distinct from expiring Smoke. Real heat/energy,
Lava/Acid and optional visual effects remain separate increments; timings stay
deferred. Actual implementation revision will be recorded after committing.
