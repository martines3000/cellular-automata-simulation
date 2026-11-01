# Strata · Element Sandbox

A 2D cellular-automata sandbox in Rust and egui: draw materials and watch them fall,
flow, burn, freeze and turn into steam.

## Project history

**2022: school project.** Built in less than a week while learning Rust and egui.
The original version had cave generation, sand, water, wood, fire and a browser build.

**2026: revisited with OpenAI Codex.** I guided the work; Codex audited the code,
fixed simulation bugs and added 118 regression tests and local benchmarks. It is
now a native desktop playground with Ice, Snow and Steam, worlds up to 1024 × 1024,
a redesigned interface, adjustable brushes, undo, zoom/pan and three experiment labs.
Memory and rendering were improved; GPU simulation remains future work.

## Run it

Install [Rust](https://rustup.rs/) and your platform's graphics dependencies;
[setup details](docs/guide.md#run-natively). The toolchain and dependencies are pinned.

```sh
cargo run --locked --release --bin vaja_2
```

Use **Space** to play/pause. Choose a material and drag to draw, or select an
experiment and press **Load lab** while paused. Run local checks with `bash check.sh`.

[Controls and material rules](docs/guide.md) · [Performance measurements](PERFORMANCE.md) ·
[Original assessment](review-report.html) · [Progress](plans/STATUS.md)
