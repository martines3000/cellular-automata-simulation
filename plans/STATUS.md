# Implementation status

Updated: 2026-10-30. Historical baseline: master at `1792c2f`.
Assessment: [review-report.html](../review-report.html#F04).

## Verification coordination

- Owner: Codex, this F04 chat; sole editor of `src/app/world/mod.rs`.
- Active job: none; final verification complete. The competing benchmark exited
  before checks began. All verification was serialized; shared lock released.
- Shared lock for cooperating workers:
  `/tmp/cellular-automata-simulation-verification.lock` (directory; owner recorded
  inside). Inspect process inventory before acquiring; never remove another owner's
  lock or stop their processes. Run one verification command at a time.
- Settings used: installed Rust 1.95.0, build jobs 2, test threads 1.

## Finding resolutions and backlog

| Priority | Finding | Status / next bounded task |
| --- | --- | --- |
| Resolved locally | F04 water cleanup | Implemented and verified in working tree; uncommitted. [Completed plan and exact commands](completed/f04-water-cleanup.md). Debug/release: 7/7 each; native checks/tests/format pass; six unchanged baseline Clippy diagnostics. |
| Before public serving | F10 path traversal | Open; repair or replace server before exposing it. |
| Next simulation fix | F03 fire indexing | Open; fix signed neighbor indexing with debug/release regression. |
| Then | F05/F06 editing state | Open; preserve scalars during Smooth and initialize generation state. |
| Then | F07/F08 displacement/direction | Open; conserve displaced water before changing directional rules. |
| Separate web task | F01/F02/F11/F12 | Open; choose reproducible graph/toolchain and paired artifacts. |
| Later | F09 timing; F13–F16 performance/interaction | Open; after relevant correctness baseline. |

Only F04 is authorized for implementation here. All other findings retain their
historical status. Resolution details belong in the linked completed plan.
