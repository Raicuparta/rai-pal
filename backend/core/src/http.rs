use std::{
	path::Path,
	sync::LazyLock,
	time::Duration,
};

use futures_util::StreamExt;
use rai_pal_proc_macros::serializable_struct;
use tokio::{
	fs::File,
	io::{
		AsyncWriteExt,
		BufWriter,
	},
};

use crate::{
	path_extensions::AsValidStr,
	result::Result,
};

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
	downloaded_bytes: f64,
	total_bytes: Option<f64>,
}

impl DownloadStatus {
	pub fn new(
		url: String,
		target_path: String,
		downloaded_bytes: usize,
		total_bytes: Option<u64>,
	) -> Self {
		Self {
			url,
			target_path,

			#[allow(clippy::cast_precision_loss)]
			downloaded_bytes: downloaded_bytes as f64,

			#[allow(clippy::cast_precision_loss)]
			total_bytes: total_bytes.map(|total| total as f64),
		}
	}
}

pub async fn download(
	url: &str,
	target_path: &Path,
	status_callback: impl Fn(DownloadStatus) + Send,
) -> Result<()> {
	let response = CLIENT.get(url).send().await?.error_for_status()?;

	let file = File::create(target_path).await?;
	let mut file = BufWriter::new(file);

	let mut downloaded_bytes: usize = 0;

	let url_str = url.to_string();
	let target_path_str = target_path.try_to_str()?;
	let total_bytes = response.content_length();

	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = chunk?;

		file.write_all(&chunk).await?;

		downloaded_bytes += chunk.len();

		status_callback(DownloadStatus::new(
			url_str.clone(),
			target_path_str.to_string(),
			downloaded_bytes,
			total_bytes,
		));
	}

	file.flush().await?;

	status_callback(DownloadStatus::new(
		url_str,
		target_path_str.to_string(),
		downloaded_bytes,
		Some(downloaded_bytes as u64),
	));

	Ok(())
}
