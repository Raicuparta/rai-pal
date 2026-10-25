---
name: update-changelog
description: "Use when asked to update the changelog."
---

# Update Changelog

Check what the latest release version is. Should be the latest git tag in the `main` branch.

Then, find what the new version is gonna be. This should be the version in `backend/Cargo.toml`. The new version should be higher than the previous one.

If any of these things don't line up, notify the user before proceeding.

Look at the `changelogs` folder. It might already have a file for the new version. If it doesn't create it. Otherwise edit the existing one.

Look at a few of the most recent existing changelogs to understand the style.

To create or update the new changelog, look at the commits that happened since the previous version.

## Writing the bullets

The changelog is a simple flat bullet list meant for end users. It is not a commit log. Expect roughly 10-15 bullets, and keep each one short.

- Start bullets with a plain past-tense verb: `Added`, `Changed`, `Fixed`, `Improved`. No headings or sections.
- Group related commits into one broad user-facing item instead of listing every sub-feature. If several commits all contributed to one redesigned screen or flow, one bullet is enough. Mentioning the reason is good: "Changed how mods are displayed and grouped, since the mod list keeps growing."
- Skip anything the user won't notice: tests, refactors, dependency bumps, translation syncs, internal cleanup, tiny edge-case fixes, and mostly-cosmetic tweaks. When in doubt, leave it out.
- Describe the outcome. "Fixed downloading of recommended mod configs" is better than explaining that installs were aborted when the recommended config fetch failed.
- Prefix platform-specific items: "Linux: Added support for Amazon Games installed through Heroic."
- Use the names people see in the UI ("mods tab", "light mode"). Avoid internal terms ("mod families", "provider").
- Keep the tone casual and human. Short asides and reasons are welcome: "nice for mod devs", "pretty much every error message". Use parentheses for brief clarifications: "Added light mode (defaults to following your system theme)."
- Roughly order by user impact, with smaller fixes toward the end.
