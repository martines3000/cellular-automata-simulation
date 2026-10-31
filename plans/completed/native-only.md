# Remove browser delivery and retain native desktop use

- Owner: Codex, this chat, including serialized verification.
- Status: complete, committed/pushed as `778f2c8` (2026-10-31).
- Authorization: user requested removal of WASM/browser-related things, then the
  next task. Preserve historical assessment/evidence and prior implementation plans.

## Scope and decisions

- [x] Trace native startup, WASM entry point, direct dependencies, scripts and
  references. The Docker image only served the generated browser bundle.
- [x] Remove docs/PWA/generated JS/icons, web setup/build scripts, Node server,
  web-serving Dockerfile and its ignore file. No replacement server/container.
- [x] Remove cdylib output, WASM entry point, native-main cfg guard, WASM target
  dependencies, unused direct getrandom/js, and direct instant/web features.
  Use standard-library Instant; preserve the native App export/package identity.
- [x] Remove WASM check/ignore entries and update current instructions/README.
- [x] Complete native validation and record graph changes.
- [x] Prepare independent cleanup commit/push; F09 follows in its own plan.

## Findings and limitations

F01/F02/F10/F11/F12 are retired for the current application by feature removal,
not by repairing the historical browser graph/server. The unchanged historical
report retains their evidence. egui/eframe may retain transitive cross-platform
packages in Cargo's graph; no dependency migration is needed to remove our web
entry point, delivery and hosting. Native release opt-level remains 2.

## Validation and recovery

Use installed Rust 1.95.0, build jobs 2 / test threads 1, process/worktree inventory
and shared lock. Build the native binary and run the existing 27 regressions in
debug/release, native check, doc tests and formatting. No UI/performance claim.
Cargo may prune the ignored lock for the changed manifest; record actual graph
changes without upgrading the remaining dependencies. Recover by reverting this
bounded cleanup commit alone; preserve historical files and other changes.

## Outcomes

Native build and all 27 regression tests pass in both debug and release.
Project metadata lists a normal library and native binary only, with no cdylib.
The direct browser/WASM surface is removed, including the vulnerable Node server;
F01/F02/F10/F11/F12 are retired for this native-only scope. This does not repair
or claim compatibility for the historical web pipeline. All four historical
report/evidence files are byte-for-byte unchanged.

The updated ignored Cargo.lock has 325 packages, SHA-256
`1253e4b644402f3ae38bc0193886c1995e7d148384ff9dd8d69ac887fd7e3098`.
Compared with the previous validation graph, removed console_error_panic_hook
0.1.7, tracing-wasm 0.2.1, tracing-subscriber 0.3.23, sharded-slab 0.1.7 and
thread_local 1.1.10. No remaining package version/checksum changed. instant and
getrandom remain transitive dependencies of the existing framework/native stack;
they are no longer direct application dependencies. No upgrades/installations.

Sequential validation under the shared lock, Rust 1.95.0 / build jobs 2:

| Exact command | Result |
| --- | --- |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 check --workspace --all-targets` | PASS. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 build --bin vaja_2` | PASS, native binary links. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1` | 27 passed; 0 binary tests. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --release --lib -- --test-threads=1` | 27 passed. |
| `CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --doc -- --test-threads=1` | PASS, 0 doctests. |
| `cargo +1.95.0 fmt --all -- --check` | PASS. |
| `bash -n check.sh` | PASS. |
| `cargo +1.95.0 metadata --no-deps --format-version=1` | Native bin + ordinary library; no cdylib. |

Active source/configuration reference scan finds no remaining direct web entry,
WASM check, web build/server reference or external Instant/getrandom use. No UI
launch/performance claim. Six existing strict Clippy diagnostics remain; compare
again after F09 rather than changing unrelated lints. Verification lock released;
commit identity is recorded in the subsequent status update/history.
