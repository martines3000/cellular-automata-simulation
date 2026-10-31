# Rust cellular automata

- Cave generation
- Simulation of various materials
  - Water
  - Sand
  - Wood
  - Fire

## Run natively

With Rust and the platform's graphics/linker dependencies installed:

```sh
cargo run --release --bin vaja_2
```

Run the headless regression suite with:

```sh
CARGO_BUILD_JOBS=2 cargo +1.95.0 test --workspace --all-targets --all-features -- --test-threads=1
```

The project targets desktop use. The historical assessment is preserved in
[review-report.html](review-report.html); current findings and verification are
tracked in [plans/STATUS.md](plans/STATUS.md).

## Simulation timing

The Steps/s slider requests 1–360 simulation steps per second. Fractional elapsed
time carries across frames. Pause/resume and rate changes start a fresh interval.
After a long stall, an update runs at most 16 catch-up steps and drops excess
whole-step debt to keep the UI responsive; achievable speed still depends on the
machine keeping up.

## Possible improvements

- use Bevy game engine for rendering
- use shaders
- use GPU rendering

### Info

- This was a small school project for an exercise in a class
- Was built in **less than a week**
- Still new to rust

### New things I learned

- Difference between immediate mode and retained mode UI
- Egui
- A little bit about rust folder structure

![preview](./preview.png)
