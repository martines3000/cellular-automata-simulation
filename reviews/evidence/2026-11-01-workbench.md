# Workbench measurements and regressions — 2026-11-01

Baseline: `237ebc5` plus dev-only benchmark additions. Final measurements use the
workbench implementation on the same branch; its actual commit is recorded in
[STATUS](../../plans/STATUS.md). Historical assessment/research files are unchanged.
No dependency, toolchain, phase-order or physical-coefficient changes.

## Conditions and interpretation

- Apple M3 Pro, Mac15,6, 18 GiB RAM, macOS, AC power; Rust 1.95.0, release opt-level 2.
- Tracked Cargo.lock SHA256:
  `69d03b7273670b295a330e4922c3ebbe83eef9e99875db3ccad699506dfd8fb0`.
- Root Codex owned `/tmp/cellular-automata-simulation-verification.lock`; one
  checkout, jobs 2, one test thread. No overlapping GUI, builds, tests or benchmark
  jobs; GUI was stopped before final sampling. No other owner's process stopped.
- Ordinary desktop/background activity remained. Inventory snapshots showed
  variable browser/system activity before sampling and modest sync/system load
  before the final runs. This was not a controlled idle-machine experiment.
  Report repeats/spread rather than interpreting small differences as improvements.
- Divan 0.1.21, single thread, 30 requested samples, sample size 1, 100 ms minimum /
  1 s maximum including setup for case run limits. Actual counts vary (below).
  Fresh seeded input/setup is excluded from the reported operation time.
- Two initial runs established the baseline, two interim runs compared the first
  texture prototype, then two final runs compared the completed workbench paths.
  Raw console output stayed in `/tmp`; this compact extract is the retained evidence.

## Baseline evidence

Two run medians at 1024²: mixed tick 18.72 / 19.54 ms (27 / 26 samples), modern
Ice/Snow/Steam mix 17.54 / 18.27 ms (26 / 25), one grid copy 369.4 / 368.6 µs
(30 each), full mixed rectangle generation 9.639 / 10.25 ms (23 each). These
establish CPU costs; they are not measurements of GPU time or interactive FPS.

## Final paired renderer comparison

Same source, seeded world, 1280×820-point headless UI, Fit world, paused, no
fractional surfaces. Geometry uses batched solid quads; texture uses one visible-cell
color image and quad. UI work and texture delta preparation are included; egui
CPU tessellation, GPU upload/rasterization/presentation and simulation are excluded.
Each entry is the median from run 1 / run 2; counts are per run.

| Size | Detailed geometry | Fast overview | Samples geometry / overview |
| --- | --- | --- | --- |
| 100² | 161.6 / 173.8 µs | 161.3 / 179.7 µs | 62,60 / 62,59 |
| 256² | 1.120 / 1.129 ms | 351.5 / 357.9 µs | 30,30 / 30,30 |
| 512² | 4.720 / 4.779 ms | 1.588 / 1.597 ms | 28,27 / 30,30 |
| 1024² | 19.26 / 19.44 ms | 6.532 / 6.725 ms | 7,7 / 17,17 |

At 100² both auto paths choose detailed geometry; this is a control, not a texture
speedup. At 1024² observed per-sample ranges were 19.07–19.80 / 19.32–20.00 ms
for geometry and 6.475–6.575 / 6.576–7.338 ms for overview. Small sample counts
reflect expensive fresh UI setup and the time cap. About 2.9× lower preparation
cost in this large-world fixture supports the overview option; no end-to-end
speedup or FPS claim follows. The texture uploads up to 4 MiB for a full 1024²
image; native upload cost remains unmeasured.

The pinned backend uses linear filtering. Overview is deliberately smoothed below
3 points/cell; close views and fractional surfaces keep detailed geometry. Empty
texels are transparent, preserving guides. Geometry batches avoid native-visible
antialias seams between adjoining cell rectangles. Material highlights are static,
close-view-only, capped at 128 fire halos and 1024 surface accents per frame.

## Simulation attribution and GPU decision

Final full-tick run medians, in ms:

| Workload | 100² | 256² | 512² | 1024² |
| --- | --- | --- | --- | --- |
| Original mix | .09791 / .09758 | 1.117 / 1.119 | 4.642 / 4.848 | 18.62 / 19.46 |
| Modern mix | .08758 / .08787 | 1.028 / 1.020 | 4.435 / 4.446 | 17.85 / 17.92 |

Modern phase probes at 1024² (each includes its own copy; 30 samples/run):
sand 2.655 / 2.651 ms, water 5.036 / 5.049, fire 2.035 / 2.040,
smoke 1.593 / 1.595, wood 1.341 / 1.348, frozen 2.430 / 2.431,
Steam 1.293 / 1.301. Each phase starts from the same fresh snapshot, so summing
these does not reconstruct a sequential tick. One isolated copy was .3477 / .3690 ms.

The CPU tick approaches/exceeds a 60 Hz budget at 1024² under these fixtures.
GPU compute is a plausible separate experiment; the texture path accelerates
presentation preparation only. First profile actual native frame/upload costs,
then investigate CPU phase/copy work or a bounded compute prototype. Preserve
neighbor-write conflict resolution, phase barriers, water cleanup budgets and the
CPU reference. No GPU simulation, shader postprocessing or framework migration
was added. Existing Divan plus scoped CPU diagnostics sufficed without a new crate.
The existing basin tessellation case still measures legacy rectangle tessellation,
not the new app mesh's complete native frame.

## Correctness and native validation

118 correctness tests pass in debug and release, with 2 performance helpers ignored.
New cases cover:

- Batched/multi-sample strokes, all diagonal directions, overlaps, repeated samples,
  outside/re-entry without a connecting line, round/square maximum sizes and borders.
- Exact sparse undo, frozen/gas phase tools, sub-threshold and over-scale quantities,
  history count and allocated-capacity budget, command shortcut and keyboard focus.
- Pointer-centered zoom in/out, negative pan, no initial pan jump, secondary input
  without painting, overview/fractional/close-view switching and color coverage.
- Deterministic labs at 3/7/100/256/512/1024, unchanged borders/settings, and 80
  ticks per 100² lab checked against every phase's explicit conservation budget.
- Existing 101 physics/UI regressions retained, including all water-cleanup cases.

Native macOS inspection: wide strokes, Undo button and Command-Z, lab selection /
Snowfall load, floating Ice/Wood playback, seam-free basin fills and smoothed
texture overview. Short/minimum layouts and scroll-reachable controls are tested
headlessly. App was stopped after inspection. No browser, Windows or GPU timing
validation claimed; delivery remains native/local-only.

## Exact sequential commands and results

All use the pinned toolchain/lock. `bash check.sh` expands to:

```sh
CARGO_BUILD_JOBS=2 cargo check --locked --workspace --all-targets
CARGO_BUILD_JOBS=2 cargo build --locked --bin vaja_2
cargo fmt --all -- --check
CARGO_BUILD_JOBS=2 cargo clippy --locked --workspace --all-targets --all-features -- -D warnings -W clippy::all
CARGO_BUILD_JOBS=2 cargo test --locked --workspace --all-targets --all-features -- --test-threads=1
CARGO_BUILD_JOBS=2 cargo test --locked --workspace --doc -- --test-threads=1
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib --all-features -- --test-threads=1
```

PASS: check/build/fmt/strict Clippy, 118 debug and 118 release tests; 2 ignored
helpers; binary 0 tests; 0 doc tests. Separate native release binary build passed:
`CARGO_BUILD_JOBS=2 cargo build --locked --release --bin vaja_2`.

```sh
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_fixture_smoke_test -- --ignored --nocapture --test-threads=1
CARGO_BUILD_JOBS=2 cargo test --locked --release --lib performance_baseline -- --ignored --nocapture --test-threads=1
```

PASS: final untimed smoke covers 19 workloads × 4 sizes = 76 cases. Both final
sampling runs pass. Final subsequent change only made undo accounting charge Vec
capacity rather than length; full checks passed again, sampled default empty
brush/history workloads are unchanged. Initial implementation checks exposed
scroll-dependent test assumptions and one mesh test that counted a palette swatch;
these were corrected to target visible controls/canvas geometry, then rerun.
Historical dependency/web limitations remain in the original report; not rerun or
repaired here. No performance threshold is used as a test assertion.
