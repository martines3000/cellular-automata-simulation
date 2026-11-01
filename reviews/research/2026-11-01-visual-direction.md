# Visual direction for the native sandbox

Researched 2026-11-01. Proposals below are design judgments, not implemented features or measured performance claims. Parent agent owns implementation and all verification. No assets, dependencies, benchmarks or GPU experiments were added by this research.

## Direction: a luminous material laboratory

Make the world the main attraction: crisp material silhouettes inside a dark mineral canvas, surrounded by a quiet, readable instrument panel. Aim for a polished pixel sandbox rather than desktop utility windows. Nolla's first-party [Noita page](https://noitagame.com/) and [press kit](https://noitagame.com/press/) provide relevant material-driven game references; use the contrast and atmosphere as inspiration, without copying assets, branding or its combat UI. A dark background and restrained warm/cool material contrast are our proposal, not a claim about Noita's implementation.

Observed checkout: two independently positioned Menu/Display windows, generic Toggle labels, a white canvas, named-color cells and renderer merging of equal adjacent rectangles. Water and supported wood can have fractional geometry. The existing simulation should remain the visual source of truth; decoration must not invent physical state.

Suggested palette (starting values; native contrast/clarity review still needed):

| Role | Color | Use |
| --- | --- | --- |
| Canvas | `#080F19` | Empty space; no animated wallpaper |
| Panel | `#111C28` | Solid backgrounds behind controls |
| Raised surface | `#1B2B3A` | Material cards and settings groups |
| Primary text | `#E7EEF2` | Labels and values |
| Secondary text | `#A9BAC9` | Supporting descriptions |
| Mint accent | `#76E2C1` | Play action, selected outline, focus |
| Warm accent | `#F0BE78` | Paused state and gentle emphasis |
| Destructive | `#F29B90` | Clear/reset text, separated from Play |

Use opaque panels, generous spacing, 15–16 point body text and explicit selected outlines. Material swatches should match the rendered materials; accompany every swatch with a name. Avoid color alone to communicate Running/Paused or the selected material. Microsoft recommends sufficient contrast and solid backing for text over variable game scenes. [Xbox contrast guidance](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/102)

## UI hierarchy to implement first

1. **Header:** short game title, conspicuous Play/Pause and readable Running/Paused state; simulation rate nearby. Keep Clear away from transport.
2. **Left palette:** labeled material cards; selected item has outline and a short behavior hint. Call empty painting **Eraser**. Show the current brush state instead of making selected materials look disabled.
3. **Central canvas:** fill the available area; no separate movable Display window. Draw a cell-aligned brush outline only when the pointer is over editable space. Keep input hit testing limited to the canvas.
4. **Secondary settings:** separate View (zoom, pan, fractional surfaces) from World (size, Generate terrain, Clear, Smooth, generation threshold). Label destructive resize **Resize and clear** and explain its consequence. Keep paused-only restrictions and tell players why a control is disabled.
5. **Footer:** concise current-tool/interaction hint; replace raw engine terms such as Shift x/y with Pan X/Y. “Fractional surfaces” should explain water fill heights and floating wood, matching current `use_shift` behavior.

At narrow or short window sizes, allow the settings/palette to scroll rather than clipping controls; preserve a useful canvas. Keep keyboard focus, visible labels and native control behavior. Later keyboard shortcuts should respect egui text/numeric input focus. Do not advertise shortcuts or brush sizes that have not been implemented.

## Feasible now on egui 0.17

Version-pinned primary sources confirm `SidePanel`, `TopBottomPanel`, `CentralPanel`, style spacing/text styles, `Visuals`, `Context::set_style`, `set_visuals`, `request_repaint` and `animate_bool_with_time`. This supports the layout/theme without a framework upgrade. [egui 0.17 overview](https://docs.rs/crate/egui/0.17.0), [0.17 style source](https://raw.githubusercontent.com/emilk/egui/0.17.0/egui/src/style.rs), [0.17 context source](https://raw.githubusercontent.com/emilk/egui/0.17.0/egui/src/context.rs)

Start with normal styled egui widgets and simple painted swatches. Custom fonts and textures are unnecessary for the first pass. If renderer color snapshots must stay unchanged, apply a bounded, uniform material tint only at app presentation; update visual tests deliberately if production colors change. Per-cell random color noise defeats existing rectangle merging and should wait for an alternative renderer.

## Optional polish, in order

| Stage | Proposed improvement | Guardrail |
| --- | --- | --- |
| 1 | Short hover/selection transitions (roughly 100–160 ms), brush outline, clear state changes | Finite transitions; return to idle without perpetual repaint; provide reduced motion if motion expands |
| 2 | Distinct restrained material palette, subtle static canvas frame, selected-material information | Keep solid/powder/liquid silhouettes readable; no decorative grid at every zoom level |
| 3 | Low intensity fire/lava emission, water surface highlight and soft smoke presentation | Optional presentation effects; never change mass/lifetime or collision behavior; cap extra geometry |
| 4 | A small authored scene/preset gallery showing material interactions | Separate feature with deterministic fixture tests; useful content before cinematic effects |
| 5 | Offscreen emissive layer with limited bloom and configurable quality | Requires renderer integration and profiling; skip blur, chromatic aberration, camera shake and film grain by default |

Motion should answer an input or explain a state, not compete with the simulation. Avoid pulsing the whole canvas, high contrast strobing and rapid red flashes. Microsoft's guidance favors being able to stop distracting animation and avoiding photosensitivity triggers. These are design constraints, not a claim this application has passed accessibility certification. [Motion guidance](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/117), [photosensitivity guidance](https://learn.microsoft.com/en-us/xbox/accessibility/xbox-accessibility-guidelines/118)

## Renderer path: separate presentation from physics

Keep the current merged rectangles for the first UI pass. Measure tick, shape construction, tessellation and interactive frame work separately before choosing a replacement; follow [the existing performance policy](../../PERFORMANCE.md) and [prior research](2026-10-31-performance-tools.md). Timed measurements remain deferred until an exclusive quiet slot is agreed.

If geometry/tessellation is the bottleneck, prototype a CPU-generated color texture drawn as a quad, preserving nearest sampling, clipping, zoom/pan and fractional surfaces. One texel per cell cannot encode the existing partial water height by itself. A lower-resolution emissive texture could later add glow without several translucent rectangles per lit cell. Texture upload bandwidth, retained buffers and visual equivalence need measurement; no speedup is assumed. The pinned `Context` source above verifies texture-loading support, not a ready-made postprocessing pipeline.

Only consider GPU simulation after CPU tick measurements justify the migration. Current `eframe` presentation already rasterizes on the GPU; GPU compute is a separate architectural change. The wgpu project's examples demonstrate render-to-texture, storage textures and combined compute/render workflows, but its development examples are not drop-in APIs for egui 0.17. [Official wgpu examples](https://github.com/gfx-rs/wgpu/blob/trunk/examples/README.md)

A future compute prototype must resolve neighboring writes, preserve phase barriers and explicit transfer budgets, and compare against the CPU oracle. Keep simulation and visual state separate; decorative embers or glow must never be counted as cells. A renderer/compute migration, new materials and art polish should be separate reviewable changes.

## Review checklist

Check the native UI at its default size and a smaller window; verify palette scrolling, readable labels and selection, disabled-action explanations, no painting through controls, pause/edit/resize behavior, zoom/pan/fractional surfaces and a settled paused frame that returns to idle. Test against both nearly empty and dense bright material scenes. Native screenshots and interaction checks are still the implementation owner's responsibility; this note establishes no functional, frame-rate or GPU-performance result.
