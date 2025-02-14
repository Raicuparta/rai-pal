import { defineConfig } from "@pandacss/dev";

export default defineConfig({
	// Whether to use css reset
	preflight: true,

	// Where to look for your css declarations
	include: ["./frontend/**/*.{ts,tsx}"],

	// Files to exclude
	exclude: [],

	// Useful for theme customization
	theme: {
		extend: {
			tokens: {
				colors: {
					dark: {
						100: { value: "#A6A7AB" },
						200: { value: "#909296" },
						300: { value: "#5c5f66" },
						400: { value: "#373A40" },
						500: { value: "#2C2E33" },
						600: { value: "#25262b" },
						700: { value: "#1A1B1E" },
						800: { value: "#141517" },
						900: { value: "#101113" },
					},
				},
			},
		},
	},

	// The output directory for your css system
	outdir: "frontend/styled-system",
});
