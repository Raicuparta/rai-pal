#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModTag {
	pub title: String,
	pub description: String,
	pub filterable: Option<bool>,
}
