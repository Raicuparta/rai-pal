#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum ProgressStatus {
	Pending { id: String, name: String },
	InProgress { id: String, progress: f64 },
	Finished { id: String },
	Failed { id: String, error: String },
}
