import type { MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
  defaultRadius: "md",
  primaryColor: "violet",
  components: {
    Badge: {
      defaultProps: {
        variant: "light",
        fullWidth: true,
      },
    },
    Button: {
      defaultProps: {
        variant: "default",
      },
    },
  },
};
