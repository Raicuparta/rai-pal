# Instructions for AI agents

Rai Pal is a Slint desktop app. The Rust workspace is at the repository root: `crates/backend` has shared logic, `crates/frontend` has the UI and app runtime, and `crates/proc-macros` has shared derive macros.

## Finding things

Avoid system-wide searches when you're looking for stuff. If the user is likely to know where something is, ask first before starting a long task.

## Commits

When asked to commit anything, use simple commit messages, no description.

## Quality

These are all important:

- Always prefer type-safe solutions.
- Avoid repetition (especially easy mistake to make when something needs to be represented on both Rust side and Slint side, or on multiple Rust crates).
- For enums and similar types, avoid structuring the code in a way that means adding one new element (like one new possible enum value) means changing the code in many places. Type safety should mean the code won't compile until all variants are handled, and the number of required changes per new added variant should be minimal.
- Code must be easy to reason about (splitting large files is good when it makes sense, but gotta avoid adding too much indirection, etc).
- Always pick the simplest solution that follows the points above.

## APIs

None of the crates in this repo are meant for external usage. We don't have to care about backwards compatibility of the Rust/Slint APIs. It's also important to lean towards code simplicity vs making the API more usable for third parties (since they don't exist). One good example to avoid is making lots of getters for structs; we should instead just expose struct properties directly. So avoid anything that complicates the code for reasons that don't make sense for private crates.

## Formatting

Ignore warnings, clippy, and formatting until the implementation is confirmed correct. Run `scripts/check.sh` before committing if asked to commit.

## Running the app

- `scripts/dev.sh` — app with the MCP server enabled.
- `scripts/dev-live.sh` — live preview (no bundled translations).
- `cargo check -p rai-pal-frontend` — compile-check the app.
- Backend logs go to stdout/stderr.

Packaging scripts live in `scripts/`; see `scripts/README.md`.

## UI

Read `.agents/skills/slint/SKILL.md` and the relevant `.agents/skills/slint/reference/*.md` before changing `.slint` files.

Use `crates/frontend/ui/theme.slint` for reusable spacing, sizes, radii, font sizes, and semantic colors. If a style is only used in one place, it probably shouldn't be in the theme.

Prefer content-driven sizing. Let layouts and children set dimensions with padding, spacing, alignment, and stretch instead of explicit sizes or forwarding preferred sizes through wrappers. Pin sizes only for deliberate constraints.

- Use `preferred-width`/`preferred-height` only for window sizing.
- Model shared state and cross-cutting actions as exported globals instead of forwarding callbacks through every layer. UI-local state stays local.
- Use typed structs/enums and `Theme` constants instead of magic strings, indexes, and numbers. Keep persisted JSON field names/serde representations compatible with existing app data.
- Use `in`/`out`/`in-out` deliberately. A callback that only reads or writes state shouldn't be routed through a component that does neither.
- Don't add props just to tune the styling of a component's content, like spacing or padding. Have the parent wrap it instead.
- Give custom interactive components `accessible-role` and `accessible-label` (and default actions where applicable).
- Mark user-visible strings with `@tr`, and use `{}` placeholders instead of string concatenation so translations can reorder arguments. Translations are gettext catalogs in `crates/frontend/translations` and the source of truth for user-visible strings.
- Reserve space for hover-only controls and track hover across the whole row so layouts don't shift or flicker.

For visual/runtime checks, use the Slint MCP server, not only source inspection.

### Layout gotchas

- Avoid putting wrapped `Text` or layouts that size to wrapped content inside `if`. Slint 1.18 gives conditional elements their unconstrained `preferred-height`, so they under- or over-report their height and cause overlaps. If you need a conditional, add `height: self.preferred-height;` to the conditional `Text`/layout to force width-aware sizing.
- Slint 1.18's default `horizontal-stretch` is 1 (not 0), and a layout's stretch is the sum of its children's. In a row with one expanding column and one fixed-content column, set `horizontal-stretch: 0` on the fixed column, or it steals the extra space and squeezes the expanding one.

## Shared code

Put logic that isn't specific to the UI in `rai-pal-backend` instead of duplicating it in `crates/frontend`. Avoid extra lock contention and don't make hot paths synchronous.

## Errors

Errors use thiserror enums in `crates/backend/src/result.rs`. Errors reach the UI mostly as strings. `fs-err` already adds useful context; for dependency `io::Error`s and similar, add context with `thiserror-context` whenever the error can reach the user.

## Tests

Don't add tests unless asked. If a change warrants tests (especially near existing tests), ask first.

## Comments

Never write comments in your code, unless asked to. Rare exception only for extremely weird hacks that can't be understood without extra context, and in those cases the comments musto only explain why, never what. Never remove comments that were already there.

## Markdown

Any time you're writing markdown, never enforce a max width by adding newlines. Let lines be as long as they can except for specific stuff like tables etc.

## Verifying results

By default, don't try to verify results via slow processes like looking at the node tree via MCP, screenshots, etc, unless asked to iterate on many tasks and testing is really necessary. If visual validation is required, prefer asking the user to check, explaining what needs to be verified. The slower verification via MCP and possibly visually via screenshots is acceptable when doing lots of unattended iteration, or when specifically asked to validate by yourself.
