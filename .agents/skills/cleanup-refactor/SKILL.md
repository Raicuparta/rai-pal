---
name: cleanup-refactor
description: Use when asked to clean up, simplify, or refactor a long-lived branch before merging it into main (e.g. "clean this up", "refactor before merge", "code review the branch").
---

# Cleanup Refactor

We iterate fast, and the plan often changes mid-flight. Code added early in a branch frequently becomes dead weight once the final strategy lands. This is the pass that removes that weight before the branch merges.

## Scope

Review the whole diff between the current branch and the target branch, plus code the branch already touched. Anything the new implementation makes obsolete is fair game, even outside the diff.

Read the quality rules in `AGENTS.md` first; they outrank anything here.

## What to look for

The bias is always toward **less code**, not more.

- **Obsolete leftovers.** Code that only made sense under the earlier strategy: speculative abstraction layers, guards and verifications for situations the final design rules out, unused options/settings, transitional wrappers, indirection kept around to avoid touching callers.
- **Redundant or doubled representations.** The same fact described in two places, mappings that exist only because of an older boundary, special-cased values where a typed representation now exists.
- **Redundancy by default.** Explicitly setting what the framework or component already does. Explicit overrides that match the default.
- **Plumbing that only exists because of the old shape.** Repeated boilerplate paid on every new addition; if adding one variant or one action means editing many sites, restructure so the compiler catches it and the number of edits is minimal.
- **Naming and structure fallout.** Suffixes, prefixes, and file splits that made sense before the last restructuring and now just add noise. Oversized files that should be split, or pieces that should merge.
- **Weak typing and magic values.** Strings/numbers where a type would do, untyped ids, order-dependent lists, behavior decided by display text.
- **Special cases.** Every `if` that exists to handle one odd value is a candidate for a general rule or a data-driven solution.

## What not to do

- Don't be nitpicky. Prefer a few clear, larger wins over many cosmetic tweaks.
- Don't remove things that are still needed, and don't silence real errors to make code shorter.
- Don't preserve backwards compatibility or public API shape. Nothing here is consumed externally.
- Don't trade simplicity for cleverness, or replace one indirection with another.
- If the right answer isn't clear, present the options instead of guessing.

## Working

- Big refactors are welcome when they end up smaller, simpler, and easier to reason about. Prefer deleting over rewriting, and rewriting over patching around the old design.
- One commit per change, simple commit message, no description.
- Keep a short running list of what you changed, what you deliberately skipped and why, and anything that needs a decision from me. Report that at the end rather than narrating during the work.
- If something looks suspiciously missing or wrong but you're not sure it's a real problem, flag it instead of silently changing behavior.
- Run `scripts/check.sh --full` and let me test before you consider it done.
