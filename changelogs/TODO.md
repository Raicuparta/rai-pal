# TODO

This file is here for me to track things I wanna fix for the next release. If there are things here on the main branch, I probably forgot to delete them. It should never have todos I only intend to complete in future releases, that's what GitHub issues are for.

- Bigger close button for modals so it's easier to click.
- Patreon leaderboard and page fixes:
  - bronze has same color of silver, needs to be fixed.
  - top three people should have a border around their avatar circle with the color of the trophy (importantly, border shouldn't cover parts of the image, should be outside).
  - everyone else should have an invisible border or padding to make up for the same border space.
  - the patreon leaderboard title should use the section divider (not sure the name) element we usually use, the one that's a line with the title in the middle.
  - The description of that section should be a tooltip instead of being in there. Same as what we did for the filters tooltips, a question mark icon (we already have the same icon in filtrs tooltips, reuse) that when hovered shows this description. put that tooltip icon on the right side of the patreon leaderboard section title, so as if its part of the section divider text, between the lines.
  - The link component used here (and anywhere else, should all be same) uses the primary color, but that looks bad. make it just use the bright text color. hover can just show underline instead of changing color.
- You saw my custom clippy warning to avoid .ok() in rust but instead of fixing it properly you were a dumbass and worked around it by using match or something stupid like that. So now you have to go through the entire code and see if there are any dumb usages of match that were likely just avoiding an .ok(), and fix it properly, for instance by using ok_and_log() (or whatever it's called) or by properly throwing the error forward when it makes sense.
- notifications should be on the bottom right corner. currently they're on the top right.
- confirm that the tauri to slint migration didn't cause any paths to change. important are the build names (should already be good) and the folder names used for configs, downloaded mods, installed mods, etc.
- debug json data should be on-demand. don't include debug json data in structs etc, it should be a request that the frontend does when a debug button is clicked.
- show DEV in window title when in dev mode.
- the reset (x) button of search input should remain visible even when the input is not focused. currently if i click something outside search that button disappears, i need a way to easily reset search.
- issue with "auto" color scheme sometimes changing some things but not others. like changes background but not some text, like filters text. example right now my desktop is in dark mode. if i change rai pal mode to auto, it shows almost everything as if im in light mode (white background etc) but some text (like filters in sidebar) is white, as if i'm dark mode. but for instance the text for name of games in the games list is black, as if white mode. so some text white mode, some text dark mode, backgrouns seem all white mode, even though desktop is in dark mode (from auto change due to time of day). strangely, if i go to kde settings and manually change to light mode, rai pal goes to correct light mode, all colors correct. if i manually change kde to dark mode, it goes back to broken half light mode in rai pal. i know for a fact that this doesn't always happen though, so it's a bit confusing. this could be something weird with kde or my specific config, so unless you're sure there's something wrong in my slint app please don't go too deep here, just tell me your findings.

# Do not do these yet:

- make the single-instance thing ignore dev.
- Filters scrolling still always scrolls entire sidebar.
- debug data currently missing a bunch of data. adding more buttons for game-mods maybe would be good.
