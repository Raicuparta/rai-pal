import { fixupConfigRules, includeIgnoreFile } from "@eslint/compat";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";
import tsParser from "@typescript-eslint/parser";
import path from "node:path";
import { fileURLToPath } from "node:url";
import js from "@eslint/js";
import { FlatCompat } from "@eslint/eslintrc";
import reactCompiler from "eslint-plugin-react-compiler";
import reactHooks from "eslint-plugin-react-hooks";
import { defineConfig, globalIgnores } from "eslint/config";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const compat = new FlatCompat({
	baseDirectory: __dirname,
	recommendedConfig: js.configs.recommended,
	allConfig: js.configs.all,
});

export default defineConfig([
	globalIgnores(["**/frontend/api/", "backend/"]),
	includeIgnoreFile(fileURLToPath(new URL(".gitignore", import.meta.url))),
	...fixupConfigRules(
		compat.extends(
			"eslint:recommended",
			"plugin:react/recommended",
			"plugin:react/jsx-runtime",
			"plugin:@typescript-eslint/recommended",
			"prettier",
		),
	),
	{
		files: ["**/*.ts", "**/*.tsx"],
	},
	reactHooks.configs["recommended-latest"],
	{
		plugins: {
			"react-refresh": reactRefresh,
			"react-compiler": reactCompiler,
		},

		languageOptions: {
			globals: {
				...globals.browser,
			},

			parser: tsParser,
			ecmaVersion: "latest",
			sourceType: "module",
		},

		settings: {
			react: {
				version: "detect",
			},
		},

		rules: {
			"react/prop-types": "off",
			"react/prefer-read-only-props": "warn",
			"react/jsx-curly-brace-presence": "warn",
			"react-compiler/react-compiler": "error",
		},
	},
]);
