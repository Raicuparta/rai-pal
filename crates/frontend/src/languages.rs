use std::rc::Rc;

use rai_pal_backend::app_settings::{AppLocale, AppSettings};
use slint::{ComponentHandle, ModelRc};

use crate::{AppSettings as UiAppSettings, HostStrings, MainWindow, SelectOption};

pub fn refresh(window: &MainWindow, settings: &AppSettings, detected: Option<AppLocale>) {
	let mut options: Vec<SelectOption> = Vec::with_capacity(AppLocale::ALL.len() + 1);

	if detected.is_some() {
		options.push(SelectOption {
			label: window
				.global::<HostStrings>()
				.get_auto_language_label()
				.to_string()
				.into(),
			value: "".into(),
		});
	}

	options.extend(AppLocale::ALL.iter().map(|locale| SelectOption {
		label: locale.native_name().into(),
		value: locale.code().into(),
	}));

	let selected = settings
		.override_language
		.map(AppLocale::code)
		.unwrap_or_default();
	let index = options
		.iter()
		.position(|option| option.value.as_str() == selected)
		.unwrap_or_default();

	let ui = window.global::<UiAppSettings>();
	ui.set_language_options(ModelRc::from(Rc::new(slint::VecModel::from(options))));
	ui.set_selected_language_index(i32::try_from(index).unwrap_or_default());
}
