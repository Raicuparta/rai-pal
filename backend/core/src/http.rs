use std::{
	path::Path,
	sync::LazyLock,
	time::Duration,
};

use futures_util::StreamExt;
use tokio::{
	fs::File,
	io::{
		AsyncWriteExt,
		BufWriter,
	},
};

use crate::{
	progress_status::ProgressStatus,
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

#[allow(
	clippy::future_not_send,
	reason = "status_callback is only used on one task"
)]
pub async fn download(
	url: &str,
	target_path: &Path,
	id: &str,
	status_callback: &(impl Fn(ProgressStatus) + Send),
) -> Result {
	let response = CLIENT.get(url).send().await?.error_for_status()?;

	let file = File::create(target_path).await?;
	let mut file = BufWriter::new(file);

	let mut downloaded_bytes: usize = 0;

	let id_str = id.to_string();
	let total_bytes = response.content_length();

	let mut stream = response.bytes_stream();

	while let Some(chunk) = stream.next().await {
		let chunk = chunk?;

		file.write_all(&chunk).await?;

		downloaded_bytes += chunk.len();

		#[allow(clippy::cast_precision_loss)]
		let percentage = total_bytes.map_or(0.0, |total| {
			if total == 0 {
				0.0
			} else {
				downloaded_bytes as f64 / total as f64
			}
		});

		status_callback(ProgressStatus::InProgress {
			id: id_str.clone(),
			progress: percentage,
		});
	}

	file.flush().await?;

	Ok(())
}
