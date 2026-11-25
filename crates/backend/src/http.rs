use std::{path::Path, sync::LazyLock, time::Duration};

use fs_err::tokio::File;
use futures_util::StreamExt;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::{
	mods::manifest::InstallStep,
	progress_status::ProgressStatus,
	result::{ContextResult, Result},
};

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[expect(
		clippy::expect_used,
		reason = "Startup invariant — the app cannot function without an HTTP client"
	)]
	reqwest::Client::builder()
		.connect_timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

pub static CLIENT_NO_REDIRECT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[expect(
		clippy::expect_used,
		reason = "Startup invariant — the app cannot function without an HTTP client"
	)]
	reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.connect_timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

pub async fn download(
	url: &str,
	target_path: &Path,
	step: &InstallStep,
	status_callback: &(impl Fn(ProgressStatus) + Send + Sync),
) -> Result {
	stream_to_file(
		url,
		target_path,
		None,
		|_, downloaded_bytes, total_bytes| {
			#[expect(
				clippy::cast_precision_loss,
				reason = "Precision loss is irrelevant for progress display"
			)]
			let percentage = total_bytes.map_or(0.0, |total| {
				if total == 0 {
					0.0
				} else {
					downloaded_bytes as f64 / total as f64
				}
			});

			status_callback(ProgressStatus {
				step: step.clone(),
				progress: percentage,
			});
		},
	)
	.await
}

pub async fn download_to_file(url: &str, target_path: &Path) -> Result {
	stream_to_file(url, target_path, None, |_, _, _| {}).await
}

pub async fn download_to_file_with_chunks(
	url: &str,
	target_path: &Path,
	timeout: Option<Duration>,
	mut on_chunk: impl FnMut(&[u8]),
) -> Result {
	stream_to_file(url, target_path, timeout, |chunk, _, _| on_chunk(chunk)).await
}

async fn stream_to_file(
	url: &str,
	target_path: &Path,
	timeout: Option<Duration>,
	mut on_chunk: impl FnMut(&[u8], usize, Option<u64>),
) -> Result {
	let mut request = CLIENT.get(url);

	if let Some(timeout) = timeout {
		request = request.timeout(timeout);
	}

	let response = request
		.send()
		.await
		.with_context(|| format!("failed to request `{url}`"))?
		.error_for_status()?;

	let file = File::create(target_path).await?;
	let mut file = BufWriter::new(file);

	let mut downloaded_bytes: usize = 0;

	let total_bytes = response.content_length();

	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = chunk.with_context(|| format!("failed to download `{url}`"))?;

		file.write_all(&chunk).await?;

		downloaded_bytes += chunk.len();

		on_chunk(&chunk, downloaded_bytes, total_bytes);
	}

	file.flush().await?;

	Ok(())
}
