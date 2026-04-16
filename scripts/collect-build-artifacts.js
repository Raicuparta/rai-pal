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
const repoOwner = args[0];
const repoName = args[1];

if (!repoOwner || !repoName) {
	console.error(
		"Usage: node scripts/collect-build-artifacts.js <owner> <repo>",
	);
	process.exit(1);
}

const cargoTomlPath = path.join(__dirname, "..", "backend", "Cargo.toml");
const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const versionMatch = cargoToml.match(/^version\s*=\s*"(.+)"/m);
if (!versionMatch) {
	console.error("Could not find version in backend/Cargo.toml");
	process.exit(1);
}
const version = versionMatch[1];

const platform = process.platform;
const outputDir = path.join(__dirname, "..", "release-assets");
fs.mkdirSync(outputDir, { recursive: true });

try {
	let bundleDir;
	let updaterSigExt;
	let updaterAssetName;
	let executableAssetName;
	let executableExt;

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
		updaterSigExt = ".zip.sig";
		updaterAssetName = "updater-windows.zip";
		executableAssetName = "RaiPal.exe";
		executableExt = ".exe";
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
		updaterSigExt = ".tar.gz.sig";
		updaterAssetName = "updater-linux.tar.gz";
		executableAssetName = "RaiPal.AppImage";
		executableExt = ".AppImage";
	} else {
		console.error(`Unsupported platform: ${platform}`);
		process.exit(1);
	}

	if (!fs.existsSync(bundleDir)) {
		console.error(`Bundle directory not found: ${bundleDir}`);
		process.exit(1);
	}

	const files = fs.readdirSync(bundleDir);

	const sigFile = files.find((f) => f.endsWith(updaterSigExt));
	if (!sigFile) {
		console.error(
			`No signature file (*${updaterSigExt}) found in ${bundleDir}`,
		);
		process.exit(1);
	}

	const signature = fs
		.readFileSync(path.join(bundleDir, sigFile), "utf8")
		.trim();

	const updaterFile = sigFile.replace(".sig", "");
	if (!files.includes(updaterFile)) {
		console.error(`Updater file not found: ${updaterFile}`);
		process.exit(1);
	}

	const executableFile = files.find((f) => f.endsWith(executableExt));
	if (!executableFile) {
		console.error(
			`Executable file (*${executableExt}) not found in ${bundleDir}`,
		);
		process.exit(1);
	}

	fs.copyFileSync(
		path.join(bundleDir, updaterFile),
		path.join(outputDir, updaterAssetName),
	);
	console.log(`Copied ${updaterFile} -> ${updaterAssetName}`);

	fs.copyFileSync(
		path.join(bundleDir, executableFile),
		path.join(outputDir, executableAssetName),
	);
	console.log(`Copied ${executableFile} -> ${executableAssetName}`);

	const platformKey = platform === "win32" ? "windows-x86_64" : "linux-x86_64";
	const partial = {
		version,
		platformKey,
		platformData: {
			signature,
			url: `https://github.com/${repoOwner}/${repoName}/releases/download/v${version}/${updaterAssetName}`,
		},
	};

	const partialFilename =
		platform === "win32"
			? "updater-partial-windows.json"
			: "updater-partial-linux.json";
	fs.writeFileSync(partialFilename, JSON.stringify(partial, null, 2));
	console.log(`Generated ${partialFilename}`);

	setOutput("version", version);
} catch (error) {
	console.error("Error collecting build artifacts:", error);
	process.exit(1);
}
