import { translations } from "../translations/translations";
import { enUs } from "../translations/en-us";
import { AppLocale } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useEffect, useCallback } from "react";
import { locale } from "@tauri-apps/plugin-os";
import { useAppSettings } from "./use-app-settings";

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

export const detectedLocaleAtom = atom<AppLocale | null>(null);

// At the time of writing this comment, Rai Pal only supports one language per locale.
// So en-GB would map to EnUs, pt-BR to PtPt, etc.
// If we ever need to use more specific translations, we'll need to change this logic.
const languageCodeToAppLocale: Record<string, AppLocale> = {
	en: "EnUs",
	pt: "PtPt",
	de: "DeDe",
	es: "EsEs",
	fr: "FrFr",
	ja: "JaJp",
	ko: "KoKr",
	zh: "ZhCn",
};

export function useGetTranslated<TCategory extends TranslationCategory>(
	category: TCategory,
) {
	const [detectedLocale, setDetectedLocale] = useAtom(detectedLocaleAtom);
	const [appSettings] = useAppSettings();

	const effectiveLocale =
		appSettings.overrideLanguage ?? detectedLocale ?? "EnUs";

	useEffect(() => {
		if (!detectedLocale) {
			locale().then((localeString) => {
				const languageCode = localeString?.split("-")[0];
				if (!languageCode) {
					console.error(
						`Invalid locale '${localeString}', couldn't get a language code out of it`,
					);
					return;
				}

				const appLocale = languageCodeToAppLocale[languageCode];

				if (!appLocale) {
					console.error(
						`Couldn't find a translation for the language code ${languageCode}, extracted from locale string ${localeString}`,
					);
					return;
				}

				setDetectedLocale(appLocale);
			});
		}
	}, [detectedLocale, setDetectedLocale]);

	return useCallback(
		<TKey extends TranslationKey<TCategory>>(
			key?: TKey,
			...args: TranslationArgs<TCategory, TKey>
		) => {
			if (!key) return undefined;

			return getTranslation(
				translations[effectiveLocale],
				category,
				key,
				...args,
			);
		},
		[effectiveLocale, category],
	);
}
