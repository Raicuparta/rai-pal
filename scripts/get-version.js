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

const cargoTomlPath = path.join(__dirname, "..", "backend", "Cargo.toml");
const cargoToml = fs.readFileSync(cargoTomlPath, "utf8");
const versionMatch = cargoToml.match(/^version\s*=\s*"(.+)"/m);
if (!versionMatch) {
	console.error("Could not find version in backend/Cargo.toml");
	process.exit(1);
}

const version = versionMatch[1];
setOutput("version", version);
setOutput("tag_name", `v${version}`);
