# Game interface and material direction

- Status: complete, committed/pushed as e14c62c, 2026-11-01; owner: Codex this chat.
- Context: user authorized a full native UI redesign and parallel research into
  five new materials, five later materials, and a more polished game aesthetic.
- Scope: native egui 0.17 interface, presentation, interaction regressions and
  cited design notes. Keep simulation phases/material rules and dependencies.
  Material implementation, GPU migration and timed benchmarks remain separate.

## Progress and implementation steps

- [x] Trace App/update, paint transforms, world timing, renderer and UI tests.
- [x] Establish clean starting tree at a097cc1; one worktree, no active checks/app.
- [x] Delegate material and visual research; research agents do no verification.
- [x] Replace overlapping windows with header, scrollable tool palette and canvas.
- [x] Verify input routing, pause/step, resize safety, compact layout and idle work.
- [x] Finish native visual inspection, local checks and documentation.
- [x] Review diff and preserve historical evidence.
- [x] Commit/push and hand off (`e14c62c`, same authorized branch).

## Decisions, discoveries and assumptions

- Working title: Strata. Navy/mineral background, warm text, mint selection and
  distinct material colors. No external assets or dependency upgrade needed.
- Keep World geometry/color snapshots intact; palette tint belongs to App's
  presentation layer and preserves merged shapes and fractional surfaces.
- No perpetual decorative animation; paused frames must settle. Hover/selection
  feedback and focused tool descriptions communicate state without idle CPU work.
- New elements are researched proposals, not silently added engine rules.
- Native app verification/builds are owned solely by this chat using the existing
  shared lock; research runs independently without build/test/GUI jobs.

## Validation

- Focused `cargo test --locked --lib game_ui -- --test-threads=1`: 10 pass.
- Initial compilation adapted to pinned egui APIs (Margin/Layout/Painter); removed
  the now-unused toggle_shift helper, replaced by an explicit checkbox.
- Tests exposed first-frame fit settling and coalesced pointer movement scrolling
  buttons away. Continuous fit mode and button drag ownership fix those cases.
- Native inspection: palette selection/description, sand placement/fall, live and
  paused states, disabled world editing, expanded terrain controls and generated/
  smoothed terrain inspected. The corner-drag resize attempt did not resize the
  native window; compact layout/scroll/resize are verified headlessly at 900×640.
  Owned native instance stopped before checks. No frame/GPU timing performed.

Final serialized verification, Rust 1.95.0 with unchanged committed lock (SHA-256
`69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`),
build jobs 2 / test threads 1:

```sh
bash check.sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1
```

`check.sh` passed every sequential native check/build, formatting, strict Clippy,
debug all-target/all-feature tests, doc tests and release all-feature library tests.
72 correctness tests pass in each profile; 2 performance helpers are ignored;
0 doc tests. Separate smoke passed all 32 cases without timing sampling. Exact
component commands are in the unchanged [check.sh](../../check.sh). Native final
palette review confirmed named, high-contrast cards with matching color swatches.
All owned native inspection processes were stopped; the temporary bundle removed.
No dependencies, lock or toolchain changes, CI, Windows/browser checks or GPU/frame
timings. Timed sampling stays deferred per user request.

## Recovery

Review the diff first; undo only this plan's owned UI/tests/docs hunks. Preserve
historical reports, unrelated work and research notes. Stop only this chat's app
and release only its verification lock. No clean/reset commands.

## Outcomes and supporting evidence

Implemented in `e14c62c`: fixed panels instead of overlapping windows;
explicit playback/step controls, focus-aware Space shortcut, scrollable named
palette, tool descriptions, Fit world (including window resize), sub-cell zoom,
optional guides and a presentation-only material palette. The core phase schedule,
water cleanup, merged geometry and twelve coverage snapshots are unchanged.

Owned implementation: `src/app/mod.rs`, new `src/app/theme.rs`, `src/main.rs`,
parent-visible World::step_once and obsolete toggle helper removal. Regression
support: app tests and world/app_timing_tests. User docs/performance comparison
notes and STATUS updated. Seven additional regressions bring correctness tests
from 65 to 72; the old collapsible-Display tests now exercise the persistent canvas
while settings expand/collapse.

- [Material proposals and conservation contracts](../../reviews/research/2026-11-01-materials.md)
- [Visual direction, effects and rendering roadmap](../../reviews/research/2026-11-01-visual-direction.md)

The five researched next materials and five later ideas are not implemented. Heat/
species state must be designed before phase changes; renderer/effects work should
follow profiling. The four protected historical assessment/evidence files remain
unchanged; no unrelated starting edits existed. Code and research are committed
and pushed as `e14c62c` on `codex/simulation-correctness-fixes`.
