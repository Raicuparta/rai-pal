use super::mod_loader::{ModLoader, ModLoaderData, ModLoaderId, ModLoaderStatic};
use crate::serializable_struct;

serializable_struct!(MelonLoader {
  pub data: ModLoaderData,
});

impl ModLoaderId for MelonLoader {
    const ID: &'static str = "melonloader";
}

impl ModLoaderStatic for MelonLoader {
    fn new(resources_path: &std::path::Path) -> crate::Result<Self>
    where
        Self: std::marker::Sized,
    {
        let path = resources_path.join(Self::ID);

        Ok(Self {
            data: ModLoaderData {
                id: Self::ID.to_string(),
                mods: vec![],
                path,
                mod_count: 0,
            },
        })
    }
}

impl ModLoader for MelonLoader {
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
