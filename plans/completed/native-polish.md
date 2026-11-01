# Native interaction, rendering and reproducible local validation

- Owner: Codex, this chat; one editor/verification owner.
- Authorized: all seven proposed next tasks, 2026-10-31. Starting at `faceeff`
  on `codex/simulation-correctness-fixes`, clean checkout, one worktree.
- Scope: F16 clicks/dragging, F15 paused redraw, F14 scratch reuse, F13 rendering,
  six existing Clippy diagnostics, actual native UI validation and native
  lock/toolchain policy. CI removed by subsequent user instruction.
  No new simulation features or dependency upgrades; preserve historical evidence.

## Progress

- [x] F16: one canvas response, single-click/drag painting; input/coordinate regressions.
- [x] F15: event-driven paused repaint; retain active simulation and interaction.
- [x] F14: copy into allocated scratch; equivalence and allocation-identity checks.
- [x] F13: skip empty/offscreen shapes; retain water/wood geometry; reuse shape capacity.
- [x] Remove six baseline lint diagnostics and make the strict check script pass.
- [x] Native UI: actual startup, pause/play, rate, painting, zoom, Clear/Generate/Smooth.
- [x] Pin installed Rust 1.95.0 and track the current native Cargo.lock; local-only checks.
- [x] Final sequential checks and unchanged historical hashes; commit/push closure records.

## Decisions and validation

Use real egui RawInput frames for headless interaction coverage. Preserve phase
order, water semantics and timing policy. Reuse ndarray storage without changing
which grid each phase reads. Verify rendering against expected clipped geometry,
including shifted water/wood. No speedup/battery claims without controlled runtime
measurements; shape counts and allocation reuse are structural evidence.

Use the existing native lock graph; record its SHA and avoid upgrades. Rust 1.95.0
is already installed. A native application should commit its resolution. Local
checks use locked builds and the same toolchain. CI was initially added,
then cancelled, disabled and removed when the user requested local-only testing.
Record actual platform validation rather than infer success.

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

F16/F15 commit: `18ff7e5`; all 42 tests passed. F14 allocation-identity regression
failed before assign() reuse, then passed. A 100-tick clone-reference comparison
passes, including complete data/tmp scalar equality. Tick and Smooth reuse the
original two buffers. `cargo +1.95.0 test --lib scratch_ -- --test-threads=1`
(jobs 2): 2 passed. Full all-target/all-feature suite: 44 passed. No throughput
or wall-clock performance claim; per-cell coordinate removal remains separate.

F13: five geometry regressions plus shape-storage reuse pass; full suite 50 passed.
Empty 100×100 world: zero cell shapes (previously 10,000). Visible solid fixture:
12 aligned / 20 fractionally panned cells. Water/wood surface geometry, viewport
edges and invalid viewport inputs covered. egui 0.17 consumes Vec in extend(), so
submit drained shapes via Painter::add to retain App capacity. No FPS claim.
Validation: `cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1`
with jobs 2, and `cargo +1.95.0 fmt --all -- --check`.

Lint cleanup: redundant field, needless returns, identical Smooth branches and
nested wood condition simplified without changing rules. Renderer already removed
the unnecessary cast. Strict all-target/all-feature Clippy now passes with
`-D warnings -W clippy::all`; all 50 tests and desktop binary build pass.

Native UI (Apple Silicon macOS, debug binary at `3a32a09`): temporary app wrapper
in /tmp enabled native computer-use access. Observed bordered world, water/sand
painting, Block size 5→12→5, Steps/s 60→300, Toggle disabling/enabling editing,
sand arriving at bottom border, Generate random cave, Smooth changing cave and
Clear restoring empty bordered world. Paused painting retained cells. Native
drag automation did not move reliably (also failed to move window titles), so
drag correctness relies on deterministic RawInput tests; no runtime drag or FPS
claim. Screenshots remained tool output; no binary/private runtime data tracked.
Owned application stopped before resuming compiler checks.

Build policy: track the existing 325-package native lock graph unchanged, SHA256
`1253e4b644402f3ae38bc0193886c1995e7d148384ff9dd8d69ac887fd7e3098`.
Pin already installed Rust 1.95.0, use --locked in checks; no version upgrades.
CI was briefly added in `af94baf`, then superseded by the user's local-only
instruction. Run 41194399117: Ubuntu job passed the full script (50 debug / 50
release); macOS job confirmed cancelled, workflow confirmed disabled_manually,
then its configuration removed.
Linux emitted an existing quick-xml 0.22.0 future-compatibility warning; dependency
migration remains separate. No further Actions runs are needed or authorized.

Local pinned/locked `bash check.sh`: PASS (check, binary build, fmt, strict
Clippy, 50 debug/all-feature tests, 0 doc tests, 50 release/all-feature tests).
Shell syntax and diff whitespace checks pass. Exact commands are in check.sh;
CARGO_BUILD_JOBS=2 and test threads 1 throughout. No source changes followed this
verification; closure/CI removal is documentation/configuration only.

Closure: F16/F15 `18ff7e5`, F14 `64651d4`, F13 `3913e5f`, lint `3a32a09`, UI
validation `611c83b`, pinned builds `af94baf`. All four historical evidence files
compare byte-for-byte with `1d407f1`; all planning/evidence files are tracked and
not ignored. Temporary app wrapper removed; owned native process stopped. No
unrelated working tree edits were present. Recovery/limits above remain valid.
