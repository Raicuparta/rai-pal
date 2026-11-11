use rai_pal_proc_macros::serde_struct;

#[serde_struct]
pub struct ModFamily {
	pub title: String,
	pub description: String,
}
