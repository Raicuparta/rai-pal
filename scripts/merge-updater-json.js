import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const args = process.argv.slice(2);
const version = args[0];
const partialsDir = args[1];

if (!version || !partialsDir) {
	console.error(
		"Usage: node scripts/merge-updater-json.js <version> <partials-dir>",
	);
	process.exit(1);
}

try {
	const changelogPath = path.join(
		__dirname,
		"..",
		"changelogs",
		`v${version}.md`,
	);
	let notes = `Version ${version}`;
	if (fs.existsSync(changelogPath)) {
		notes = fs.readFileSync(changelogPath, "utf8").trim();
	}

	const platforms = {};

	const entries = fs.readdirSync(partialsDir);
	for (const entry of entries) {
		const entryPath = path.join(partialsDir, entry);
		const stat = fs.statSync(entryPath);

		let jsonPath;
		if (stat.isDirectory()) {
			const files = fs.readdirSync(entryPath);
			const jsonFile = files.find((f) => f.endsWith(".json"));
			if (!jsonFile) continue;
			jsonPath = path.join(entryPath, jsonFile);
		} else if (entry.endsWith(".json")) {
			jsonPath = entryPath;
		} else {
			continue;
		}

		const partial = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
		platforms[partial.platformKey] = partial.platformData;
		console.log(`Merged platform: ${partial.platformKey}`);
	}

	if (Object.keys(platforms).length === 0) {
		console.error("No platform partials found");
		process.exit(1);
	}

	const updateData = {
		version: `v${version}`,
		notes,
		pub_date: new Date().toISOString(),
		platforms,
	};

	fs.writeFileSync("latest.json", JSON.stringify(updateData, null, 2));
	console.log(
		`Generated latest.json with platforms: ${Object.keys(platforms).join(", ")}`,
	);
} catch (error) {
	console.error("Error merging updater JSON:", error);
	process.exit(1);
}
