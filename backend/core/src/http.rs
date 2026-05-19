use std::{
	fs,
	io::{
		BufWriter,
		Write,
	},
	path::Path,
	sync::LazyLock,
	time::Duration,
};

use futures_util::StreamExt;

use crate::result::Result;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[allow(clippy::expect_used)]
	reqwest::Client::builder()
		.timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

pub static CLIENT_NO_REDIRECT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[allow(clippy::expect_used)]
	reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

pub async fn download_with_progress(
	url: &str,
	target_path: &Path,
	progress_callback: impl Fn(u64, Option<u64>) + Send,
) -> Result {
	let response = CLIENT.get(url).send().await?.error_for_status()?;
	let mut file = BufWriter::new(fs::File::create(target_path)?);

	let total_size = response.content_length();
	let mut downloaded: u64 = 0;

	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = chunk?;
		file.write_all(&chunk)?;
		downloaded += u64::try_from(chunk.len())?;
		progress_callback(downloaded, total_size);
	}

	file.flush()?;

	Ok(())
}
