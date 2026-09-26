# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

- Wine not being found on NixOS?
- Native linux Unreal Games usually come with a .sh launcher, which rai pal currently doesn't detect. need a whole new flow for that I guess.
- MaybeWinePrefix should actually be Maybe?? Seems to be doing it even on native linux.
- Changing the only mod source while a game in the games tab is selected breaks the mod list, have to close and reopen the game.
- Make Rai Pal show an icon and cover art when installed to steam.
- add game manually, remove it, it's still visible but not clickable.
- oh actually add game manually, is immediately selected, but not visible in list until refresh is pressed.
- maybe DOTNET_SYSTEM_GLOBALIZATION_INVARIANT is needed in bepinex linux environment in database?

native linux games bepinex "run" on steam deck:

- games start in background unfocused.
- steam can't seem to kill the game from steam's overlay menu.
- games that work:
  - in between (mono x86)
  - isle of jura (mono x64)
  - the wantering village (mono x64)
  - minami lane (mono x64)
- games that don't start at all via bepinex run button:
  - The Pedestrian (il2cpp x64) shows no sign of starting. this is a bepinex bug actually.
  -

trying to restart steam deck, suck on "waiting for rai pal to shut down"
