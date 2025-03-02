import { deDe } from "./de-de";
import { enUs } from "./en-us";
import { esEs } from "./es-es";
import { jaJp } from "./ja-jp";
import { koKr } from "./ko-kr";
import { ptPt } from "./pt-pt";
import { zhCn } from "./zh-cn";

export const translations = {
	en: enUs,
	pt: ptPt,
	de: deDe,
	ja: jaJp,
	ko: koKr,
	es: esEs,
	zh: zhCn,
};

export const isLanguageCode = (
	code: string | null | undefined,
): code is LanguageCode => {
	if (!code) return false;

	return code in translations;
};

export type LanguageCode = keyof typeof translations;
