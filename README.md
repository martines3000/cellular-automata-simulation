# Strata · Element Sandbox

- Cave generation
- Simulation of various materials
  - Water
  - Sand
  - Wood
  - Fire

## Run natively

Install Rust with rustup and the platform's graphics/linker dependencies. The
repository pins Rust 1.95.0 (including Clippy/rustfmt) and commits Cargo.lock;
Cargo uses this toolchain automatically. On macOS install Xcode Command Line Tools.
On Ubuntu 24.04 install:

```sh
sudo apt-get install pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libssl-dev libgl1-mesa-dev
```

Run the desktop application:

```sh
cargo run --locked --release --bin vaja_2
```

Run all native checks and debug/release regressions sequentially:

```sh
bash check.sh
```

Verification runs locally with `bash check.sh`; there are no GitHub Actions
workflows. Graphical interaction was checked on macOS. Dependency updates should
deliberately refresh Cargo.lock and rerun local checks.

The project targets desktop use. The historical assessment is preserved in
[review-report.html](review-report.html); current findings and verification are
tracked in [plans/STATUS.md](plans/STATUS.md).

## Playing

The fixed palette and canvas keep controls separate from the world. Choose Water,
Sand, Dirt, Wood, Fire or Eraser, then click or drag on the canvas. The selected
material has a mint outline and a short behavior description; borders are protected.

Use **Play / Pause** or **Space** to control playback. **Step** advances exactly one
step while paused. Space respects keyboard focus in editable controls. Paused
frames settle without a continuous decorative animation loop.

**Fit world** shows the whole grid and follows window resizing. Adjusting Zoom or
Pan view leaves fit mode. **Fractional surfaces** draws water heights and floating
wood; **Guide grid** adds optional view guides. Both affect presentation only.

## Larger worlds and terrain

While paused, choose 100, 256, 512 or 1024 cells per side and press **Resize and
clear** to create an empty bordered world. Resizing clears all materials, resets pan
and timing, and fits the new world. It preserves the selected material, rate and
generation settings. The toolbox scrolls in shorter windows.

Expand **Terrain & clearing** for Density, **Generate terrain**, **Smooth** and
**Clear world**. These actions require pausing; painting remains available during
playback. Density controls newly generated terrain.

## Simulation timing

The Steps/s slider requests 1–360 simulation steps per second. Fractional elapsed
time carries across frames. Pause/resume and rate changes start a fresh interval.
After a long stall, an update runs at most 16 catch-up steps and drops excess
whole-step debt to keep the UI responsive; achievable speed still depends on the
machine keeping up. Opening or closing toolbox sections does not suspend playback;
use Play / Pause to control it.

## Local profiling

See [PERFORMANCE.md](PERFORMANCE.md) for reproducible larger-world workloads,
benchmark commands and the distinction between CPU and GPU/frame measurements.

## Next performance work

Collect quiet-machine timing comparisons with the supplied workloads, then profile
native frames before changing the renderer or simulation backend.

## Design roadmap

[Material research](reviews/research/2026-11-01-materials.md) defines five proposed
next elements (Ice, Snow, Steam, Lava and Acid) and five later ideas (Stone, Oil,
Glass, Plants and Salt), including quantity/heat prerequisites and regression
contracts. These are proposals, not currently available materials.

[Visual direction](reviews/research/2026-11-01-visual-direction.md) records the
native theme and staged ideas for subtle material effects, presets and optional
emission. Advanced effects and a GPU migration depend on profiling first.

### Info

- This was a small school project for an exercise in a class
- Was built in **less than a week**
- Still new to rust

### New things I learned

- Difference between immediate mode and retained mode UI
- Egui
- A little bit about rust folder structure

![preview](./preview.png)
