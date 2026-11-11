use rai_pal_backend::{
	app_settings::{ColorScheme, SkipConfirmation as CoreSkipConfirmation, TabId},
	filter_views::{FilterGroupLabel, FilterValueLabel, FilterValueNote},
	game_engines::game_engine::EngineBrand,
	game_providers::{game_provider::GameProviderId, provider_command::ProviderCommandAction},
	games_query::FilterGroupKey,
};

use crate::{
	AppColorScheme, AppTab, EngineId, FilterGroupId, FilterGroupLabel as UiFilterGroupLabel,
	FilterValueLabel as UiFilterValueLabel, FilterValueNote as UiFilterValueNote, ProviderAction,
	ProviderId, SkipConfirmation as UiSkipConfirmation,
};

macro_rules! slint_enum_conversions {
	($core:ty => $ui:ty { $($variant:ident),+ $(,)? }) => {
		impl From<$core> for $ui {
			fn from(value: $core) -> Self {
				match value {
					$(<$core>::$variant => <$ui>::$variant,)+
				}
			}
		}

		impl From<$ui> for $core {
			fn from(value: $ui) -> Self {
				match value {
					$(<$ui>::$variant => <$core>::$variant,)+
				}
			}
		}
	};
}

macro_rules! slint_optional_enum_conversions {
	($core:ty => $ui:ty { $($variant:ident),+ $(,)? }) => {
		impl From<$core> for $ui {
			fn from(value: $core) -> Self {
				match value {
					$(<$core>::$variant => <$ui>::$variant,)+
				}
			}
		}

		impl From<Option<$core>> for $ui {
			fn from(value: Option<$core>) -> Self {
				value.map_or(<$ui>::None, <$ui>::from)
			}
		}

		impl From<$ui> for Option<$core> {
			fn from(value: $ui) -> Self {
				match value {
					<$ui>::None => None,
					$(<$ui>::$variant => Some(<$core>::$variant),)+
				}
			}
		}
	};
}

slint_enum_conversions!(FilterGroupKey => FilterGroupId {
	Installed,
	Engines,
	ModFamilies,
	Tags,
	Providers,
	UnityBackends,
	ExecutableOs,
	SupportedOs,
	Architectures,
});

slint_enum_conversions!(FilterGroupLabel => UiFilterGroupLabel {
	Status,
	Engine,
	Mod,
	Tag,
	Provider,
	UnityBackend,
	ExecutableOs,
	SupportedOs,
	Architecture,
});

slint_optional_enum_conversions!(FilterValueLabel => UiFilterValueLabel {
	Unknown,
	Arch64,
	Arch32,
	TagDemo,
	TagVr,
	TagUntagged,
	StatusInstalled,
	StatusNotInstalled,
	ProviderManual,
});

slint_enum_conversions!(FilterValueNote => UiFilterValueNote {
	None,
	EngineGodotNotFullySupported,
	EngineGameMakerNotFullySupported,
	ProviderXboxOnlyInstalled,
});

slint_enum_conversions!(TabId => AppTab {
	Games,
	Mods,
	Thanks,
});

slint_enum_conversions!(ColorScheme => AppColorScheme {
	Auto,
	Dark,
	Light,
});

slint_optional_enum_conversions!(GameProviderId => ProviderId {
	Amazon,
	Epic,
	Gog,
	Itch,
	Manual,
	Steam,
	Xbox,
});

slint_optional_enum_conversions!(EngineBrand => EngineId {
	Unity,
	Unreal,
	Godot,
	GameMaker,
});

slint_optional_enum_conversions!(ProviderCommandAction => ProviderAction {
	StartViaProvider,
	StartViaExe,
	Install,
	ShowInLibrary,
	ShowInStore,
	OpenInBrowser,
	Delete,
});

slint_optional_enum_conversions!(CoreSkipConfirmation => UiSkipConfirmation {
	InstallMod,
});
