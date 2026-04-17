import fs from "fs";
import path from "path";
import os from "os";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function setOutput(key, value) {
	if (process.env.GITHUB_OUTPUT) {
		fs.appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${value}${os.EOL}`);
	} else {
		console.log(`[Output] ${key}=${value}`);
	}
}

const args = process.argv.slice(2);
const partialsDir = args[0];

if (!partialsDir) {
	console.error("Usage: node scripts/merge-updater-json.js <partials-dir>");
	process.exit(1);
}

try {
	const platforms = {};
	let version = null;

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

		if (version === null) {
			version = partial.version;
		} else if (partial.version !== version) {
			console.error(
				`Version mismatch: expected ${version}, got ${partial.version} in ${jsonPath}`,
			);
			process.exit(1);
		}

		platforms[partial.platformKey] = partial.platformData;
		console.log(`Merged platform: ${partial.platformKey}`);
	}

	if (Object.keys(platforms).length === 0) {
		console.error("No platform partials found");
		process.exit(1);
	}

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

	const updateData = {
		version: `v${version}`,
		notes,
		pub_date: new Date().toISOString(),
		platforms,
	};

	fs.writeFileSync("latest.json", JSON.stringify(updateData, null, 2));
	setOutput("version", version);
	console.log(
		`Generated latest.json with platforms: ${Object.keys(platforms).join(", ")}`,
	);
} catch (error) {
	console.error("Error merging updater JSON:", error);
	process.exit(1);
}
