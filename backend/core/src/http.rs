use std::{
	path::Path,
	sync::LazyLock,
	time::Duration,
};

use futures_util::StreamExt;
use rai_pal_proc_macros::{
	serializable_enum,
	serializable_struct,
};
use tokio::{
	fs::File,
	io::{
		AsyncWriteExt,
		BufWriter,
	},
};

use crate::result::Result;

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[allow(clippy::expect_used)]
	reqwest::Client::builder()
		.connect_timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

pub static CLIENT_NO_REDIRECT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	#[allow(clippy::expect_used)]
	reqwest::Client::builder()
		.redirect(reqwest::redirect::Policy::none())
		.connect_timeout(Duration::from_secs(10))
		.pool_max_idle_per_host(10)
		.build()
		.expect("Failed to set up HTTP client")
});

#[serializable_struct]
pub struct DownloadStatus {
	url: String,
	target_path: String,
	downloaded: u64,
	total: Option<u64>,
}

pub async fn download(
	url: &str,
	target_path: &Path,
	status_callback: impl Fn(DownloadStatus) + Send,
) -> Result<()> {
	let response = CLIENT.get(url).send().await?.error_for_status()?;

	let file = File::create(target_path).await?;
	let mut file = BufWriter::new(file);

	let total_size = response.content_length();
	let mut downloaded: u64 = 0;

	let url_str = url.to_string();
	let target_path_str = target_path.to_string_lossy().into_owned();

	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = chunk?;

		file.write_all(&chunk).await?;

		downloaded += u64::try_from(chunk.len())?;

		status_callback(DownloadStatus {
			url: url_str.clone(),
			target_path: target_path_str.clone(),
			downloaded,
			total: total_size,
		});
	}

	file.flush().await?;

	status_callback(DownloadStatus {
		url: url_str,
		target_path: target_path_str,
		downloaded,
		total: downloaded.into(),
	});

	Ok(())
}
