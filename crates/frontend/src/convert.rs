use rai_pal_backend::{
	filter_views::{FilterGroupLabel, FilterValueLabel, FilterValueNote},
	games_query::FilterGroupKey,
};

use crate::{
	FilterGroupId, FilterGroupLabel as UiFilterGroupLabel, FilterValueLabel as UiFilterValueLabel,
	FilterValueNote as UiFilterValueNote,
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

pub fn filter_value_label(label: Option<FilterValueLabel>) -> UiFilterValueLabel {
	label.map_or(UiFilterValueLabel::None, UiFilterValueLabel::from)
}
