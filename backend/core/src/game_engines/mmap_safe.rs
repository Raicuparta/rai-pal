//! Safe(r) wrapper around [`memmap2::Mmap`] for reading PE executables.
//!
//! # Safety of memory-mapping
//!
//! [`memmap2::Mmap::map`] is marked `unsafe` because the operating system maps
//! the file's *live* pages into the process address space. If the underlying
//! file is truncated or modified (by any process) while the mapping exists,
//! the behaviour is **undefined**: the process may observe torn writes, receive
//! `SIGBUS`, or — on some platforms — read stale page-cache contents.
//!
//! In **rai-pal** we only map:
//! - Installed game executables (`.exe` / `.dll` files on disk).
//! - During engine-detection scan functions that **create, use, and drop** the
//!   mapping within a single synchronous call — the mapping never escapes the
//!   function's stack frame.
//!
//! These executables are not expected to be modified while a scan is running:
//! - Games are not being patched or uninstalled mid-scan.
//! - Scans complete in milliseconds for a single file.
//!
//! If the file *were* externally truncated during the scan, the worst outcome
//! is a process crash (`SIGBUS`), which is functionally equivalent to an I/O
//! error for our purposes, not a silent memory corruption.
//!
//! Because the mapping lifetime is so short and the risk is both unlikely and
//! unactionable at this layer, we encapsulate the single `unsafe` call here
//! rather than pushing the burden onto every call site.

use std::{
	fs::File,
	io,
	path::Path,
};

use memmap2::Mmap;

/// Memory-map `path` read-only.
///
/// Returns an I/O error if the file cannot be opened or mapped. The returned
/// [`Mmap`] must not outlive the calling function (see [module-level
/// documentation](self) for the safety rationale).
pub fn map_readonly(path: &Path) -> io::Result<Mmap> {
	let file = File::open(path)?;

	// SAFETY: See module-level safety documentation. The caller must ensure
	// the mapping does not escape the current stack frame — rai-pal's engine
	// scanners satisfy this by construction.
	let mmap = unsafe { Mmap::map(&file) }?;

	// The `File` handle can be dropped: the OS keeps the mapping alive
	// independently.
	drop(file);

	Ok(mmap)
}
