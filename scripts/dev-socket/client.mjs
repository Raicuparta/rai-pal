#!/usr/bin/env node
// Minimal client for Rai Pal's dev socket (see backend/tauri-app/src/dev_socket.rs).
// Sends one JavaScript expression to the running dev app and prints the result.
//
// Usage:
//   npm run dev-socket -- "document.title"
//   echo 'document.body.innerText' | npm run dev-socket

import { createConnection } from "node:net";

const port = Number(process.env.RAI_PAL_DEV_SOCKET_PORT || 25899);

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

const socket = createConnection({ host: "127.0.0.1", port });
let buffer = "";
socket.setEncoding("utf8");
socket.on("connect", () => socket.write(`${JSON.stringify({ id: "c", eval: js })}\n`));
socket.on("data", (chunk) => {
	buffer += chunk;
	const newline = buffer.indexOf("\n");
	if (newline < 0) return;
	const { ok, value, error } = JSON.parse(buffer.slice(0, newline));
	if (ok) {
		console.log(typeof value === "string" ? value : JSON.stringify(value, null, 2));
	} else {
		console.error(error);
	}
	process.exit(ok ? 0 : 1);
});
socket.on("end", () => {
	console.error("Dev socket closed before a complete response was received.");
	process.exit(1);
});
socket.on("error", (error) => {
	console.error(
		error.code === "ECONNREFUSED"
			? `No dev socket on 127.0.0.1:${port} — is the app running in dev mode? (npm run dev)`
			: error.message,
	);
	process.exit(1);
});
