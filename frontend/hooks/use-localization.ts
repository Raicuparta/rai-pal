import {
	BaseLocalization,
	LocalizationCategory,
	LocalizationKey,
	LocalizationParameter,
	localizations,
} from "@localizations/localizations";
import { AppLocale } from "@api/bindings";
import { atom, useAtom } from "jotai";
import { useEffect, useCallback } from "react";
import { locale } from "@tauri-apps/plugin-os";
import { useAppSettings } from "./use-app-settings";

// Takes a localized string with {parameters}, returns a Record<[Parameter], string>.
// Example: ExtractParams<"Hello, {name}!"> => { name: string }
type LocalizationParamsObject<TLocalizedString> =
	TLocalizedString extends `${string}${LocalizationParameter<infer TParameters>}${infer TRest}`
		? { [K in TParameters | keyof LocalizationParamsObject<TRest>]: string }
		: undefined;

// Takes a Category and a Key, returns the args to be passed to localization functions.
// Empty array if no parameters are needed, so that we can spread them and make them optional.
// The `& string` part is a bit of a hack because TS was having trouble realizing these can only be strings.
type LocalizationFunctionArgs<TLocalizedString> =
	LocalizationParamsObject<TLocalizedString> extends infer Params extends object
		? [params: Params]
		: [];

// Localization files are objects with two levels: categories at the root, and key-value pairs at the second level.
function isLocalizationValid(
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

// Stupid joke that makes all text become waah.
function doFunnyWario(localization: string): string {
	const wahTextParts: string[] = [];
	const textParts = localization.split(" ");
	for (let i = 0; i < textParts.length; i++) {
		const isParam = textParts[i].includes("{") || textParts[i].includes("}");

		if (isParam || textParts[i].length < 3) {
			wahTextParts.push(textParts[i]);
			continue;
		}

		const start = i === 0 ? "W" : "w";
		const end = i < textParts.length - 1 ? "h" : "h!";
		const middle = Array.from(
			{ length: textParts[i].length - 2 },
			() => "a",
		).join("");

		wahTextParts.push(`${start}${middle}${end}`);
	}

	return wahTextParts.join(" ");
}

function getLocalizedText<
	TCategory extends LocalizationCategory,
	TKey extends LocalizationKey<TCategory>,
>(
	language: unknown,
	category: TCategory,
	key: TKey,
	isWario: boolean,
	...args: LocalizationFunctionArgs<BaseLocalization[TCategory][TKey]>
): string {
	if (!isLocalizationValid(language)) {
		console.error("Invalid language object, must be a Record<string, string>");
		return `{${key}}`;
	}

	const params = args[0];
	let localization = language[category][key];

	if (localization === undefined) {
		console.error(`Missing localization for key: ${key}`);
		return `{${key}}`;
	}

	if (isWario) {
		localization = doFunnyWario(localization);
	}

	if (params) {
		for (const [param, value] of Object.entries(params)) {
			localization = localization.replace(`{${param}}`, value);
		}
	}

	return localization;
}

export const detectedLocaleAtom = atom<AppLocale | null>(null);

// At the time of writing this comment, Rai Pal only supports one language per locale.
// So en-GB would map to EnUs, pt-BR to PtPt, etc.
// If we ever need to use more specific localizations, we'll need to change this logic.
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

export function useLocalization<TCategory extends LocalizationCategory>(
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
						`Couldn't find a localization for the language code ${languageCode}, extracted from locale string ${localeString}`,
					);
					return;
				}

				setDetectedLocale(appLocale);
			});
		}
	}, [detectedLocale, setDetectedLocale]);

	return useCallback(
		<TKey extends LocalizationKey<TCategory>>(
			key?: TKey,
			...args: LocalizationFunctionArgs<BaseLocalization[TCategory][TKey]>
		) => {
			if (!key) return undefined;

			return getLocalizedText(
				localizations[effectiveLocale],
				category,
				key,
				effectiveLocale === "WaWa",
				...args,
			);
		},
		[effectiveLocale, category],
	);
}
