// Tauri v2 messes up the icons in appimage.
// If this issue gets closed, we can remove this script: https://github.com/tauri-apps/tauri/issues/15110

import fs from "fs";
import os from "os";
import path from "path";
import { spawnSync } from "child_process";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const defaultBundleDir = path.join(
	__dirname,
	"..",
	"backend",
	"target",
	"release",
	"bundle",
	"appimage",
);

function fail(message) {
	throw new Error(message);
}

function runOrFail(command, args, options = {}) {
	const result = spawnSync(command, args, {
		stdio: "inherit",
		...options,
	});

	if (result.error) {
		if (result.error.code === "ENOENT") {
			fail(`Command not found: ${command}`);
		}
		fail(`Failed to run ${command}: ${result.error.message}`);
	}

	if (result.status !== 0) {
		fail(`${command} exited with status ${result.status}`);
	}
}

function resolvePluginExecutable(commandOrPath) {
	try {
		if (commandOrPath.includes(path.sep)) {
			fs.chmodSync(commandOrPath, 0o755);
		}
	} catch {
		// Ignore chmod failures; the probe commands below will validate executability.
	}

	const pluginTypeResult = spawnSync(commandOrPath, ["--plugin-type"], {
		stdio: "ignore",
	});
	if (!pluginTypeResult.error && pluginTypeResult.status === 0) {
		return {
			command: commandOrPath,
			prefixArgs: [],
		};
	}

	const appImageResult = spawnSync(
		commandOrPath,
		["--appimage-extract-and-run", "--plugin-type"],
		{ stdio: "ignore" },
	);
	if (!appImageResult.error && appImageResult.status === 0) {
		return {
			command: commandOrPath,
			prefixArgs: ["--appimage-extract-and-run"],
		};
	}

	return null;
}

function findPluginInTauriCache(candidateNames) {
	const tauriHome = process.env.TAURI_HOME;
	const xdgCacheHome = process.env.XDG_CACHE_HOME;
	const homeDir = os.homedir();

	const searchDirs = [
		tauriHome,
		xdgCacheHome ? path.join(xdgCacheHome, "tauri") : null,
		homeDir ? path.join(homeDir, ".cache", "tauri") : null,
		homeDir ? path.join(homeDir, ".local", "share", "tauri") : null,
	].filter((dirPath) => dirPath && pathExists(dirPath));

	for (const searchDir of searchDirs) {
		for (const candidateName of candidateNames) {
			const candidatePath = path.join(searchDir, candidateName);
			if (!pathExists(candidatePath)) {
				continue;
			}

			const resolved = resolvePluginExecutable(candidatePath);
			if (resolved) {
				console.log(`Using linuxdeploy appimage plugin from: ${candidatePath}`);
				return resolved;
			}
		}
	}

	return null;
}

function findLinuxDeployPlugin() {
	if (process.platform !== "linux") {
		fail("linuxdeploy appimage plugin is only supported on Linux");
	}

	const pathResolved = resolvePluginExecutable("linuxdeploy-plugin-appimage");
	if (pathResolved) {
		return pathResolved;
	}

	const cacheResolved = findPluginInTauriCache([
		"linuxdeploy-plugin-appimage.AppImage",
		"linuxdeploy-plugin-appimage",
	]);
	if (cacheResolved) {
		return cacheResolved;
	}

	fail(
		"linuxdeploy appimage plugin was not found in PATH or Tauri cache directories",
	);
}

function selectBuiltAppImage(tempRoot) {
	const candidates = fs
		.readdirSync(tempRoot)
		.filter((entry) => entry.endsWith(".AppImage"))
		.map((entry) => path.join(tempRoot, entry));

	if (candidates.length === 0) {
		fail("linuxdeploy did not produce an AppImage output");
	}

	if (candidates.length === 1) {
		return candidates[0];
	}

	const sorted = candidates.sort((left, right) => {
		const leftMtime = fs.statSync(left).mtimeMs;
		const rightMtime = fs.statSync(right).mtimeMs;
		return rightMtime - leftMtime;
	});

	return sorted[0];
}

function getAppImagePaths(inputPath) {
	if (inputPath) {
		if (!fs.existsSync(inputPath)) {
			fail(`AppImage not found: ${inputPath}`);
		}
		return [path.resolve(inputPath)];
	}

	if (!fs.existsSync(defaultBundleDir)) {
		fail(`Bundle directory not found: ${defaultBundleDir}`);
	}

	return fs
		.readdirSync(defaultBundleDir)
		.filter((fileName) => fileName.endsWith(".AppImage"))
		.map((fileName) => path.join(defaultBundleDir, fileName));
}

function pathExists(pathToCheck) {
	try {
		fs.lstatSync(pathToCheck);
		return true;
	} catch {
		return false;
	}
}

function toRelativeSymlinkLink(rootPath, productName) {
	const dirIconPath = path.join(rootPath, ".DirIcon");
	const rootDesktopPath = path.join(rootPath, `${productName}.desktop`);

	if (pathExists(dirIconPath)) {
		fs.rmSync(dirIconPath, { force: true });
	}
	if (pathExists(rootDesktopPath)) {
		fs.rmSync(rootDesktopPath, { force: true });
	}

	fs.symlinkSync(`${productName}.png`, dirIconPath);
	fs.symlinkSync(
		`usr/share/applications/${productName}.desktop`,
		rootDesktopPath,
	);

	const dirIconTarget = fs.readlinkSync(dirIconPath);
	const desktopTarget = fs.readlinkSync(rootDesktopPath);
	if (path.isAbsolute(dirIconTarget) || path.isAbsolute(desktopTarget)) {
		fail("Post-process failed: symlink target is still absolute");
	}
}

function detectProductName(rootPath) {
	const rootEntries = fs.readdirSync(rootPath);
	const desktopEntry = rootEntries.find((entry) => {
		if (!entry.endsWith(".desktop")) {
			return false;
		}
		const fullPath = path.join(rootPath, entry);
		return fs.lstatSync(fullPath).isSymbolicLink();
	});

	if (desktopEntry) {
		return path.basename(desktopEntry, ".desktop");
	}

	const appDirDesktopPath = path.join(rootPath, "usr", "share", "applications");
	if (!fs.existsSync(appDirDesktopPath)) {
		fail("Could not find desktop entries in extracted AppImage");
	}

	const appDirDesktopEntry = fs
		.readdirSync(appDirDesktopPath)
		.find((entry) => entry.endsWith(".desktop"));
	if (!appDirDesktopEntry) {
		fail("Could not detect product name from desktop entry");
	}

	return path.basename(appDirDesktopEntry, ".desktop");
}

function assertFileIsWorldExecutable(filePath) {
	const mode = fs.statSync(filePath).mode & 0o777;
	if ((mode & 0o001) === 0) {
		fail(
			`Post-process failed: ${path.basename(filePath)} is not world-executable (mode ${mode.toString(8)})`,
		);
	}
}

function normalizeLaunchPermissions(rootPath) {
	const appRunPath = path.join(rootPath, "AppRun");
	if (!pathExists(appRunPath)) {
		fail("Could not find AppRun in extracted AppImage");
	}

	fs.chmodSync(appRunPath, 0o755);

	const wrappedPath = path.join(rootPath, "AppRun.wrapped");
	if (pathExists(wrappedPath)) {
		fs.chmodSync(wrappedPath, 0o755);
	}
}

function verifyLaunchPermissions(appImagePath, tempRoot) {
	const verifyRoot = fs.mkdtempSync(path.join(tempRoot, "verify-"));
	const verifyExtractedPath = path.join(verifyRoot, "squashfs-root");

	try {
		runOrFail(appImagePath, ["--appimage-extract"], {
			cwd: verifyRoot,
		});

		const appRunPath = path.join(verifyExtractedPath, "AppRun");
		if (!pathExists(appRunPath)) {
			fail("Verification failed: AppRun missing after repack");
		}

		assertFileIsWorldExecutable(appRunPath);

		const wrappedPath = path.join(verifyExtractedPath, "AppRun.wrapped");
		if (pathExists(wrappedPath)) {
			assertFileIsWorldExecutable(wrappedPath);
		}
	} finally {
		fs.rmSync(verifyRoot, { recursive: true, force: true });
	}
}

function patchAppImage(appImagePath, pluginTool) {
	console.log(`Patching AppImage: ${appImagePath}`);

	const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "appimage-fix-"));
	const extractedPath = path.join(tempRoot, "squashfs-root");
	const rebuiltPath = path.join(tempRoot, path.basename(appImagePath));

	try {
		runOrFail(appImagePath, ["--appimage-extract"], {
			cwd: tempRoot,
		});

		if (!fs.existsSync(extractedPath)) {
			fail("AppImage extraction failed: squashfs-root was not created");
		}

		const productName = detectProductName(extractedPath);
		toRelativeSymlinkLink(extractedPath, productName);
		normalizeLaunchPermissions(extractedPath);

		runOrFail(
			pluginTool.command,
			[...pluginTool.prefixArgs, "--appdir", "squashfs-root"],
			{
				cwd: tempRoot,
				env: {
					...process.env,
					ARCH: process.env.ARCH ?? "x86_64",
				},
			},
		);

		const generatedPath = selectBuiltAppImage(tempRoot);
		fs.copyFileSync(generatedPath, rebuiltPath);
		verifyLaunchPermissions(rebuiltPath, tempRoot);

		fs.copyFileSync(rebuiltPath, appImagePath);
		fs.chmodSync(appImagePath, 0o755);
		console.log(`Patched AppImage in place: ${appImagePath}`);
	} finally {
		fs.rmSync(tempRoot, { recursive: true, force: true });
	}
}

export function fixAppImageSymlinks(inputPath) {
	const appImagePaths = getAppImagePaths(inputPath);

	if (appImagePaths.length === 0) {
		fail(`No AppImage files found in ${defaultBundleDir}`);
	}

	const pluginTool = findLinuxDeployPlugin();

	for (const appImagePath of appImagePaths) {
		patchAppImage(appImagePath, pluginTool);
	}

	console.log(`Patched ${appImagePaths.length} AppImage file(s).`);
}

function isDirectCliInvocation() {
	if (!process.argv[1]) {
		return false;
	}

	try {
		return fs.realpathSync(process.argv[1]) === fs.realpathSync(__filename);
	} catch {
		return path.resolve(process.argv[1]) === path.resolve(__filename);
	}
}

const isCliInvocation = isDirectCliInvocation();

if (isCliInvocation) {
	try {
		const args = process.argv.slice(2);
		fixAppImageSymlinks(args[0]);
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error);
		console.error(message);
		process.exit(1);
	}
}
