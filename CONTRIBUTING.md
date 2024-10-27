# Contributing

Before starting any of this: please make an issue about your potential contribution and discuss it with me (unless we already discussed it of course).

Rai Pal is currently undergoing a major rewrite, so this might be outdated. Message me if you need help.

## Setting up

Rai Pal is made with [Tauri v2](https://v2.tauri.app/). You'll need to follow the instructions in the [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/) page to install all the dependencies, including [Node section](https://v2.tauri.app/start/prerequisites/#nodejs). Ignore anything related to mobile.

Once that's taken care of:
- Clone this repo.
- `cd` into the root folder of the project.
- Run the usual `npm install` to install all dependencies
- Run `npm run dev` to start the local dev server and frontend. First time will take a while.

## Project Structure

The project is split into two main parts:

[`/backend`](/backend): Rust code that does most of the heavy lifting.
[`/frontend`](/frontend): TypeScript React code that handles drawing the actual app views, and some of the logic too.

## Backend

[`/backend/Cargo.toml`](/backend/Cargo.toml) is the [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) manifest. So it's Rust settings and stuff that relates to all crates within the workspace. Those crates are:

- [`/backend/tauri-app`](/backend/tauri-app): Anything specific to Tauri, like all the app commands and events that can be called.
- [`/backend/core`](/backend/core): Main Rai Pal logic. Shouldn't depend on anything Tauri-specific, could theoretically be split out into an isolated crate to be used by different frontends. The crate in `tauri-app` will reference this crate, to actually do the Rai Pal magic.
- [`/backend/proc-macros`](/backend/proc-macros): Procedural macros ([read more](https://doc.rust-lang.org/reference/procedural-macros.html)).

Dependencies that are used by multiple crates in the workspace get defined at the workspace level, on the root `Cargo.toml`. Dependencies that are only used by a specific create, get defined in that crate's `Cargo.toml`.

The root/workspace `Cargo.toml` also contains the settings for profiles and linters. Set up your IDE (probably [VSCode](https://code.visualstudio.com/)) to automatically format your code using [Clippy](https://doc.rust-lang.org/clippy/).

## Frontend

I initially tried to keep all business logic on the backend, but eventually settled on a balance that prevented the Rust side from getting too confusing. For instance, the logic for merging different types of data (like installed games, owned games, and remote games) is on the frontend, because I went insane trying to reconcile all that on the backend without making the frontend wait for the entire thing to be processed. So there are a lot of situations where you'll need to decide whether some piece of logic should be on the frontend or on the backend. I usually go by how annoying the Rust code can get: if it gets pretty annoying, I make it dumber and move some logic to the frontend. And of course, anything performance-intensive should be on the Rust side.

## General architecture

The way I fetch data from the backend might be a bit uncommon. I admit that I didn't really look up similar projects to figure out how they did it, I was just winging it. I did learn a lot from the [Outer Wilds Mod Manager](https://github.com/ow-mods/ow-mod-man), but Rai Pal has some different requirements. What I want from Rai Pal is:
- The frontend shouldn't visibly block while fetching or computing data.
- Data should be visible on the frontend as soon as possible.
- Slow data processes shouldn't prevent other processes from delivering their data to the frontend.

Some people have 800 games installed at the same time (not even exaggerating). So parsing through that entire list takes a long time. Ideally, the user shouldn't need to wait for the process of discovering installed games to finish before starting to find the list of owned (but not necessarily installed) games. Same goes for the other way around. Some people own 23k games on Steam (again, not exaggerating, an actual number). Parsing that list takes a long time. So we need to make sure one process doesn't block the other, and that we show the results progressively (games show as they're discovered).

There's also a lot of caching involved, but I'm still trying to figure out the best implementation.

There's probably a lot more confusing stuff going on in this project, but it's hard for me to go over it all. Please to message me if you want to contribute and need more help unravelling this god forsaken thing.
