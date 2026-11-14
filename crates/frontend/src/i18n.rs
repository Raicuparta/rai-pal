use rai_pal_backend::app_settings::{AppLocale, AppSettings};

pub fn detect_locale() -> Option<AppLocale> {
	let system_locale = sys_locale::get_locale()?;
	let language_code = system_locale.split(['-', '_']).next()?;

	let locale = AppLocale::from_code(language_code);

	if locale.is_none() {
		log::error!(
			"Couldn't find a localization for the language code {language_code}, extracted from locale string {system_locale}"
		);
	}

	locale
}

pub fn effective_locale(settings: &AppSettings, detected: Option<AppLocale>) -> AppLocale {
	settings
		.override_language
		.or(detected)
		.unwrap_or(AppLocale::EnUs)
}

pub fn apply_translation(locale: AppLocale) {
	let code = locale.code();

	if let Err(err) = slint::select_bundled_translation(code) {
		log::error!("Failed to select translation {code}: {err}");
		if let Err(fallback_err) = slint::select_bundled_translation("en") {
			log::warn!("Failed to select fallback translation: {fallback_err}");
		}
	}

	slint::update_all_translations();
}
