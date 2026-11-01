# Repository review progress

Review date: 2026-10-30 (Europe/Ljubljana)
Repository: /Users/martines/Desktop/git_repos/personal/cellular-automata-simulation
Scope: Analysis only; preserve existing source, configuration, manifests and lockfiles. Deliver self-contained review-report.html and this progress file. Temporary evidence/build artifacts permitted.

## Status — COMPLETE
- All requested review phases completed; final report has all twelve requested sections.
- Deliverables: `review-report.html` (self-contained HTML, approximately 345 kB) and `review-progress.md`.
- 16 significant findings: 7 High, 7 Medium, 2 Low; no Critical claims. Confirmed defects separated from measured costs, static bounds and optional creative ideas.
- Existing application/configuration/lockfiles untouched. All 26 tracked file hashes match the initial clean baseline; no Cargo.lock or target created in the original checkout.
- Native check/build and formatting pass; original tests pass with zero tests. WASM check and strict Clippy fail for documented reasons. Review probes: debug 8/15 pass, release 9/15 pass; expected assertion failures expose current behavior.
- Full evidence, temporary harness and baseline lockfile embedded in HTML. HTML structure/anchors validated; desktop and narrow browser previews checked; tables contain horizontal overflow and disclosures work.
- Owned native process and report preview server stopped. No tool upgrades, source fixes, features, deployment or new simulation assets performed.

The sections below are a chronological audit trail, including earlier hypotheses and next-step notes superseded by this completion status.

## Evidence and commands
- Read user attachment Pasted text.txt (full requirements).
- Ran pwd and rg for AGENTS.md; none found in repository inventory so far.

## Continuation
1. Record git revision, branch, working-tree baseline and environment.
2. Read project files and skills, then run non-destructive baseline checks with temporary build outputs.
3. Save findings with exact line references here throughout review.
4. Verify dependency releases from primary sources; create and validate HTML report.

## Baseline captured
- Branch master, commit 1792c2f7dc735be4663fc2f2683cf43fdc473839. Original git status clean; no repository/ancestor AGENTS.md found.
- Environment: macOS Darwin 25.6.0 arm64; rustc/cargo 1.95.0; wasm32 target installed; Node v24.15.0; wasm-bindgen-cli 0.2.114; jq present; wasm-opt absent. Docker CLI present (daemon not yet tested).
- Validation isolated in a temporary repository copy; exact location saved in /tmp/cellular-review-location. It contains tracked-file SHA-256 baseline, logs and copied sources. No original lockfile exists; Cargo.lock is ignored.
- `cargo fmt --all -- --check`, `bash -n build_web.sh setup_web.sh check.sh`, and `node --check run.js` passed in copy.
- Native cargo check running; official registry lookups running.

## Static findings to verify
- Fire signed neighbor offsets cast to usize before addition: world/mod.rs:585-590; debug arithmetic overflow expected when supported fire updates.
- Smooth only copies cell_type into stale tmp then swaps entire Cell grid: 213-254; water/smoke mass can revert on Smooth. Generation leaves prior mass (201-209).
- Sand update reads old data but writes tmp occupied sources: 352-362; stacked particles may lose grains.
- Water cutoff overwrites accumulated incoming tmp mass at 402-405; test stacked cells with mass below MIN_MASS.
- Wood sideways water displacement uses flow >= 0 and does not clear origin until positive resulting mass; left branch omits Wood from accepted types (437-505).
- Smooth does not re-add border; under low-density caves boundary may open, invalidating unchecked neighbor accesses in simulation/rendering.
- Rendering emits shapes for air and offscreen cells (87-169). Five clones + five full update passes per tick (638-655). Simulation tied to render timing and millisecond truncation (172-173, 630-636); repaint always requested (app/mod.rs:46).
- Web script guesses package from folder despite Cargo package vaja_2. Checkout folder cellular-automata-simulation fails cargo -p. Stray ! before assignment is not yet treated as standalone failure. docs WASM binary absent and ignored. Root-relative PWA URLs break repository subpath hosting.
- run.js concatenates raw URL into fs path; verify traversal only with benign temp fixture.

## Next steps
Complete builds; create isolated Rust rule probes from unchanged sources, verify material counts/masses/borders and debug/release behavior. Verify static server with benign fixtures. Read preview image and attempt native runtime/UI only if build works. Resolve direct dependency latest/resolved versions and tool sources; build HTML incrementally.

## Validation phase complete; targeted probes in progress
- `cargo check --workspace --all-targets`: PASS (14.08s). `cargo build --bin vaja_2`: PASS. Native process launched without stderr; CUA cannot bind the unbundled executable (Invalid app), so no interactive native smoke claim.
- `cargo test --workspace --all-targets --all-features`: PASS with **0 tests**. `cargo test --workspace --doc`: PASS with **0 doctests**.
- WASM lib check with `RUSTFLAGS=--cfg=web_sys_unstable_apis`: FAIL in egui_web 0.17.0; current web-sys scroll f64 versus offset i32 and clipboard returns Clipboard instead of Option. Transitive API drift, not absent WASM target.
- Strict Clippy command from check.sh FAILS on six lint diagnostics. Formatting passes.
- Temporary test-only module appended to isolated world source, original repository untouched. Initial probes confirm fire debug overflow at line 586, water 1.005 -> 0 after update, Smooth water 7 -> 0, generation retains 7 mass under air, left-side wood excluded from displacement.
- Sand stacked probes so far PASS; do not report hypothesized sand loss. Cave smoothing border test PASS; initial hypothesis of border erosion disproved for valid solid border. Preserve these distinctions.
- Benign Node server fixture: GET / -> 200, GET /../review-public-sentinel.txt -> 200 sentinel outside web root, GET /index.html?test=1 -> 404. No real private files accessed.
- Preview image inspected: white square-cell canvas, primary-color materials, floating egui windows, generic controls, no palettes/presets/brush UI.
- Official crates.io API versions saved to registry.json in temporary evidence directory. Latest stable as retrieved 2026-10-30: egui/eframe 0.37.0 (MSRV 1.95), ndarray 0.17.2, rand 0.10.3, getrandom 0.4.3, serde 1.0.229, instant 0.1.13 (upstream explicitly unmaintained), panic hook 0.1.7, tracing-wasm 0.2.1, wasm-bindgen/CLI 0.2.130. Isolated resolution: ndarray 0.15.6, rand 0.8.8, direct getrandom 0.2.17, egui/eframe 0.17.0. Installed CLI is 0.2.114, mismatches fresh graph 0.2.130.
- Primary web sources: Rust official releases latest 1.99.1; eframe CHANGELOG; Node official download latest stable/LTS 26.13.0 and Node 16 EOL; Binaryen official latest version_133.
- Incremental HTML exists with preliminary summary.

## Next steps
Finish 15 rule probes in debug and release (release compilation ongoing); pressure/wood conservation probe and symmetric water bias added. Run one bounded release CPU microbenchmark (empty bordered worlds; no FPS/GPU claim). Complete official tool verification. Generate full twelve-section HTML with <=20 significant findings and extensive grounded feature/visual tables. Include exact isolated harness and relevant logs in report collapsibles. Browser QA the final report; verify tracked file hashes unchanged, stop only owned runtime/server processes, update this file to completion.

## Rule probes and measurement complete
- Review-only tests: debug **8 pass / 7 fail**, release **9 pass / 6 fail** (15 probes). Expected invariant failures provide evidence; not an existing repository suite.
- Fire overflow is debug-only in measured profile; release interior arithmetic wraps and the test passes. Do not claim release fire always crashes.
- Water cutoff: 1.005 water units -> 0 in one isolated water pass.
- Smooth: water 7 -> 0; rand_generate threshold 0 leaves mass 7 under air.
- Wood lifting: supported 20-water cell lifting wood into a 5-water cell yields total 25 -> 20. Left Wood never accepted by left-side flow guard.
- Water symmetry: identical open sides receive 1.875 (left) versus 2.5 (right) from a 10-unit source on dirt. Treat as visible directional bias, not mass loss.
- Passing probes: stacked sand (both free and blocked diagonals), solid border after smoothing, two-step sand near boundary, pointer pan/scale round trip, shape-count/size, zero-flow wood count, interval conversion. These are narrow checks, not exhaustive proofs.
- Release CPU-only empty bordered-world microbenchmark (median ms; ticks manually execute unchanged five phases, excluding throttle):
  - 100², 100 samples: tick 0.0500, shape generation 0.0308, 10,000 shapes.
  - 256², 100 samples: tick 0.3087, shapes 0.2511, 65,536 shapes.
  - 512², 50 samples: tick 1.5276, shapes 2.5881, 262,144 shapes.
  - 1024², 10 samples: tick 6.1504, shapes 9.9056, 1,048,576 shapes.
- Cell is 24 bytes on tested arm64 host. Two 100² grids ~469 KiB; two 1024² grids 48 MiB. Five clones copy 1.2MB/tick at 100², 125.8MB/tick at 1024² (payload only; read/write traffic higher). No GPU/tessellation/frame timings or WASM performance measurements.
- build_web.sh executed in isolated copy: fails cargo package ID lookup based on temp folder name. Default real repository folder also differs from package vaja_2. Stray ! does not cause early abort; do not misreport syntax failure.
- Tool verification: jq 1.8.2 installed/latest; Binaryen latest version_133 absent locally; Node release index latest stable 26.13.0 LTS; web-time candidate 1.1.0. Source URLs and raw data retained in tools-latest.json and registry.json.

## Report authoring underway
Next: write full report with concrete reproducible probes, preserved source links, feature combinations, three creative directions, visual/performance priorities, staged milestone and validation limitations. Browser QA, hash verification and process cleanup remain.

## Report content drafted
- Twelve requested sections drafted in generator /tmp/build_cellular_review.py (final HTML generation pending).
- 16 significant findings: build compatibility/package names; fire debug overflow; water cleanup, stale smoothing, generation residue, wood lifting and symmetry; time/render coupling; serving traversal; PWA subpath/artifact delivery; setup reproducibility; full-grid geometry/clones/idle repaint; click-only painting gap. Findings have severity, confidence, source links, evidence, impact, recommendation and effort.
- Feature table covers immediate brush/step/presets/save tools plus stone/ash, connected caves, heat/steam/ice, oil/fuel, plants, erosion, lava/glass, electrical devices, reactive materials and weather/challenges. Recommends coherent dam/raft → boiler cavern → living watershed progression.
- Three visual directions: scientific workbench, polished pixel laboratory (recommended), atmospheric cavern. Rendering options preserve egui/CPU model; no rewrite assumed.
- Roadmap stages and exit criteria drafted; first milestone Reliable Pixel Sandbox deliberately excludes thermal/compute expansion.
- SHA-256 verification performed: all 26 original tracked files unchanged at this point.

## Remaining
Finish evidence/limitations section, generate report, validate internal links/structure, browser visual QA. Stop owned native runtime if alive. Final source hash/git state check and completion update.

## Final handoff
- Review completed at master / 1792c2f7dc735be4663fc2f2683cf43fdc473839, 2026-10-30 Europe/Ljubljana.
- Final HTML: /Users/martines/Desktop/git_repos/personal/cellular-automata-simulation/review-report.html
- Progress: /Users/martines/Desktop/git_repos/personal/cellular-automata-simulation/review-progress.md
- Temporary evidence directory: /var/folders/s4/lwy4cp0d4f1dkhzv5fsd497c0000gn/T/cellular-review-didg3h7t. Location marker: /tmp/cellular-review-location. Report generator: /tmp/build_cellular_review.py. Temporary paths may be cleaned by the OS; HTML embeds the important logs, harness, registry evidence, source hashes and review-resolution lockfile.
- Browser QA: default narrow preview; desktop breakpoint tested via temporary viewport override; narrower 354-CSS-pixel view tested; no document-wide horizontal overflow. All internal anchors resolve. Evidence disclosure opens. Viewport reset and QA tab closed.
- Final scope limits: no successful interactive native/browser app smoke, cross-platform or container build, GPU/frame/dense-fluid measurement, full vulnerability-database audit or exhaustive rule proof. See section 12 for precise limitations.
- Nothing remains to complete this analysis-only request. Recommended implementation work is intentionally unperformed and described in the report.
- If asked to implement next: reread the report/requirements, verify git status and commit again, agree/select the next milestone from the roadmap, then work under that new authorization. Do not mistake the temporary test-only module for a production patch.
