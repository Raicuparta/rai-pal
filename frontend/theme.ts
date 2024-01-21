import type { MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
	defaultRadius: "md",
	primaryColor: "violet",
	scale: 0.9,
	colors: {
		dark: [
			"#c2c1c5",
			"#a7a6ab",
			"#909096",
			"#5d5c66",
			"#383740",
			"#2d2c33",
			"#26252b",
			"#1b1a1e",
			"#141417",
			"#111013",
		],
	},
	components: {
		Badge: {
			defaultProps: {
				variant: "light",
			},
		},
		Button: {
			defaultProps: {
				variant: "default",
				size: "xs",
			},
		},
		Input: {
			defaultProps: {
				size: "xs",
			},
		},
		Stack: {
			defaultProps: {
				gap: "xs",
			},
		},
		Group: {
			defaultProps: {
				gap: "xs",
			},
		},
		Notification: {
			defaultProps: {
				withBorder: true,
			},
		},
	},
};
