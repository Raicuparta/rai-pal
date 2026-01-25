import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const platform = process.platform;
const args = process.argv.slice(2);
const version = args[0];
const repoOwner = args[1];
const repoName = args[2];

if (!version || !repoOwner || !repoName) {
	console.error(
		"Usage: node scripts/generate-updater-json.js <version> <owner> <repo>",
	);
	process.exit(1);
}

try {
	let bundleDir = "";
	let updaterExt = "";

	if (platform === "win32") {
		bundleDir = path.join(
			__dirname,
			"..",
			"backend",
			"target",
			"release",
			"bundle",
			"nsis",
		);
		updaterExt = ".zip";
	} else if (platform === "linux") {
		bundleDir = path.join(
			__dirname,
			"..",
			"backend",
			"target",
			"release",
			"bundle",
			"appimage",
		);
		updaterExt = ".tar.gz";
	} else {
		console.error(`Unsupported platform for updater generation: ${platform}`);
		process.exit(0);
	}

	if (!fs.existsSync(bundleDir)) {
		console.error(`Bundle directory not found: ${bundleDir}`);
		console.error(
			"If running locally, please create this folder or run a build first.",
		);
		process.exit(1);
	}

	const files = fs.readdirSync(bundleDir);
	const sigFile = files.find((f) => f.endsWith(`${updaterExt}.sig`));

	if (!sigFile) {
		console.error(`No signature file found in ${bundleDir}`);
		process.exit(1);
	}

	const signature = fs
		.readFileSync(path.join(bundleDir, sigFile), "utf8")
		.trim();
	const bundleFilename = sigFile.replace(".sig", "");

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
		notes: notes,
		pub_date: new Date().toISOString(),
		platforms: {
			[platform === "win32" ? "windows-x86_64" : "linux-x86_64"]: {
				signature: signature,
				url: `https://github.com/${repoOwner}/${repoName}/releases/download/v${version}/${bundleFilename}`,
			},
		},
	};

	const outputFilename =
		platform === "win32" ? "latest-windows.json" : "latest-linux.json";
	fs.writeFileSync(outputFilename, JSON.stringify(updateData, null, 2));

	console.log(`Generated ${outputFilename} pointing to ${bundleFilename}`);
} catch (error) {
	console.error("Error generating updater JSON:", error);
	process.exit(1);
}
