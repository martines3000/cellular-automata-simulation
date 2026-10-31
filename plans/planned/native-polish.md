# Native interaction, rendering and reproducible validation

- Owner: Codex, this chat; one editor/verification owner.
- Authorized: all seven proposed next tasks, 2026-10-31. Starting at `faceeff`
  on `codex/simulation-correctness-fixes`, clean checkout, one worktree.
- Scope: F16 clicks/dragging, F15 paused redraw, F14 scratch reuse, F13 rendering,
  six existing Clippy diagnostics, actual native UI validation, native lock/toolchain/CI.
  No new simulation features or dependency upgrades; preserve historical evidence.

## Progress

- [x] F16: one canvas response, single-click/drag painting; input/coordinate regressions.
- [x] F15: event-driven paused repaint; retain active simulation and interaction.
- [ ] F14: copy into allocated scratch; equivalence and allocation-identity checks.
- [ ] F13: skip empty/offscreen shapes; retain water/wood geometry; reuse shape capacity.
- [ ] Remove six baseline lint diagnostics and make the strict check script pass.
- [ ] Native UI: actual startup, pause/play, rate, painting, zoom, Clear/Generate/Smooth.
- [ ] Pin installed Rust 1.95.0, track the current native Cargo.lock, add native CI.
- [ ] Final sequential checks, unchanged historical hashes, clean commits/push.

## Decisions and validation

Use real egui RawInput frames for headless interaction coverage. Preserve phase
order, water semantics and timing policy. Reuse ndarray storage without changing
which grid each phase reads. Verify rendering against expected clipped geometry,
including shifted water/wood. No speedup/battery claims without controlled runtime
measurements; shape counts and allocation reuse are structural evidence.

Use the existing native lock graph; record its SHA and avoid upgrades. Rust 1.95.0
is already installed. A native application should commit its resolution. CI uses
locked builds/checks and the same toolchain. Start with macOS/Linux native jobs;
record which platforms actually ran locally/remotely rather than infer success.

Before verification inspect worktrees/processes, acquire the shared directory lock
from STATUS, use jobs 2 / test threads 1, and run checks/UI measurements serially.
Native UI may require a temporary macOS app wrapper; keep it and screenshots/logs
outside the repo, preserve existing user processes and stop only owned instances.

## Recovery

Each bounded change gets its own commit. Revert only owned commits/hunks after
reviewing later changes. Preserve historical files and unrelated work. Release only
the owned lock and terminate only owned processes. No git clean.

## Outcomes

F16/F15 reproduction: a normal two-frame click already paints in egui 0.17,
contrary to the old static F16 interpretation. Preserve the historical report;
current RawInput tests establish that a same-frame press/release click was lost,
secondary clicks painted, and paused frames never became idle. All now pass.
One canvas response handles primary input; release does not double-add water.
Clipping rejects off-canvas/border positions, including under pan/zoom. Four egui
input-frame tests and a world clipping regression pass. Exact targeted commands:
`CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib app::tests -- --test-threads=1` and
`CARGO_BUILD_JOBS=2 cargo +1.95.0 test --lib painting_ -- --test-threads=1`.
The old egui API has immediate repaint only, so active simulation continues to
request repaint; paused simulation relies on egui's input-driven repaint.
