import { style } from "@vanilla-extract/css";
import { cssVars } from "../../theme";

export const tableCardStyle = style({
  padding: 0,
  flex: 1,
});

export const virtuosoStyle = style({
  height: "100%",
});

export const tableStyle = style({
  tableLayout: "fixed",
});

export const tableHeadTrStyle = style({
  background: cssVars.colors.dark[8],
});

export const tableHeadThStyle = style({
  cursor: "pointer",
  ":hover": {
    background: cssVars.colors.dark[7],
  },
});

export const tableRowStyle = style({
  cursor: "pointer",
});
