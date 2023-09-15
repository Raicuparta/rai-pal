import type { MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
  defaultRadius: "md",
  colorScheme: "dark",
  primaryColor: "violet",
  globalStyles: () => ({
    svg: {
      fontSize: "1.5em",
    },
  }),
  components: {
    Button: {
      defaultProps: {},
    },
  },
};
