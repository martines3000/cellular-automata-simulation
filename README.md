# Strata · Element Sandbox

- Cave generation
- Simulation of various materials
  - Water
  - Sand
  - Wood
  - Fire
  - Ice
  - Snow
  - Steam

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
Sand, Dirt, Wood, Fire, Ice, Snow, Steam or Eraser, then click or drag on the canvas. The
selected material has a mint outline and a short behavior description; borders are protected.

Use **Play / Pause** or **Space** to control playback. **Step** advances exactly one
step while paused. Space respects keyboard focus in editable controls. Paused
frames settle without a continuous decorative animation loop.

**Fit world** shows the whole grid and follows window resizing. Adjusting Zoom or
Pan view leaves fit mode. **Fractional surfaces** draws water heights and floating
wood; **Guide grid** adds optional view guides. Both affect presentation only.

## Ice, Snow and water conservation

**Ice** falls through empty space, rises one cell at a time through water, and
rests on the surface. **Snow** falls and slides into piles; contact with Water
melts it. Fire melts adjacent Ice and Snow. These are scripted game interactions,
without a temperature or energy solver.

The **Freeze** tool turns existing Water (including passive water residue) into
Ice. **Melt** returns Ice/Snow to Water. Neither tool changes the stored quantity;
repeat applications do not add material. Melted amounts below the active-water
threshold remain passive water in the empty cell, preserving that residue. The
existing small-Water cleanup still has its explicitly tested loss budget.

Painting places 10 water units per Ice cell and 1 per Snow cell; repainting replaces
the cell rather than accumulating frozen quantity. Erasing/replacing a frozen cell
removes its contents deliberately. Select a material again to leave Freeze/Melt.
Tools work while paused or playing and protect the outer border.

## Steam and condensation

**Steam** rises through air or water and drifts diagonally or sideways around
obstacles. Water displaced by a bubble keeps its quantity. Trapped Steam has no
lifetime and never evaporates from the simulation, even at tiny quantities.

Fire vaporizes adjacent liquid Water. Steam condenses in place beside Ice/Snow;
the cold solid is unchanged. The **Vaporize** and **Condense** tools perform the
same conserved water/Steam transitions directly. Condensed amounts below the
active-water threshold remain passive water; nothing is lost by conversion.
Painting Steam places 1 water unit and replaces the old contents, while erasing
removes them. Select a material to leave a phase tool.

These are scripted interactions without heat/energy, gas expansion or pressure.
Dirt ceilings do not consume Steam; it does not displace powders or Wood. During
one fire phase, newly melted Ice/Snow remains liquid rather than being immediately
vaporized by another neighboring fire. The existing phases keep their relative
order, followed by frozen materials and then Steam. New condensate flows during
the next water phase.

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
contracts. Ice/Snow/Steam and scripted phase tools are now implemented; Lava/Acid
remain proposals and require additional transport/reaction state.

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
