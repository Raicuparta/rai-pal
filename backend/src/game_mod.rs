use crate::{
	local_mod::LocalMod,
	mod_loaders::mod_database::DatabaseMod,
	serializable_struct,
};

serializable_struct!(GameMod {
  pub local_mod: Option<LocalMod>,
  pub database_mod: Option<DatabaseMod>,
});
