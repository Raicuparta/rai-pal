use rai_pal_proc_macros::serializable_enum;

#[serializable_enum]
#[derive(sqlx::Type)]
pub enum GameTag {
	VR,
	Demo,
}
