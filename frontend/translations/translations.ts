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
export const translations = {
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

	return code in translations;
};

export type LanguageCode = keyof typeof translations;

type BaseTranslation = typeof enUs;

type ExtractParams<
	T extends string
> =
	T extends `${string}{${infer Key}}${infer Rest}`
		? [Key, ...ExtractParams<Rest>]
		: []

type StringWithParamsFromTuple<TTuple extends string[]> =
  TTuple extends [infer K extends string, ...infer Rest extends string[]]
    ? `${string}{${K}}${string}` & StringWithParamsFromTuple<Rest>
    : string;

export type Translation = {
	[category in keyof BaseTranslation]: {
		[translationKey in keyof BaseTranslation[category]]: StringWithParamsFromTuple<ExtractParams<BaseTranslation[category][translationKey] & string>>;
	};
};