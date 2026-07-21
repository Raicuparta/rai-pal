use rai_pal_proc_macros::serializable_struct;

#[serializable_struct]
pub struct ProgressStatus {
	pub id: String,
	pub name: String,
	pub current: f64,
	pub total: Option<f64>,
}

impl ProgressStatus {
	pub fn new(id: String, name: String, current: usize, total: Option<u64>) -> Self {
		Self {
			id,
			name,

			#[allow(clippy::cast_precision_loss)]
			current: current as f64,

			#[allow(clippy::cast_precision_loss)]
			total: total.map(|total| total as f64),
		}
	}
}
