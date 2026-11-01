# Implementation plans

Use `plans/planned/<finding>-<topic>.md` for multi-step fixes, uncertain designs or
work likely to span sessions. Copy [plans/TEMPLATE.md](plans/TEMPLATE.md) and keep
only useful detail. A small, low-risk fix can use a concise checklist in
[plans/STATUS.md](plans/STATUS.md) instead.

Update the plan and status when reproduction is established, a fix is chosen,
targeted tests finish, final verification finishes, or work is interrupted.
Record behavior, evidence, limitations and verification ownership. Link historical
findings rather than reproducing the assessment. Describe uncommitted work as
implemented in the working tree; add a revision only after a real commit.

Move finished or superseded plans to `plans/completed/`, recording the outcome or
replacement link. Keep open plans in `plans/planned/`; its README preserves the
empty directory. Historical review deliverables remain unchanged.

On recovery, inspect status/diffs and the active verification record first.
Preserve unrelated work, including untracked files. Revert only owned hunks after
reviewing the diff; remove only files created by that plan. Never restore entire
shared files blindly or clean the checkout. Release only a lock you own.
