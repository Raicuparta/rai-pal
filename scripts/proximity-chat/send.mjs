// Sends player position/rotation packets to the Rai Pal proximity chat UDP
// socket, for testing the loopback spatial audio self-test. See
// backend/core/src/proximity_chat/position_socket.rs for the packet format.
//
// Usage:
//   node scripts/proximity-chat/send.mjs --position 1,2,3 --rotation 0,1.57,0
//   node scripts/proximity-chat/send.mjs --orbit
//   node scripts/proximity-chat/send.mjs --orbit --radius 3 --period 6 --json
//   node scripts/proximity-chat/send.mjs --orbit --source 4,0,0
//
// The optional --source places the sound source at a fixed position while the
// listener (--position/--orbit) moves, which is how the loopback tests the 3D
// effect. Without it, Rai Pal uses its fallback peer position.
//
// Press Ctrl+C to stop.

import dgram from "node:dgram";

const MAGIC = Buffer.from("RPAL", "ascii");
const VERSION_LISTENER = 1;
const VERSION_WITH_SOURCE = 2;
const HEADER_SIZE = 8;

const args = parseArgs(process.argv.slice(2));
const host = args.host ?? "127.0.0.1";
const port = Number(args.port ?? 6767);
const useJson = Boolean(args.json);
const source = parseVec(args.source, undefined);

const socket = dgram.createSocket("udp4");

function encodePacket(position, rotation, soundSource) {
	if (useJson) {
		const payload = { position, rotation };
		if (soundSource) {
			payload.source = soundSource;
		}
		return Buffer.from(JSON.stringify(payload));
	}

	const floatCount = soundSource ? 9 : 6;
	const packet = Buffer.alloc(HEADER_SIZE + floatCount * 4);
	MAGIC.copy(packet, 0);
	packet.writeUInt8(soundSource ? VERSION_WITH_SOURCE : VERSION_LISTENER, 4);
	position.forEach((value, index) =>
		packet.writeFloatLE(value, HEADER_SIZE + index * 4),
	);
	rotation.forEach((value, index) =>
		packet.writeFloatLE(value, HEADER_SIZE + 12 + index * 4),
	);
	if (soundSource) {
		soundSource.forEach((value, index) =>
			packet.writeFloatLE(value, HEADER_SIZE + 24 + index * 4),
		);
	}
	return packet;
}

function send(position, rotation, soundSource) {
	const packet = encodePacket(position, rotation, soundSource);
	socket.send(packet, port, host, (error) => {
		if (error) {
			console.error(`Failed to send packet: ${error.message}`);
		}
	});
}

function parseVec(value, fallback) {
	if (value === undefined) {
		return fallback;
	}
	const parts = String(value).split(",").map(Number);
	if (parts.length !== 3 || parts.some((part) => !Number.isFinite(part))) {
		throw new Error(
			`Expected a comma-separated vector of 3 numbers, got "${value}"`,
		);
	}
	return parts;
}

console.log(
	`Sending proximity chat packets to ${host}:${port} (${useJson ? "json" : "binary"})`,
);

if (args.orbit) {
	const radius = Number(args.radius ?? 2);
	const period = Number(args.period ?? 4);
	const rotation = parseVec(args.rotation, [0, 0, 0]);
	const startedAt = Date.now();

	const tick = () => {
		const angle = ((Date.now() - startedAt) / 1000 / period) * Math.PI * 2;
		send(
			[Math.cos(angle) * radius, 0, Math.sin(angle) * radius],
			rotation,
			source,
		);
	};
	tick();
	const interval = setInterval(tick, 50);
	process.on("SIGINT", () => {
		clearInterval(interval);
		socket.close();
		process.exit(0);
	});
} else {
	send(
		parseVec(args.position, [0, 0, 0]),
		parseVec(args.rotation, [0, 0, 0]),
		source,
	);
	socket.close();
}

function parseArgs(rawArgs) {
	const parsed = {};
	for (let index = 0; index < rawArgs.length; index += 1) {
		const arg = rawArgs[index];
		if (!arg.startsWith("--")) {
			continue;
		}
		const key = arg.slice(2);
		const next = rawArgs[index + 1];
		if (next === undefined || next.startsWith("--")) {
			parsed[key] = true;
		} else {
			parsed[key] = next;
			index += 1;
		}
	}
	return parsed;
}
