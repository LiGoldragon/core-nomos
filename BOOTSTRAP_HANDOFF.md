# Bootstrap worktree handoff

This repository's `bootstrap` branch/worktree is intentionally retained while the
Protos engine catches up with `main`.

- **Worktree:** `/home/li/wt/github.com/LiGoldragon/core-nomos/bootstrap`
- **Branch and current revision:** the pushed `bootstrap` bookmark
- **Purpose:** isolated Nomos/bootstrap changes, including the NameTable/emission
  boundary that keeps string operations out of the typed schema-to-logos transform
- **Dependency boundary:** do not absorb the separate machinery-pin or broad
  consumer re-pin cascade here; coordinate moving machinery pins through their
  owning claims

Before this worktree is merged or otherwise concluded, run a cross-examination of
`main` and `bootstrap`: surface their relevant diffs, retain good bootstrap changes,
and reject stale or harmful ones deliberately. Conclude it as `Merged` only after
its work is an ancestor of `main`; do not label the active branch `Rejected` merely
to close a lane.
