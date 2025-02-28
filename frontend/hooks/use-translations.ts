import { enUs } from "./en-us";

// en-us is the only language defined in TS, used as the source of truth.
// Other language defined in JSON, modifiable at runtime.
export const enUs = {
	hello: "Hello, {name}!",
	goodbye: "Goodbye, {otherName}, {pipi}!",
	simple: "Simple",
} as const;

type EnUsTranslation = typeof enUs;
type TranslationKey = keyof EnUsTranslation;

// Takes a translated string with {parameters}, returns a Record<[Parameter], string>.
// Example: ExtractParams<"Hello, {name}!"> => { name: string }
type ExtractParams<TTranslatedString extends string> =
	TTranslatedString extends `${string}{${infer TParameters}}${infer TRest}`
		? { [K in TParameters | keyof ExtractParams<TRest>]: string }
		: undefined;

// Takes a TranslationKey, returns the args to be passed to translation functions.
// Uses en-us language as the source of truth.
// Empty array if no parameters are needed, so that we can spread them.
// Example: TranslationArgs<"hello"> => [{ name: string }]
// Example: TranslationArgs<"simple"> => []
type TranslationArgs<TKey extends TranslationKey> =
	ExtractParams<EnUsTranslation[TKey]> extends undefined
		? []
		: [params: ExtractParams<EnUsTranslation[TKey]>];

// Translation files are flat objects with string values, parsed from JSON.
function isTranslationValid(
	language: unknown,
): language is Record<string, string> {
	return (
		typeof language === "object" &&
		language !== null &&
		!Array.isArray(language) &&
		Object.values(language).every((value) => typeof value === "string")
	);
}

export function getTranslation<TKey extends TranslationKey>(
	language: unknown,
	key: TKey,
	...args: TranslationArgs<TKey>
): string {
	if (!isTranslationValid(language)) {
		console.error("Invalid language object, must be a Record<string, string>");
		return `{${key}}`;
	}

	const params = args[0];
	let translation = language[key];

	if (translation === undefined) {
		console.error(`Missing translation for key: ${key}`);
		return `{${key}}`;
	}

	if (params) {
		for (const [param, value] of Object.entries(params)) {
			translation = translation.replace(`{${param}}`, value);
		}
	}
	return translation;
}

const frFr = {};

getTranslation(enUs, "goodbye", { otherName: "John", pipi: "pipi" });
getTranslation(frFr, "hello", { name: "Fr" });
getTranslation(frFr, "simple");
