use super::mod_loader::{ModLoader, ModLoaderData, ModLoaderID};
use crate::serializable_struct;

serializable_struct!(MelonLoader {
  pub data: ModLoaderData,
});

impl ModLoaderID for MelonLoader {
    const ID: &'static str = "melonloader";
}

impl ModLoader for MelonLoader {
    fn new(path: &std::path::Path) -> crate::Result<Self>
    where
        Self: std::marker::Sized,
    {
        Ok(Self {
            data: ModLoaderData {
                id: Self::ID.to_string(),
                mods: vec![],
                path: path.to_path_buf(),
                mod_count: 0,
            },
        })
    }

    fn get_id(&self) -> String {
        Self::ID.to_string()
    }

    fn get_data(&self) -> ModLoaderData {
        self.data.clone()
    }

    fn install(&self, _game: &crate::game::Game) -> crate::Result {
        todo!()
    }

    fn install_mod(&self, _game: &crate::game::Game, _mod_idd: String) -> crate::Result {
        todo!()
    }

    fn open_mod_folder(&self, _mod_id: String) -> crate::Result {
        todo!()
    }
}
