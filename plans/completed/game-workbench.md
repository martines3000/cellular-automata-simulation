# Simulation workbench

- Status: complete in the working tree; owner Codex. User authorized continued implementation, local
  measurement, brush/visual redesign and commit/push on the existing fresh branch.
- Baseline: `237ebc5`; historical assessments and dated research remain unchanged.
- Scope: measure current seven-phase engine, improve rendering where measurements
  support it, then brushes/navigation/paused undo and authored experiment scenes.
  Keep native-only delivery, existing material budgets and phase order. No new
  material chemistry, dependency migration or speculative GPU compute rewrite.

## Progress

- [x] Inspect checkout, instructions, current callers and verification inventory.
- [x] Add modern-material and phase benchmark cases; sample current implementation.
- [x] Use results to choose renderer work and document GPU decision.
- [x] Add adjustable continuous brushes, canvas navigation and bounded paused undo.
- [x] Add authored labs and restrained material presentation.
- [x] Run targeted/full local checks, native inspection, comparison and handoff.

## Decisions and validation

Root Codex owns all verification; shared directory lock in STATUS, jobs 2 and
one test thread. Exact commands/results go here and in compact dated evidence.
User's latest request supersedes the earlier timing deferral. Record background
load and repeat spread; headless CPU timings cannot establish GPU time or FPS.
Retain original benchmark distribution; new frozen/gas workloads get separate names.

## Recovery

Preserve unrelated work. Inspect diffs before reverting only owned hunks/files;
never reset entire shared files or use git clean. Release only this owner's lock
and stop only this chat's GUI processes. Keep targets and raw profiling logs out
of commits. No historical evidence edits. Plan completion records actual commits.

## Outcome and evidence

Added 1–63-cell round/square continuous brushes, once-per-stroke placement,
sparse paused undo (20 strokes / 32 MiB retained allocated record capacity),
pointer-centered zoom, secondary/middle pan and negative-shift handling. Existing
one-cell operations feed both brushes and test-local adapters; physics unchanged.

Measured seven phases plus new frozen/gas fixtures at four scales. Fast overview
uses a smoothed visible-cell texture below 3 points/cell; fractional/close views
use solid batched quads, eliminating native rectangle seams. Optional bounded,
static material accents preserve idle behavior. Added CPU-only frame diagnostics,
Floating Ice / Steam Chamber / Snowfall labs, and a populated paused start scene.

[Exact samples, commands, before/after evidence and limits](../../reviews/evidence/2026-11-01-workbench.md).
Full check.sh passes strict Clippy/fmt/native check/build, 118 debug/release tests,
2 ignored helpers, 0 doc tests. Final untimed smoke passes 76 cases; two final
matched renderer runs pass. Native inspection complete; test app stopped. Historical
evidence and dependency graph unchanged. Commit recorded in STATUS after creation.

Next: actual native GPU/frame/upload profiling, then an equivalence-tested CPU
optimization or bounded GPU compute experiment. No compute backend or thermal
materials were added; those need separate correctness/performance contracts.
