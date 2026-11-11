use fs_err as fs;
use std::{collections::HashSet, path::PathBuf};

use rai_pal_proc_macros::{serializable_enum, serializable_struct};

use crate::{
	app_paths,
	games_query::{FilterGroupKey, GamesQuery},
	path_extensions::PathExt,
	result::Result,
};

#[serializable_enum]
pub enum AppLocale {
	EnUs,
	DeDe,
	EsEs,
	FrFr,
	JaJp,
	KoKr,
	PtPt,
	ZhCn,
	WaWa,
}

struct LocaleInfo {
	code: &'static str,
	native_name: &'static str,
}

impl AppLocale {
	const fn info(self) -> LocaleInfo {
		match self {
			Self::EnUs => LocaleInfo {
				code: "en",
				native_name: "English (US)",
			},
			Self::DeDe => LocaleInfo {
				code: "de",
				native_name: "Deutsch (Deutschland)",
			},
			Self::EsEs => LocaleInfo {
				code: "es",
				native_name: "Español (España)",
			},
			Self::FrFr => LocaleInfo {
				code: "fr",
				native_name: "Français (France)",
			},
			Self::JaJp => LocaleInfo {
				code: "ja",
				native_name: "日本語",
			},
			Self::KoKr => LocaleInfo {
				code: "ko",
				native_name: "한국어",
			},
			Self::PtPt => LocaleInfo {
				code: "pt",
				native_name: "Português (Portugal)",
			},
			Self::ZhCn => LocaleInfo {
				code: "zh",
				native_name: "简体中文",
			},
			Self::WaWa => LocaleInfo {
				code: "wa",
				native_name: "Wario (Waah!)",
			},
		}
	}

	pub const fn code(self) -> &'static str {
		self.info().code
	}

	pub const fn native_name(self) -> &'static str {
		self.info().native_name
	}

	pub fn from_code(code: &str) -> Option<Self> {
		Self::ALL.into_iter().find(|locale| locale.code() == code)
	}
}

#[serializable_enum]
#[derive(Default)]
pub enum TabId {
	#[default]
	Games,
	Mods,
	Thanks,
}

#[serializable_enum]
#[derive(Default)]
pub enum ColorScheme {
	#[default]
	Auto,
	Dark,
	Light,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SkipConfirmation {
	#[serde(rename = "skip-confirm-install-mod-confirm")]
	InstallMod,
}

#[serializable_struct]
#[derive(Default, Copy)]
pub struct WindowState {
	pub x: i32,
	pub y: i32,
	pub width: u32,
	pub height: u32,
	pub maximized: bool,
}

#[serializable_struct]
#[derive(Default)]
pub struct AppSettings {
	pub hide_game_thumbnails: bool,
	pub override_language: Option<AppLocale>,
	pub games_query: GamesQuery,
	pub selected_tab: TabId,
	#[serde(default)]
	pub skip_confirm_dialogs: HashSet<SkipConfirmation>,
	#[serde(default)]
	pub expanded_filters: HashSet<FilterGroupKey>,
	#[serde(default)]
	pub color_scheme: ColorScheme,
	#[serde(default)]
	pub window_state: Option<WindowState>,
}

// If the settings schema changes, update this so it gets recreated.
const SETTINGS_VERSION: u32 = 1u32;

impl AppSettings {
	fn try_read() -> Result<Self> {
		let path = Self::get_path()?;
		if !path.is_file() {
			return Ok(Self::default());
		}

		let data = fs::read_to_string(&path)?;

		Ok(serde_json::from_str(&data)?)
	}

	pub fn read() -> Self {
		Self::try_read().unwrap_or_else(|err| {
			log::error!("Failed to read app settings, resetting to default. Error: {err}");
			Self::default()
		})
	}

	pub fn try_write(&self) -> Result {
		let path = Self::get_path()?;
		fs::create_dir_all(path.try_parent()?)?;

		// Write to a temp file first so a crash mid-write can't corrupt settings.
		let temp_path = path.with_extension("json.tmp");
		fs::write(&temp_path, serde_json::to_string(self)?)?;
		fs::rename(&temp_path, &path)?;

		Ok(())
	}

	fn get_path() -> Result<PathBuf> {
		app_paths::app_data_file(&format!("settings-{SETTINGS_VERSION}.json"))
	}
}
