use crate::mods::manifest::InstallStep;

#[derive(Clone, Debug)]
pub struct ProgressStatus {
	pub step: InstallStep,
	pub progress: f64,
}
