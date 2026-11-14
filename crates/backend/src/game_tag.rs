use rai_pal_proc_macros::serde_enum;

#[serde_enum]
pub enum GameTag {
	VR,
	Demo,
}
