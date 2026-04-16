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

const version = process.argv[2];

if (!version) {
	console.error("Usage: node scripts/generate-release-body.js <version>");
	process.exit(1);
}

try {
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

	console.log("--- Release Body ---");
	console.log(releaseBody);
	console.log("--------------------");

	const bodyPath = path.join(__dirname, "..", "temp_release_body.md");
	fs.writeFileSync(bodyPath, releaseBody);
	console.log(`Written to: ${bodyPath}`);
	setOutput("body_path", bodyPath);
} catch (error) {
	console.error("Error in pre-release script:", error);
	process.exit(1);
}
