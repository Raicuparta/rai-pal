import type { MantineThemeOverride } from "@mantine/core";
import { themeToVars } from "@mantine/vanilla-extract";

export const theme: MantineThemeOverride = {
  defaultRadius: "md",
  primaryColor: "violet",
};

export const cssVars = themeToVars(theme);
