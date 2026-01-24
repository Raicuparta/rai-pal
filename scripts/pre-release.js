import fs from "fs";
import path from "path";
import os from "os";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function setOutput(key, value) {
	// In GitHub Actions, we write to the file specified in GITHUB_OUTPUT.
	// When running locally, we just print to console.
	if (process.env.GITHUB_OUTPUT) {
		fs.appendFileSync(process.env.GITHUB_OUTPUT, `${key}=${value}${os.EOL}`);
	} else {
		console.log(`[Output] ${key}=${value}`);
	}
}

try {
	const packageJsonPath = path.join(__dirname, "..", "package.json");
	const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
	const version = packageJson.version;

	console.log(`Detected version: ${version}`);
	setOutput("version", version);

	const changelogPath = path.join(
		__dirname,
		"..",
		"changelogs",
		`v${version}.md`,
	);
	let changelog = "";

	if (fs.existsSync(changelogPath)) {
		changelog = fs.readFileSync(changelogPath, "utf8").trim();
	} else {
		console.warn(`No changelog found at ${changelogPath}`);
		changelog = `No changelog file found for version v${version}`;
	}

	const releaseBody = `${changelog}

![Download for Windows](https://shields.io/badge/-Download_Rai_Pal_for_Windows-8A2BE2?style=for-the-badge&logo=windows&logoColor=white)
![Download for Linux](https://shields.io/badge/-Download_Rai_Pal_for_Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)
`;

	const bodyPath = path.join(__dirname, "..", "temp_release_body.md");
	fs.writeFileSync(bodyPath, releaseBody);
	setOutput("body_path", bodyPath);
} catch (error) {
	console.error("Error in pre-release script:", error);
	process.exit(1);
}
