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

impl From<FilterValueLabel> for UiFilterValueLabel {
	fn from(value: FilterValueLabel) -> Self {
		match value {
			FilterValueLabel::Unknown => Self::Unknown,
			FilterValueLabel::Arch64 => Self::Arch64,
			FilterValueLabel::Arch32 => Self::Arch32,
			FilterValueLabel::TagDemo => Self::TagDemo,
			FilterValueLabel::TagVr => Self::TagVr,
			FilterValueLabel::TagUntagged => Self::TagUntagged,
			FilterValueLabel::StatusInstalled => Self::StatusInstalled,
			FilterValueLabel::StatusNotInstalled => Self::StatusNotInstalled,
			FilterValueLabel::ProviderManual => Self::ProviderManual,
		}
	}
}

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

pub const fn provider_id(provider: GameProviderId) -> ProviderId {
	match provider {
		GameProviderId::Amazon => ProviderId::Amazon,
		GameProviderId::Epic => ProviderId::Epic,
		GameProviderId::Gog => ProviderId::Gog,
		GameProviderId::Itch => ProviderId::Itch,
		GameProviderId::Manual => ProviderId::Manual,
		GameProviderId::Steam => ProviderId::Steam,
		GameProviderId::Xbox => ProviderId::Xbox,
	}
}

pub const fn core_provider_id(provider: ProviderId) -> Option<GameProviderId> {
	Some(match provider {
		ProviderId::None => return None,
		ProviderId::Amazon => GameProviderId::Amazon,
		ProviderId::Epic => GameProviderId::Epic,
		ProviderId::Gog => GameProviderId::Gog,
		ProviderId::Itch => GameProviderId::Itch,
		ProviderId::Manual => GameProviderId::Manual,
		ProviderId::Steam => GameProviderId::Steam,
		ProviderId::Xbox => GameProviderId::Xbox,
	})
}

pub const fn engine_id(engine: Option<EngineBrand>) -> EngineId {
	match engine {
		Some(EngineBrand::Unity) => EngineId::Unity,
		Some(EngineBrand::Unreal) => EngineId::Unreal,
		Some(EngineBrand::Godot) => EngineId::Godot,
		Some(EngineBrand::GameMaker) => EngineId::GameMaker,
		None => EngineId::None,
	}
}

pub const fn provider_action(action: ProviderCommandAction) -> ProviderAction {
	match action {
		ProviderCommandAction::StartViaProvider => ProviderAction::StartViaProvider,
		ProviderCommandAction::StartViaExe => ProviderAction::StartViaExe,
		ProviderCommandAction::Install => ProviderAction::Install,
		ProviderCommandAction::ShowInLibrary => ProviderAction::ShowInLibrary,
		ProviderCommandAction::ShowInStore => ProviderAction::ShowInStore,
		ProviderCommandAction::OpenInBrowser => ProviderAction::OpenInBrowser,
		ProviderCommandAction::Delete => ProviderAction::Delete,
	}
}

pub const fn core_provider_action(action: ProviderAction) -> Option<ProviderCommandAction> {
	Some(match action {
		ProviderAction::None => return None,
		ProviderAction::StartViaProvider => ProviderCommandAction::StartViaProvider,
		ProviderAction::StartViaExe => ProviderCommandAction::StartViaExe,
		ProviderAction::Install => ProviderCommandAction::Install,
		ProviderAction::ShowInLibrary => ProviderCommandAction::ShowInLibrary,
		ProviderAction::ShowInStore => ProviderCommandAction::ShowInStore,
		ProviderAction::OpenInBrowser => ProviderCommandAction::OpenInBrowser,
		ProviderAction::Delete => ProviderCommandAction::Delete,
	})
}

pub const fn core_skip_confirmation(skip: UiSkipConfirmation) -> Option<CoreSkipConfirmation> {
	match skip {
		UiSkipConfirmation::None => None,
		UiSkipConfirmation::InstallMod => Some(CoreSkipConfirmation::InstallMod),
	}
}
