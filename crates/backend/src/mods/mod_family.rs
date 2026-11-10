use rai_pal_proc_macros::serializable_struct;

#[serializable_struct]
pub struct ModFamily {
	pub title: String,
	pub description: String,
}
