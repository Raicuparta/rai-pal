import { deDe } from "./de-de";
import { enUs } from "./en-us";
import { esEs } from "./es-es";
import { jaJp } from "./ja-jp";
import { koKr } from "./ko-kr";
import { ptPt } from "./pt-pt";
import { zhCn } from "./zh-cn";
import { frFr } from "./fr-fr";

// The keys for this object need to match the language codes returned by tauri-plugin-os.
// By language codes I mean the first part of the BCP47 locale string. Example for en-US: "en".
export const languageToTranslation = {
	en: enUs,
	pt: ptPt,
	de: deDe,
	ja: jaJp,
	ko: koKr,
	es: esEs,
	zh: zhCn,
	fr: frFr,
};

export const isLanguageCode = (
	code: string | null | undefined,
): code is LanguageCode => {
	if (!code) return false;

	return code in languageToTranslation;
};

export type LanguageCode = keyof typeof languageToTranslation;

// en-US is used as the source of truth for translation format.
type BaseTranslation = typeof enUs;

// A string of this type must have the same {parameter}s as the string type in TBaseText.
// If TBaseText has no parameters, any string is valid.
// This does not unfortunately validate whether unknown parameters are in the string.
type ParametrizedString<TBaseText> =
  TBaseText extends `${string}{${infer Key}}${infer Rest}`
    ? `${string}{${Key}}${string}` & ParametrizedString<Rest>
    : string;

// This is the type to give to any non-base translations.
// This will validate that all translations have the right keys and parameters.
export type Translation = {
	[category in keyof BaseTranslation]: {
		[translationKey in keyof BaseTranslation[category]]: ParametrizedString<BaseTranslation[category][translationKey]>;
	};
};