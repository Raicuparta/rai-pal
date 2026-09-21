# Steam can't close (or focus) games launched by Rai Pal

## Symptom

- Launch a Steam game from Rai Pal (native or Proton). Steam detects it as
  running: the library shows **Resume** instead of **Play**, and pressing
  **Resume** correctly focuses the game.
- Pressing **Exit Game** in the Steam UI (Steam Deck overlay, or the library
  page on desktop) does nothing. It hangs on "exiting game..." while the game
  keeps running.
- On Steam Deck game mode the game is not focused automatically on launch.

## Repro

- Desktop (Linux) also reproduces the close failure, so it doesn't need a Deck.
- Launch a Steam game from Rai Pal, then try to close it from Steam.

## What we know

- The game registers with Steam via SteamAPI, which is why Steam knows it's
  running.
- Rai Pal launches the game itself (it does not ask Steam to launch it).
- Closing Rai Pal no longer kills the game (fixed by launching the game through
  `systemd-run --user`, so it is parented to systemd instead of Rai Pal).
- Giving the game a Steam-style systemd scope (`steam-launch-wrapper --scope`)
  did **not** make Steam able to close or focus it.
- `steam://rungameid/<appid>` and `steam -applaunch <appid>` do **not** focus a
  running game: Steam starts a second instance instead.

## Goal

Make Steam's **Exit Game** work for games Rai Pal launched, and ideally make
game mode focus the game on launch. Ideally without changing the game's Steam
Launch Options (Rai Pal explicitly does not want a wrapper/launch-option
approach).

## Notes for whoever picks this up

- The current launch path is `backend/core/src/game_launch.rs`
  (`GameLaunch`/`spawn_game`), `steam_runtime.rs`, and `steam_proton.rs`.
- Steam's own launches go through `reaper` + `steam-launch-wrapper`; the exact
  mechanism it uses to stop a game is not confirmed. Don't assume; test.
