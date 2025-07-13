import { deDe } from "./de-de";
import { enUs } from "./en-us";
import { esEs } from "./es-es";
import { jaJp } from "./ja-jp";
import { koKr } from "./ko-kr";
import { ptPt } from "./pt-pt";
import { zhCn } from "./zh-cn";
import { frFr } from "./fr-fr";
import { AppLocale } from "@api/bindings";

// The keys for this object need to match the language codes returned by tauri-plugin-os.
// By language codes I mean the first part of the BCP47 locale string. Example for en-US: "en".
export const localizations: Record<AppLocale, Localization> = {
	EnUs: enUs,
	PtPt: ptPt,
	DeDe: deDe,
	EsEs: esEs,
	FrFr: frFr,
	JaJp: jaJp,
	KoKr: koKr,
	ZhCn: zhCn,
	WaWa: enUs,
};

// en-US is used as the source of truth for localization format.
export type BaseLocalization = typeof enUs;

// Categories are the root-level keys of the localization objects.
export type LocalizationCategory = keyof BaseLocalization;

// Keys are the leafs. Localization objects don't go any deeper than that.
// The `& string` part is a bit of a hack because TS was having trouble realizing these can only be strings.
export type LocalizationKey<TCategory extends LocalizationCategory> =
	keyof BaseLocalization[TCategory] & string;

// A string of this type must have the same {parameter}s as the string type in TBaseText.
// If TBaseText has no parameters, any string is valid.
// This does not unfortunately validate whether unknown parameters are in the string.
type ParametrizedString<TBaseText> =
	TBaseText extends `${string}${LocalizationParameter<infer Key>}${infer Rest}`
		? `${string}${LocalizationParameter<Key>}${string}` &
				ParametrizedString<Rest>
		: string;

// How an individual parameter is defined in localization strings.
export type LocalizationParameter<TParameter extends string> =
	`{${TParameter}}`;

// This is the type to give to any non-base localizations.
// This will validate that all localizations have the right keys and parameters.
export type Localization = {
	[category in LocalizationCategory]: {
		[localizationKey in keyof BaseLocalization[category]]: ParametrizedString<
			BaseLocalization[category][localizationKey]
		>;
	};
};

export function getNativeLocaleName(locale: AppLocale): string {
	if (locale === "WaWa") {
		return "Wario (Waah!)";
	}

	return localizations[locale].meta.nativeName;
}
