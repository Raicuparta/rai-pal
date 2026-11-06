use rai_pal_core::app_settings::{AppLocale, AppSettings};

pub fn locale_code(locale: AppLocale) -> &'static str {
	match locale {
		AppLocale::EnUs => "en",
		AppLocale::DeDe => "de",
		AppLocale::EsEs => "es",
		AppLocale::FrFr => "fr",
		AppLocale::JaJp => "ja",
		AppLocale::KoKr => "ko",
		AppLocale::PtPt => "pt",
		AppLocale::ZhCn => "zh",
		AppLocale::WaWa => "wa",
	}
}

pub fn native_name(locale: AppLocale) -> &'static str {
	match locale {
		AppLocale::EnUs => "English (US)",
		AppLocale::DeDe => "Deutsch (Deutschland)",
		AppLocale::EsEs => "Español (España)",
		AppLocale::FrFr => "Français (France)",
		AppLocale::JaJp => "日本語",
		AppLocale::KoKr => "한국어",
		AppLocale::PtPt => "Português (Portugal)",
		AppLocale::ZhCn => "简体中文",
		AppLocale::WaWa => "Wario (Waah!)",
	}
}

pub fn detect_locale() -> Option<AppLocale> {
	let locale = sys_locale::get_locale()?;
	let language_code = locale.split(['-', '_']).next()?;

	let locale = match language_code {
		"en" => AppLocale::EnUs,
		"pt" => AppLocale::PtPt,
		"de" => AppLocale::DeDe,
		"es" => AppLocale::EsEs,
		"fr" => AppLocale::FrFr,
		"ja" => AppLocale::JaJp,
		"ko" => AppLocale::KoKr,
		"zh" => AppLocale::ZhCn,
		_ => {
			eprintln!(
				"Couldn't find a localization for the language code {language_code}, extracted from locale string {locale}"
			);
			return None;
		}
	};

	Some(locale)
}

pub fn effective_locale(settings: &AppSettings, detected: Option<AppLocale>) -> AppLocale {
	settings
		.override_language
		.or(detected)
		.unwrap_or(AppLocale::EnUs)
}

pub fn apply_translation(locale: AppLocale) {
	let code = locale_code(locale);

	if let Err(err) = slint::select_bundled_translation(code) {
		eprintln!("Failed to select translation {code}: {err}");
		let _ = slint::select_bundled_translation("en");
	}

	slint::update_all_translations();
}

pub fn locale_from_code(code: &str) -> Option<AppLocale> {
	match code {
		"en" => Some(AppLocale::EnUs),
		"de" => Some(AppLocale::DeDe),
		"es" => Some(AppLocale::EsEs),
		"fr" => Some(AppLocale::FrFr),
		"ja" => Some(AppLocale::JaJp),
		"ko" => Some(AppLocale::KoKr),
		"pt" => Some(AppLocale::PtPt),
		"zh" => Some(AppLocale::ZhCn),
		"wa" => Some(AppLocale::WaWa),
		_ => None,
	}
}
