# Implementation status

Updated: 2026-11-01. Historical baseline: master at `1792c2f`.
Assessment: [review-report.html](../review-report.html#F04).

## Verification coordination

- Owner: Codex, this chat; sole editor of `src/app/world/mod.rs`.
- Active job: none. Workbench checks, native inspection and CPU sampling complete;
  owned app stopped and shared lock released. Commit/push handoff follows.
  [Completed plan](completed/game-workbench.md).
- Shared lock for cooperating workers:
  `/tmp/cellular-automata-simulation-verification.lock` (directory; owner recorded
  inside). Inspect process inventory before acquiring; never remove another owner's
  lock or stop their processes. Run one verification command at a time.
- Settings used: installed Rust 1.95.0, build jobs 2, test threads 1.

## Finding resolutions and backlog

| Priority | Finding | Status / next bounded task |
| --- | --- | --- |
| Committed/pushed | F04 water cleanup | `1d407f1` on `codex/simulation-correctness-fixes`. [Completed plan and exact commands](completed/f04-water-cleanup.md). Debug/release: 7/7 each; native checks/tests/format pass; six unchanged baseline Clippy diagnostics. |
| Retired by removal | F10 path traversal | Node server removed with browser delivery; historical evidence unchanged. [Cleanup plan](completed/native-only.md). |
| Committed/pushed | F03 fire indexing | `b99b9de`; 3 fire tests (960 supported configurations), 10 total tests pass. [Completed plan](completed/f03-fire-indexing.md). |
| Committed/pushed | F05/F06 editing state | `6236c6e`; 4 edit tests and 14 total tests pass. [Completed plan](completed/f05-f06-edit-state.md). |
| Committed/pushed | F07 displacement | `7195209`; 19 total tests pass, including 6,561-case mass matrix. [Completed plan](completed/f07-wood-displacement.md). |
| Committed/pushed | F08 direction / F07 follow-up | `01b4340`; 27 total tests pass in debug/release/all-features. [Completed plan](completed/f08-lateral-flow.md). |
| Retired by removal | F01/F02/F11/F12 | Browser entry point, build/hosting scripts and assets removed at user request; no web compatibility claim. [Cleanup plan](completed/native-only.md). |
| Committed/pushed | F09 timing | `cc3c42f`; fixed-step accumulator, 16-step catch-up cap, pause/rate resets and Steps/s label. 37 tests pass. [Completed plan](completed/f09-fixed-step.md). |
| Committed/pushed | F16 input / F15 repaint | `18ff7e5`; primary clicks/dragging, coalesced click handling and idle paused frames; real egui input regressions. [Native follow-up plan](completed/native-polish.md). |
| Committed/pushed | F14 scratch reuse | `64651d4`; original two allocations retained; 100-tick equality against clone reference. |
| Committed/pushed | F13 rendering | `3913e5f`; visible-cell culling, empty-cell omission and shape capacity reuse, geometry regressions. |
| Committed/pushed | Display visibility/timing | `d68114d`; collapsed Display keeps active simulation advancing; preserves pause and paint/update order. [Completed plan](completed/display-visibility.md). |
| Committed/pushed | Mixed-material water preservation | `563cfb2`; sand/fire/smoke displacement preserves passive water; seeded full-phase budgets. [Completed plan](completed/mixed-material-performance.md). |
| Committed/pushed | Larger-world setup and scaling | `449ed04`; 100/256/512/1024 controls, compact 8 B cells and row geometry merging; local-only Divan harness. |
| Measured / native GPU follow-up | CPU/GPU/frame measurements | [Completed CPU sampling](completed/local-performance-measurements.md); workbench evidence below. GPU/presentation timings remain separate. |

The user's 2026-10-30 follow-up authorized committing/pushing on a fresh branch
and continuing work; on 2026-10-31 the user requested edge-case regressions.
The simulation correctness sequence F03–F08 is complete. Resolution details belong
in completed plans. F04's four historical evidence files are now tracked unchanged.

Latest local verification (2026-10-31): `bash check.sh` passes using Rust 1.95.0
and the committed lock: native check/build, fmt, strict Clippy, 65 debug and 65
release all-feature tests; 2 performance helpers ignored; 0 doc tests. Separate
non-timing harness smoke passes all 32 workload/size cases. No timed sampling ran.

Exact scaling checks: Cell 24→8 B; two 1024² grids 48→16 MiB; full dense-basin
rectangles 526,334→2,557. Twelve seeded fractional/panned coverage snapshots match.
Native check confirmed resize to 1024 and expanded pan limits; later automated
input failed while the process remained alive. Further native visual validation
and GPU/frame timings remain unverified. [Dated evidence](../reviews/evidence/2026-10-31-scaling.md).

Reproducible native builds: `af94baf`. The native graph was committed
unchanged at that milestone; current lock adds 8 dev-only benchmark packages.
Rust 1.95.0 is pinned and Cargo checks are locked. The user subsequently
requested local-only testing: CI run confirmed cancelled, workflow confirmed disabled
and removed, with README/AGENTS updated. No future Actions runs are configured.
Ubuntu's brief CI run passed; it reported a quick-xml 0.22.0 future-compatibility
warning. Dependency migration remains separate. Browser delivery was removed
(`778f2c8`); no browser validation claimed or required.

All sixteen historical findings are implemented or retired by browser removal.
Historical reports and extracted evidence remain byte-for-byte unchanged. Counts
in earlier rows describe those commits; current suite contains 101 correctness
tests plus 2 ignored performance helpers (including the follow-ups below).

[Completed native follow-up](completed/native-polish.md) and
[Display visibility fix](completed/display-visibility.md) retain their earlier
commands/results. [Completed mixed-material/scaling setup](completed/mixed-material-performance.md)
records the current changes, exact checks and limits.

Remaining priorities: profile actual native frames/upload costs after the workbench
CPU samples below, then compare a bounded CPU or compute prototype against the oracle;
dependency compatibility remains separate (historical quick-xml warning). Windows
support is removed from the backlog; CI remains removed. Benchmark tools and GPU
tradeoffs are in [PERFORMANCE.md](../PERFORMANCE.md) and its linked research.

Latest follow-up (2026-11-01): [Game interface and material direction](completed/game-ui.md)
committed/pushed as `e14c62c` on `codex/simulation-correctness-fixes`. Fixed game
layout, material cards/swatches, playback/step/Space controls, Fit world and optional
guides. `bash check.sh` passes strict Clippy/fmt/native build/check, 72 debug and 72
release correctness tests, 2 ignored helpers, 0 doc tests. Non-timing smoke passes
all 32 benchmark cases. Native selection/paint/playback/terrain/palette reviewed;
compact layout and scrolling tested headlessly. Historical evidence unchanged.

Next design work: [Ice/Snow/Steam/Lava/Acid](../reviews/research/2026-11-01-materials.md)
requires explicit heat/species budgets before implementation; later Stone/Oil/Glass/
Plants/Salt. [Visual roadmap](../reviews/research/2026-11-01-visual-direction.md)
proposes finite feedback, material effects and presets, with profiling before
renderer/GPU changes. These are researched proposals, not implemented features.

Latest material follow-up: [Frozen water](completed/frozen-water.md), committed/
pushed as `a2d8038` on `codex/simulation-correctness-fixes`. Ice/Snow preserve water-equivalent quantities;
Freeze/Melt tools, falling/floating/piling and scripted fire/water-contact melting
are integrated. Full local checks pass: 88 debug/release correctness tests,
strict Clippy/fmt, native debug/release builds, 0 doc tests; 2 ignored helpers.
Separate non-timing workload smoke passes all 32 cases. Native Ice/Melt/Freeze
presentation checked. Cell stays 8 B; no dependencies changed. Implementation and records are committed.

Next bounded material task: conserved Steam transport and condensation, separate
from Smoke lifetime; real thermal energy, Lava/Acid and visual effects remain
separate increments. The dated research stays historical; Ice/Snow are now shipped
through the scripted rules described in README. Timings remain deferred.

Latest material follow-up: [Steam](completed/steam.md), committed/pushed as
`50444a8` on `codex/simulation-correctness-fixes`. Steam rises/drifts, bubbles through Water,
retains trapped quantity, condenses beside Ice/Snow, and has conserved Vaporize/
Condense tools. Fire vaporizes pre-existing liquid; snapshot rules prevent freshly
melted frozen material from vaporizing again in the same fire phase.

`bash check.sh` passes: 101 debug/release correctness tests, strict Clippy/fmt,
native check/build and 0 doc tests; 2 ignored helpers. Separate untimed fixture
smoke passes all 32 cases; release app built. Native Steam/Condense presentation
checked; a later UI action failed with noWindowsAvailable while the app remained
alive, so Vaporize is verified headlessly. App stopped, lock released. Cell stays
8 B; seven phases now copy before updating; benchmark comparison notes updated.

Next bounded task: Lava plus Stone with explicit reaction quantities; Acid/thermal
energy and optional effects remain separate. No performance timings collected.

## Current handoff — simulation workbench

Implemented in the working tree; commit/push pending final diff inspection.
[Completed plan](completed/game-workbench.md), [measurements and exact checks](../reviews/evidence/2026-11-01-workbench.md).
Earlier timing deferrals and proposed priorities above are chronological; the user
subsequently authorized measurement, brushes, visuals and continued implementation.

- Fast smoothed overview for large zoomed-out worlds; solid batched close-view quads
  remove rectangle seams. Fractional geometry remains available; optional bounded
  static material accents and CPU diagnostics added.
- Round/square 1–63-cell continuous brushes, once-per-stroke quantity placement,
  bounded paused undo / Cmd/Ctrl+Z, pointer-centered zoom and right/middle pan.
- Floating Ice, Steam Chamber and Snowfall labs at every supported size; app opens
  in the paused floating-ice experiment. Loading a lab replaces the world explicitly.
- Full check.sh passes: 118 debug and 118 release tests, 2 ignored performance
  helpers, 0 doc tests, native build/check, fmt and strict Clippy. Release app built;
  native interaction/presentation reviewed and app stopped. Untimed 76-case smoke
  and two final measurement runs pass. Historical evidence/lock/toolchain unchanged.
- At 1024², matched UI preparation medians: geometry 19.26/19.44 ms vs overview
  6.532/6.725 ms. CPU-only, variable desktop background load; no FPS/GPU claim.
- Next priorities: native GPU/frame/upload profiling; equivalence-tested CPU phase /
  memory work or bounded compute prototype; further lab/brush polish; explicit
  Lava/Stone reaction design before additional chemistry. No Windows/CI/web backlog.
