import type { MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
	defaultRadius: "md",
	primaryColor: "violet",
	scale: 0.9,
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
