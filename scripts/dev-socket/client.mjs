#!/usr/bin/env node
// Minimal client for Rai Pal's dev commands (see backend/tauri-app/src/dev_commands.rs).
// Sends one JavaScript expression to the running dev app over the user socket and
// prints the result.
//
// The user socket uses a dynamic port, so we discover it by probing the known port
// range for the `/check` endpoint. Then we issue `GET /dev/eval?code=...`.
//
// Usage:
//   npm run dev-socket -- "document.title"
//   echo 'document.body.innerText' | npm run dev-socket

import { request } from "node:http";

const USER_SOCKET_PORT_RANGE_START = 43950;
const USER_SOCKET_PORT_RANGE_END = 43960;
const USER_SOCKET_PHRASE = "RAI PAL";

const js =
	process.argv.slice(2).join(" ").trim() ||
	(await new Promise((resolve) => {
		let input = "";
		process.stdin.setEncoding("utf8");
		process.stdin.on("data", (chunk) => (input += chunk));
		process.stdin.on("end", () => resolve(input.trim()));
	}));

if (!js) {
	console.error('Usage: npm run dev-socket -- "<javascript>"');
	process.exit(1);
}

function httpGet(port, path) {
	return new Promise((resolve, reject) => {
		const req = request(
			{ host: "127.0.0.1", port, path, method: "GET" },
			(res) => {
				let body = "";
				res.setEncoding("utf8");
				res.on("data", (chunk) => (body += chunk));
				res.on("end", () => resolve({ status: res.statusCode, body }));
			},
		);
		req.on("error", reject);
		req.end();
	});
}

async function findUserSocketPort() {
	for (
		let port = USER_SOCKET_PORT_RANGE_START;
		port <= USER_SOCKET_PORT_RANGE_END;
		port++
	) {
		try {
			const res = await httpGet(port, "/check");
			if (res.status === 200 && res.body === USER_SOCKET_PHRASE) return port;
		} catch {
			// Port not open; keep probing.
		}
	}
	return null;
}

const port = await findUserSocketPort();
if (!port) {
	console.error(
		"No Rai Pal user socket found on 127.0.0.1 — is the app running in dev mode? (npm run dev)",
	);
	process.exit(1);
}

const res = await httpGet(port, `/dev/eval?code=${encodeURIComponent(js)}`);

try {
	const data = JSON.parse(res.body);
	if (data.ok) {
		console.log(
			typeof data.value === "string"
				? data.value
				: JSON.stringify(data.value, null, 2),
		);
		process.exit(0);
	}
	console.error(data.error);
	process.exit(1);
} catch {
	console.error(`Unexpected response (${res.status}): ${res.body}`);
	process.exit(1);
}
