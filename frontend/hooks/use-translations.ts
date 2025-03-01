import { enUs } from "./en-us";
import { ptPt } from "./pt-pt";
import { zhCn } from "./zh-cn";

// en-us is the only language defined in TS, used as the source of truth.
// Other language defined in JSON, modifiable at runtime.
type BaseTranslation = typeof enUs;

// Categories are the root-level keys in the translation object.
export type TranslationCategory = keyof BaseTranslation;

// Keys are the leafs. Translation objects don't go any deeper than that.
// The `& string` part is a bit of a hack because TS was having trouble realizing these can only be strings.
export type TranslationKey<TCategory extends TranslationCategory> =
	keyof BaseTranslation[TCategory] & string;

// Takes a translated string with {parameters}, returns a Record<[Parameter], string>.
// Example: ExtractParams<"Hello, {name}!"> => { name: string }
type ExtractParams<TTranslatedString extends string> =
	TTranslatedString extends `${string}{${infer TParameters}}${infer TRest}`
		? { [K in TParameters | keyof ExtractParams<TRest>]: string }
		: undefined;

// Takes a Category and a Key, returns the args to be passed to translation functions.
// Empty array if no parameters are needed, so that we can spread them.
// The `& string` part is a bit of a hack because TS was having trouble realizing these can only be strings.
type TranslationArgs<
	TCategory extends TranslationCategory,
	TKey extends TranslationKey<TCategory>,
> =
	ExtractParams<BaseTranslation[TCategory][TKey] & string> extends undefined
		? []
		: [params: ExtractParams<BaseTranslation[TCategory][TKey] & string>];

// Translation files are objects with two levels: categories at the root, and key-value pairs at the second level.
// Parsed from JSON.
function isTranslationValid(
	language: unknown,
): language is Record<string, Record<string, string>> {
	return (
		typeof language === "object" &&
		language !== null &&
		!Array.isArray(language) &&
		Object.values(language).every((value) => typeof value === "object") &&
		Object.values(language).every((value) => !Array.isArray(value)) &&
		Object.values(language).every((value) =>
			Object.values(value).every((value) => typeof value === "string"),
		)
	);
}

function getTranslation<
	TCategory extends TranslationCategory,
	TKey extends TranslationKey<TCategory>,
>(
	language: unknown,
	category: TCategory,
	key: TKey,
	...args: TranslationArgs<TCategory, TKey>
): string {
	if (!isTranslationValid(language)) {
		console.error("Invalid language object, must be a Record<string, string>");
		return `{${key}}`;
	}

	const params = args[0];
	let translation = language[category][key];

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

export function useGetTranslated<TCategory extends TranslationCategory>(
	category: TCategory,
) {
	return function t<TKey extends TranslationKey<TCategory>>(
		key?: TKey,
		...args: TranslationArgs<TCategory, TKey>
	) {
		if (!key) return undefined;

		return getTranslation(zhCn, category, key, ...args);
	};
}
