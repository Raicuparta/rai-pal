use rai_pal_proc_macros::serializable_enum;

#[serializable_enum]
pub enum GameSubscription {
	UbisoftClassics,
	UbisoftPremium,
	XboxGamePass,
	EaPlay,
}
