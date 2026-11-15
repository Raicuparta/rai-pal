---
name: windows-vm
description: Use when testing Windows-only behavior of Rai Pal on the Windows VM, building the Windows exe, or deploying a build to it.
---

# Windows VM Testing

Windows-only code (registry, `LOCALAPPDATA`, provider detection) can't be verified on Linux — `--target x86_64-pc-windows-gnu` fails in `aws-lc-sys` and no MSVC target is installed. So: build with the podman Windows builder, run it in the VM.

## Confirm the VM first

`ls /mnt/big_nvme/VMs/windows-11/shared` — if it exists, the VM is `win11` on this host and the steps below apply. If not, ask the user about the Windows VM; don't hunt for other VMs.

## Driving the VM

Guest agent runs as SYSTEM, no sudo/SSH. Helper, recreate if missing at `/tmp/win.js`:

```js
const { execFileSync } = require("child_process");
function v(a) { return JSON.parse(execFileSync("virsh", ["-c", "qemu:///system", "qemu-agent-command", "win11", JSON.stringify(a)], { encoding: "utf8" })).return; }
function run(exe, args) {
	const r = v({ execute: "guest-exec", arguments: { path: exe, arg: args.map((a) => a.replace(/\\/g, "/")), "capture-output": true } });
	let st;
	for (let i = 0; i < 600; i++) {
		try { st = v({ execute: "guest-exec-status", arguments: { pid: r.pid } }); } catch (e) { }
		if (st && st.exited) break;
		Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 100);
	}
	if (!st || !st.exited) { console.error("timeout"); process.exit(124); }
	if (st["out-data"]) process.stdout.write(Buffer.from(st["out-data"], "base64"));
	if (st["err-data"]) process.stderr.write(Buffer.from(st["err-data"], "base64"));
	process.exit(st.exitcode ?? 0);
}
const [cmd, ...args] = process.argv.slice(2);
run("C:\\Windows\\System32\\" + cmd.replace(/\//g, "\\"), args);
```

`node win.js cmd.exe /c Z:/script.bat`, or `node win.js WindowsPowerShell/v1.0/powershell.exe -ExecutionPolicy Bypass -File Z:/script.ps1`.

- Args lose unquoted backslashes, so paths in args use `/`. Args with spaces + quotes break: write a `.bat`/`.ps1` to the share and run that, redirecting output to `Z:\out.txt` to read from the host.
- Host `/mnt/big_nvme/VMs/windows-11/shared` is guest `Z:\`. Guest user is `pc` (`C:\Users\pc`, desktop is a convenient drop spot).
- Per-user registry is not SYSTEM's HKCU: use `HKU\<SID>` from `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\ProfileList`.
- No clicking or typing available; `virsh screenshot win11 /tmp/s.ppm` only observes. Ask the user for UI actions.
- Don't launch the launcher exe with a URI as an argument to test deeplinks (spawns a broken second instance) — give the user the URI for Win+R or `start "" "<uri>"`.

## Deploying a Windows build

```bash
cd /mnt/big_nvme/Repos/rai-pal
cp Cargo.toml /tmp/Cargo.toml.bak
podman run --rm -v /mnt/big_nvme/Repos/rai-pal:/app -v rai-pal-builder-cargo:/usr/local/cargo -w /app \
  ghcr.io/raicuparta/rai-pal-builder:windows-latest bash -c 'bash scripts/build-windows.sh'
cp /tmp/Cargo.toml.bak Cargo.toml && git checkout Cargo.lock
```

`build-windows.sh` strips the winit git patch in place and changes `Cargo.lock`, so restore both. Artifacts in `target/bundles/nsis/`: `RaiPal-<version>-x86_64.exe` (portable, keeps its data and `Rai Pal.log` next to itself) and `...-setup.exe`.

```bash
mkdir -p /mnt/big_nvme/VMs/windows-11/shared/rai-pal-build
cp target/bundles/nsis/RaiPal-*-x86_64*.exe /mnt/big_nvme/VMs/windows-11/shared/rai-pal-build/
```

then in the guest copy `Z:/rai-pal-build/*.exe` to `C:\Users\pc\Desktop\rai-pal-build\`.

## Verifying

Let the user run the app and report. When something provider-specific misbehaves, read that provider launcher's own log in the guest first — it usually names the exact failure.
