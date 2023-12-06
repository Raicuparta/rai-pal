use crate::{
	local_mod::LocalMod,
	mod_loaders::mod_database::RemoteMod,
	serializable_struct,
};

serializable_struct!(GameMod {
  pub local_mod: Option<LocalMod>,
  pub remote_mod: Option<RemoteMod>,
});
