use crate::files::copy_dir_all;
use crate::game::{Game, OperatingSystem};
use crate::mod_loaders::mod_loader::{ModLoader, ModLoaderData};
use crate::{game::UnityScriptingBackend, game_mod::Mod};
use crate::{serializable_struct, Result};
use anyhow::anyhow;
use glob::glob;
use std::fs;
use std::path::Path;

serializable_struct!(BepInEx {
    pub data: ModLoaderData,
});

impl ModLoader for BepInEx {
    fn new(resources_path: &Path) -> Result<Self> {
        let id = "bepinex".to_string();
        let path = resources_path.join(id.clone());

        let mut mods = find_mods(&path, UnityScriptingBackend::Il2Cpp)?;
        mods.append(&mut find_mods(&path, UnityScriptingBackend::Mono)?);
        let mod_count = mods.len();

        Ok(Self {
            data: ModLoaderData {
                id,
                mods,
                path,
                mod_count: u32::try_from(mod_count)?,
            },
        })
    }

    fn get_data(&self) -> ModLoaderData {
        self.data.clone()
    }

    fn install(&self, game: &Game) -> Result {
        let scripting_backend_path = &self.data.path.join(game.scripting_backend.to_string());
        let architecture_path = scripting_backend_path
            .join(game.operating_system.to_string())
            .join(game.architecture.to_string());

        let mod_files_folder = architecture_path.join("mod-files");
        let folder_to_copy_to_game = architecture_path.join("copy-to-game");
        let game_data_folder = &game.get_installed_mods_folder()?;

        copy_dir_all(mod_files_folder, game_data_folder)
            .map_err(|err| anyhow!("Failed to copy mod loader files: {err}"))?;

        let game_folder = game
            .full_path
            .parent()
            .ok_or_else(|| anyhow!("Failed to get game parent folder"))?;

        copy_dir_all(folder_to_copy_to_game, game_folder)?;

        let config_origin_path =
            &self
                .data
                .path
                .join("config")
                .join(if game.unity_version.is_legacy {
                    "BepInEx-legacy.cfg"
                } else {
                    "BepInEx.cfg"
                });

        let config_target_folder = game_data_folder.join("BepInEx").join("config");

        fs::create_dir_all(&config_target_folder)?;

        fs::copy(config_origin_path, config_target_folder.join("BepInEx.cfg"))?;

        let doorstop_config =
            fs::read_to_string(scripting_backend_path.join("doorstop_config.ini"))?;

        fs::write(
            game_folder.join("doorstop_config.ini"),
            doorstop_config.replace(
                "{{MOD_FILES_PATH}}",
                game_data_folder
                    .to_str()
                    .ok_or_else(|| anyhow!("Failed to parse game data folder"))?,
            ),
        )?;

        if std::env::consts::OS != "windows" && game.operating_system == OperatingSystem::Windows {
            if let Some(steam_launch) = &game.steam_launch {
                ensure_wine_will_load_bepinex(&game.full_path, steam_launch.app_id)?;
            }
        }

        Ok(())
    }

    fn install_mod(&self, game: &Game, mod_id: String) -> Result {
        let game_mod = self
            .data
            .mods
            .iter()
            .find(|game_mod| game_mod.id == mod_id)
            .ok_or_else(|| anyhow!("Failed to find mod with id {mod_id}"))?;

        self.install(game)?;
        game_mod.install(game)
    }

    fn open_mod_folder(&self, mod_id: String) -> Result {
        let game_mod = self
            .data
            .mods
            .iter()
            .find(|game_mod| game_mod.id == mod_id)
            .ok_or_else(|| anyhow!("Failed to find mod with id {mod_id}"))?;

        game_mod.open_folder()
    }
}

fn ensure_wine_will_load_bepinex(game_path: &Path, steam_app_id: u32) -> Result {
    // I think since this gets rid of symbolic links then it shouldn't be possible to
    // get stuck in an infinite loop, but who knows. If you're currently stuck in
    // an infinite loop then I'm very sorry, hope infinity goes well for you.
    let mut steam_apps_folder = game_path.canonicalize()?;

    loop {
        if steam_apps_folder.ends_with("steamapps") {
            break;
        }

        steam_apps_folder = steam_apps_folder
            .parent()
            .ok_or_else(|| {
                anyhow!(
                    "Failed to get steamapps folder from {}",
                    game_path.display()
                )
            })?
            .to_path_buf();
    }

    let compat_data_dir = steam_apps_folder
        .join("compatdata")
        .join(steam_app_id.to_string());
    let pfx_folder = compat_data_dir.join("pfx");
    let user_reg = pfx_folder.join("user.reg");

    let user_reg_data = fs::read_to_string(&user_reg).map_err(|err| {
        anyhow!(
            "Failed to read registry file from {}: {err}",
            user_reg.display()
        )
    })?; // TODO: handle error

    let ensured_user_reg_data = reg_add_in_section(
        &user_reg_data,
        "[Software\\\\Wine\\\\DllOverrides]",
        "winhttp",
        "native,builtin",
    );

    if user_reg_data != ensured_user_reg_data {
        fs::copy(&user_reg, pfx_folder.join("user.reg.bak"))?;
        fs::write(&user_reg, ensured_user_reg_data)?;
    }

    Ok(())
}

fn reg_add_in_section(reg: &str, section: &str, key: &str, value: &str) -> String {
    let mut split = reg.split('\n').collect::<Vec<_>>();

    let mut begin = 0;

    for (index, line) in split.iter().enumerate() {
        if line.starts_with(section) {
            begin = index + 2;
            break;
        }
    }

    let mut end = 0;

    for (index, line) in split.iter().enumerate().skip(begin) {
        if line.is_empty() {
            end = index;
            break;
        }
    }

    let line_start = &format!("\"{key}\"");
    let new_line = format!("{line_start}=\"{value}\"");

    for (_, line) in split.iter_mut().enumerate().skip(begin) {
        if line.starts_with(line_start) {
            *line = &new_line;
            return split.join("\n");
        }
    }

    split.insert(end, &new_line);

    split.join("\n")
}

fn find_mods(mod_loader_path: &Path, scripting_backend: UnityScriptingBackend) -> Result<Vec<Mod>> {
    let mods_folder_path = mod_loader_path
        .join(scripting_backend.to_string())
        .join("mods");

    let entries: Vec<_> = glob(
        mods_folder_path
            .join("*")
            .to_str()
            .ok_or_else(|| anyhow!("Failed to parse mods folder path"))?,
    )?
    .collect();

    Ok(entries
        .iter()
        .filter_map(|entry| match entry {
            Ok(mod_path) => Some(Mod::new(mod_path, scripting_backend).ok()?),
            Err(_) => None,
        })
        .collect())
}
