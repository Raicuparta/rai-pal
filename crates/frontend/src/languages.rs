use std::rc::Rc;

use rai_pal_backend::app_settings::{AppLocale, AppSettings};
use slint::{ComponentHandle, ModelRc};

use crate::{AppSettings as UiAppSettings, MainWindow, SelectOption};

pub fn refresh(window: &MainWindow, settings: &AppSettings, detected: Option<AppLocale>) {
	let options: Vec<SelectOption> = AppLocale::ALL
		.iter()
		.map(|locale| SelectOption {
			label: locale.native_name().into(),
			value: locale.code().into(),
		})
		.collect();

	let selected_index = match settings.override_language {
		Some(locale) => AppLocale::ALL
			.iter()
			.position(|item| *item == locale)
			.and_then(|index| i32::try_from(index).ok())
			.unwrap_or_default(),
		None if detected.is_some() => -1,
		None => 0,
	};

	let ui = window.global::<UiAppSettings>();
	ui.set_language_options(ModelRc::from(Rc::new(slint::VecModel::from(options))));
	ui.set_selected_language_index(selected_index);
}
