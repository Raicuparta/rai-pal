# Instructions for AI agents

Rai Pal is a Slint desktop app. The Rust workspace is at the repository root:
`crates/backend` contains shared logic, `crates/frontend` contains the UI and application
runtime, and `crates/proc-macros` contains shared derive macros.

## Finding things

Avoid system-wide searches when you're looking for stuff. If the user is likely
to know where something is, ask first before starting a long task.

## Formatting

Ignore warnings, clippy, and formatting until the implementation is confirmed
correct. Run `scripts/format.sh` before committing so formatting churn is part
of the same commit.

## Running the app

- `scripts/dev.sh` — Slint app with the MCP server enabled.
- `scripts/dev-live.sh` — Slint live preview (no bundled translations).
- `cargo check -p rai-pal-frontend` — compile-check the app.
- Backend logs go to stdout/stderr.

Packaging scripts live in `scripts/`; see `scripts/README.md`.

## Styles

Use `crates/frontend/ui/theme.slint` for reusable spacing, sizes, radii, font sizes,
and semantic colors. If some style is only used in one place, it probably
shouldn't be in the theme.

Prefer content-driven sizing. Let layouts and children determine dimensions
implicitly; use padding, spacing, alignment, and stretch instead of explicit
sizes, preferred-size forwarding, or duplicating child dimensions in wrappers.
Pin sizes only for deliberate constraints.

## Slint

Read `.agents/skills/slint/SKILL.md` and the relevant
`.agents/skills/slint/reference/*.md` before changing `.slint` files.

- Prefer native Slint elements/widgets over hand-rolled equivalents:
  `TabWidget`, `ContextMenuArea` + `Menu`/`MenuItem`, `PopupWindow` for non-menu
  overlays, and built-in layouts instead of manual `x`/`y` positioning.
- Use `preferred-width`/`preferred-height` only for window sizing; don't
  forward child preferred sizes through wrappers.
- Model shared state and cross-cutting actions as exported globals instead of
  forwarding callbacks through every layer. UI-local state stays local.
- Use typed structs/enums and `Theme` constants instead of magic strings,
  indexes, and numbers. Keep persisted JSON field names/serde representations
  compatible with existing app data.
- Use `in`/`out`/`in-out` deliberately. A callback that only reads or only
  writes state should not be routed through a component that does neither.
- Give custom interactive components `accessible-role` and `accessible-label`
  (and default actions where applicable).
- Mark user-visible strings with `@tr`, and use `{}` placeholders instead of
  string concatenation so translations can reorder arguments.
- Reserve space for hover-only controls and track hover across the whole row so
  layouts don't shift or flicker.
- Compile-check with `cargo check -p rai-pal-frontend`; for visual/runtime checks
  use the Slint MCP server or `slint-viewer`, not only source inspection.
- Avoid putting wrapped `Text` or layouts that size to wrapped content inside
  `if`. Slint 1.18 gives conditional elements their unconstrained
  `preferred-height`, so they under- or over-report their height and cause
  overlaps. If you need a conditional, add `height: self.preferred-height;` to
  the conditional `Text`/layout to force width-aware sizing.
- Slint 1.18's default `horizontal-stretch` is 1 (not 0), and a layout's
  stretch is the sum of its children's. In a row with one expanding column and
  one fixed-content column, explicitly set `horizontal-stretch: 0` on the fixed
  column, otherwise it steals the extra space and squeezes the expanding one.

## Shared code

Put logic that is not specific to the UI in `rai-pal-backend` instead of
duplicating it in `crates/frontend`. Be careful not to introduce extra lock contention
or make hot paths synchronous.

Translations are gettext catalogs in `crates/frontend/translations`; they are the
source of truth for user-visible strings.

## Error handling

Errors use thiserror enums in `crates/backend/src/result.rs`. Errors reach the UI mostly
as strings. `fs-err` already adds useful context; for dependency `io::Error`s and
similar, add context with `thiserror-context` whenever the error can reach the
user.

## Tests

Don't add tests unless asked. If a change warrants tests (especially near
existing tests), ask first.
