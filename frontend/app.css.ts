import { globalStyle } from "@vanilla-extract/css";
import { cssVars } from "./theme";

globalStyle("svg", {
  fontSize: "1.5em",
});

// At the time of writing this, wry has a bug where the scrollbar is not visible on Linux (probably a webkit2gtk thing).
// These scrollbar styles are a workaround that make the scrollbar visible on those systems.
// The bug will probably only be fixed in Tauri 2.0
// If this project has been updated to Tauri 2.0, please remove this and test it to see if the issue is fixed.
globalStyle("*::-webkit-scrollbar", {
  width: cssVars.spacing.md,
});

globalStyle("*::-webkit-scrollbar-track", {
  background: cssVars.colors.dark[8],
  borderTopRightRadius: cssVars.radius.md,
  borderBottomRightRadius: cssVars.radius.md,
});

globalStyle("*::-webkit-scrollbar-thumb", {
  borderWidth: 3,
  borderColor: cssVars.colors.dark[8],
  borderStyle: "solid",
  background: cssVars.colors.dark[2],
  borderRadius: cssVars.radius.md,
});

globalStyle("*::-webkit-scrollbar-thumb:hover", {
  background: cssVars.colors.dark[1],
});
