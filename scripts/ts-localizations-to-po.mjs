import fs from "node:fs";
import path from "node:path";

const sourceDir = "frontend/localizations";
const outputDir = "backend/slint-app/translations";

const locales = {
	EnUs: "en",
	DeDe: "de",
	EsEs: "es",
	FrFr: "fr",
	JaJp: "ja",
	KoKr: "ko",
	PtPt: "pt",
	ZhCn: "zh",
	WaWa: "wa",
};

const files = {
	EnUs: "en-us.ts",
	DeDe: "de-de.ts",
	EsEs: "es-es.ts",
	FrFr: "fr-fr.ts",
	JaJp: "ja-jp.ts",
	KoKr: "ko-kr.ts",
	PtPt: "pt-pt.ts",
	ZhCn: "zh-cn.ts",
	WaWa: "en-us.ts",
};

function readLocalization(file) {
	const content = fs.readFileSync(path.join(sourceDir, file), "utf8");
	const match =
		content.match(/=\s*(\{[\s\S]*\})\s*as const\s*;\s*$/) ??
		content.match(/=\s*(\{[\s\S]*\});\s*$/);

	if (!match) {
		throw new Error(`Failed to parse ${file}`);
	}

	return Function(`return (${match[1]})`)();
}

function getParameters(text) {
	return [...text.matchAll(/\{([a-zA-Z0-9_]+)\}/g)].map((match) => match[1]);
}

function toSlintPlaceholders(text, parameterOrder) {
	return text.replace(/\{([a-zA-Z0-9_]+)\}/g, (_, name) => {
		const index = parameterOrder.indexOf(name);
		if (index < 0) {
			throw new Error(`Unknown placeholder {${name}} in "${text}"`);
		}
		return `{${index}}`;
	});
}

function doFunnyWario(text) {
	const parts = text.split(" ");
	const output = [];

	for (let i = 0; i < parts.length; i++) {
		const part = parts[i];
		if (!part) continue;

		const isParameter = part.includes("{") || part.includes("}");
		if (isParameter || part.length < 3) {
			output.push(part);
			continue;
		}

		const start = i === 0 ? "W" : "w";
		const end = i < parts.length - 1 ? "h" : "h!";
		const middle = Array.from({ length: part.length - 2 }, () => "a").join("");
		output.push(`${start}${middle}${end}`);
	}

	return output.join(" ");
}

function escapePo(text) {
	return text
		.replaceAll("\\", "\\\\")
		.replaceAll('"', '\\"')
		.replaceAll("\n", "\\n")
		.replaceAll("\t", "\\t");
}

function writePo(language, entries) {
	const dir = path.join(outputDir, language, "LC_MESSAGES");
	fs.mkdirSync(dir, { recursive: true });

	const lines = [
		'msgid ""',
		'msgstr ""',
		`"Content-Type: text/plain; charset=UTF-8\\n"`,
		`"Content-Transfer-Encoding: 8bit\\n"`,
		`"Language: ${language}\\n"`,
		`"Plural-Forms: nplurals=2; plural=(n != 1);\\n"`,
	];

	for (const entry of entries) {
		lines.push(
			"",
			`msgctxt "${escapePo(entry.context)}"`,
			`msgid "${escapePo(entry.source)}"`,
			`msgstr "${escapePo(entry.translation)}"`,
		);
	}

	lines.push("");
	fs.writeFileSync(
		path.join(dir, "rai-pal-slint.po"),
		`${lines.join("\n")}\n`,
	);
}

const english = readLocalization(files.EnUs);

for (const [locale, language] of Object.entries(locales)) {
	const localization = locale === "WaWa" ? english : readLocalization(files[locale]);
	const entries = [];

	for (const context of Object.keys(english)) {
		for (const key of Object.keys(english[context])) {
			const source = english[context][key];
			const parameterOrder = getParameters(source);
			const translated = localization[context]?.[key];

			if (translated === undefined) {
				continue;
			}

			let translation = locale === "WaWa" ? doFunnyWario(translated) : translated;

			if (locale === "WaWa" && context === "meta" && key === "nativeName") {
				translation = "Wario (Waah!)";
			}

			entries.push({
				context,
				source: toSlintPlaceholders(source, parameterOrder),
				translation: toSlintPlaceholders(translation, parameterOrder),
			});
		}
	}

	writePo(language, entries);
	console.log(`Wrote ${language}: ${entries.length} entries`);
}
