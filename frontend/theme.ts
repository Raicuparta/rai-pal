import { type MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
	defaultRadius: "md",
	primaryColor: "violet",
	colors: {
		dark: [
			"#C1C2C5",
			"#A6A7AB",
			"#909296",
			"#5c5f66",
			"#373A40",
			"#2C2E33",
			"#25262b",
			"#1A1B1E",
			"#141517",
			"#101113",
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
		Tooltip: {
			defaultProps: {
				openDelay: 200,
			},
		},
	},
};
