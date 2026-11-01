# Material expansion research — 2026-11-01

Design research only; no material implementation or dependency changes. Read the
current `src/app/world/{mod.rs,cell.rs}` before proposing these additions. The five
next concepts are Ice, Snow, Steam, Lava and Acid; the five later concepts are
Stone, Oil, Glass, Plants and Salt. All behavior below is a game-design proposal
unless explicitly identified as a sourced fact.

## Evidence and design boundary

- **Fact:** ordinary ice is less dense than liquid water and floats. **Proposal:**
  floating ice chunks with pale blue edges and a clear melt interaction, without
  a physically accurate buoyancy solver. [USGS: Water Density](https://www.usgs.gov/water-science-school/science/water-density)
- **Fact:** snow water equivalent measures its meltwater; visual snow depth alone
  does not determine that quantity. **Proposal:** record meltwater quantity rather
  than converting every snow cell into a full water cell. [USACE: Snow Properties](https://www.hec.usace.army.mil/confluence/hmsdocs/hmstrm/snow-accumulation-and-melt/snow-properties)
- **Fact:** evaporation turns liquid water into vapor; condensation reverses it
  and releases heat. **Proposal:** visible, pearl-colored rising Steam represents
  the process for readability; it is distinct from expiring Smoke. Water vapor
  itself is usually invisible. [USGS: Evaporation](https://www.usgs.gov/water-science-school/science/evaporation-and-water-cycle),
  [USGS: Condensation](https://www.usgs.gov/water-science-school/science/condensation-and-water-cycle)
- **Fact:** lava–water interaction can cool lava and generate steam; the outcome
  depends on how contact occurs. **Proposal:** slow orange lava forms dark crust
  and releases bounded Steam on contact; omit explosive pressure initially.
  [USGS: Lava entering the sea](https://www.usgs.gov/news/volcano-watch-what-happens-when-lava-flows-sea)
- **Fact:** acid reacts with calcium carbonate to produce other substances,
  including carbon dioxide; this does not mean every material dissolves in every
  acid. **Proposal:** a fictional lime-green Acid erodes an explicit whitelist and
  consumes a finite corrosion budget. [ACS: Calcium carbonate reaction](https://www.acs.org/education/whatischemistry/adventures-in-chemistry/experiments/fate-of-calcium-carbonate.html)
- **Game precedent:** The Powder Toy combines powders, liquids, gases, heat and
  interactions, and documents seed growth and acid/base reactions. Its scope is
  inspiration, not a recommendation to copy its engine or code. [Official site](https://powdertoy.co.uk/)

## Five next materials

| Order | Material and readable behavior | First bounded interaction | Required contract |
| --- | --- | --- | --- |
| 1 | **Ice:** solid cyan chunks; block flow, float by conservative swaps. | Heat melts exactly the stored water-equivalent quantity into Water. Freeze Water only after capacity/occupancy rules exist. | Melting and freezing cannot create water; displaced water survives; equal temperatures do not oscillate states. |
| 2 | **Snow:** soft white powder; falls and piles, with a lower water equivalent than equal-volume Ice. | Warmth melts it; defer compaction, wind and avalanches. | Falling preserves its quantity; meltwater is its stored quantity, independent of visual cell size. |
| 3 | **Steam:** rising pale gas; gently drifts sideways when blocked. | Heated Water becomes Steam; cooling returns the same water quantity. | Trapped Steam retains quantity; no smoke-style lifetime deletion; blocked condensation stays Steam rather than overwriting another cell. |
| 4 | **Lava:** slow, warm glowing liquid with a darker cooling skin. | Heats neighboring Water/Ice/Snow and cools into existing Dirt as a temporary crust representation. | Record cooled lava quantity separately if Dirt cannot hold it; never store it in Dirt.mass because that scalar already participates in water displacement. Stone later replaces this compromise. |
| 5 | **Acid:** bright corrosive liquid with a small bubbling cue. | Initially erodes interior Dirt only; protect the outer border by coordinate. | Every erosion consumes reagent strength; transfer amount and strength together; explicitly account for removed solid and spent solution. No unlimited corrosion or implicit pH simulation. |

These are separate increments, not five types to add in one patch. Ice/Snow need
an agreed heat trigger first; a bounded heat/cool brush is a smaller starting point
than a full fluid-temperature simulation. Lava should wait for Steam. Acid needs
species-aware liquid transfers and a reaction-product policy before implementation.
If the temporary lava-crust ledger is unacceptable, implement Stone first and move
one of the five concepts to the following increment instead of hiding material loss.

## Representation and scheduling prerequisites

- Today `Cell` is only a type and `mass: f32`: Water uses quantity; Smoke uses
  lifetime; some `None` cells retain passive water. `MAX_MASS = 10` is a preferred
  fluid scale, not a cap; `MIN_MASS = 0.01` permits specific cleanup. No temperature,
  acid concentration or general solid quantity exists. Adding enum variants alone
  would not make transitions correct.
- Agree a typed state design first: water-equivalent quantity for Water/Ice/Snow/
  Steam; separate smoke lifetime; acid solution amount plus reagent strength; lava
  amount plus heat state. Do not reinterpret the existing scalar globally. A side
  grid or explicit payload are alternatives to compare for simplicity and memory;
  neither is selected or implemented here.
- For real thermal exchange, carry energy/enthalpy with moved material, include
  latent heat, and budget external heating/cooling. Averaging temperatures does not
  conserve energy. A cheaper scripted interaction model is acceptable if described
  as such; choose hysteresis/dwell rules to avoid freeze/melt flicker.
- Keep existing Sand → Water → Fire → Smoke → Wood relative ordering and copy
  current data into scratch before each phase. Design new transport/reaction slots
  explicitly; a candidate is a final reaction pass after existing phases with new
  products unable to react again until next tick. New material transport needs its
  own scheduled pass; avoid moving products twice through scan-order effects.
- Preserve incoming transfers and passive water in every displacement. Reactions
  must reserve destinations and consume each input once; deterministic conflict
  rules must handle two sources targeting one cell. No pressure solver or engine
  migration is a prerequisite for the first heat-brush experiments.

## Regression contracts before each material ships

1. Closed fixtures track `W = Water + passive water in None + Ice + Snow + Steam`
   in common units: `W_after = W_before + painted - erased - documented cleanup -
   explicit venting`. Log each deliberate loss; use exact expectations for small
   binary-exact quantities and a tight derived floating-point bound for long runs.
2. Test zero, immediately below/at/above `MIN_MASS`, low fractional quantities,
   values over `MAX_MASS`, simultaneous inflow/outflow, multi-source destinations,
   and transitions with no free neighbor. No incoming quantity may disappear.
3. Test every boundary/corner, supported stacks, confined gas, liquid displacement,
   opposite-direction mirrored fixtures, paused editing/resize/clear, and many
   ticks. Assert finite nonnegative quantities, valid type/state combinations,
   untouched outer Dirt border and unchanged unrelated materials.
4. Thermal fixtures budget internal energy plus heat supplied/removed and latent
   energy; insulated exchange conserves the chosen model. Scripted rules instead
   test explicit trigger/transition counts and make no energy-conservation claim.
5. Acid fixtures track reagent consumed and dissolved-solid products or an explicit
   sink ledger. Splitting/merging cannot amplify reagent; zero-strength solution
   cannot erode; repeated contact cannot consume the same solid twice.
6. Seeded fixture tests cover mixed original/new phases; debug and release must
   agree. Visual animation never changes simulation quantities or advances ticks.

## Five later materials

| Material | Payoff and dependency |
| --- | --- |
| **Stone** | Distinct permanent construction and Lava crust; separate border immunity from material identity, then allow reheating only with an explicit heat budget. |
| **Oil** | Floating amber slicks and burning surfaces; needs immiscible-liquid displacement plus fuel/product budgets. Most oils float, but some are denser than fresh water, so select one fictional light-oil behavior deliberately. [NOAA: Oil in rivers](https://response.restoration.noaa.gov/node/417) |
| **Glass** | Transparent channels and observation chambers; propose heat-fused Sand only after solid quantity/heat accounting, with readable outlines and no optical simulation. |
| **Plants** | Growing green branches create living scenes; consume stored Water and a named growth resource. Defer light/air simulation, and describe any unlimited growth resource as an external source. |
| **Salt** | Dissolving crystals and deposits after evaporation; needs dissolved-solute transport before freezing-point changes, with salt surviving the water cycle. |

Visual treatment is proposal: tiny snow sparkles, Ice fracture lines, Steam wisps,
Lava emissive tint and Acid bubbles should be bounded display effects. Keep each
material recognizable by value/texture as well as hue, offer reduced motion, and
measure cost before introducing per-cell animated geometry or glow passes.
