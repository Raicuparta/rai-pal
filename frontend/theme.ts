import { type MantineThemeOverride } from "@mantine/core";

export const theme: MantineThemeOverride = {
	defaultRadius: "md",
	primaryColor: "violet",
	fontFamily:
		// Mantine comes with this long-ass default value for fontFamily so that it picks the first good one installed in the system,
		// but it used to include Helvetica. This made it fall back to a weird broken font on Linux,
		// where all text was shifted upwards a bit, so here I just copied that default but with helvetica removed.
		"-apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Arial, sans-serif, Apple Color Emoji, Segoe UI Emoji",
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
				withinPortal: false,
			},
		},
		Menu: {
			defaultProps: {
				shadow: "md",
			},
		},
		Popover: {
			defaultProps: {
				shadow: "md",
			},
		},
	},
};
