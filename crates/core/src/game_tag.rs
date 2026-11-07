use rai_pal_proc_macros::serializable_enum;

#[serializable_enum]
pub enum GameTag {
	VR,
	Demo,
}

impl GameTag {
	pub const ALL: [Self; 2] = [Self::VR, Self::Demo];
}
