use std::{
	collections::BTreeMap,
	path::{Path, PathBuf},
	process::ExitCode,
};

use rspolib::{FileOptions, pofile};

const DOMAIN: &str = "rai-pal-frontend";
const DEFAULT_TRANSLATIONS: &str = "crates/frontend/translations";

type Key = (String, String);

struct Catalog {
	messages: BTreeMap<Key, String>,
	structure_problems: Vec<String>,
}

fn main() -> ExitCode {
	match run() {
		Ok(()) => ExitCode::SUCCESS,
		Err(message) => {
			eprintln!("{message}");
			ExitCode::FAILURE
		}
	}
}

fn run() -> Result<(), String> {
	let (pot_path, translations_dir) = parse_args()?;
	let pot = read_catalog(&pot_path)?;

	if !pot.structure_problems.is_empty() {
		return Err(format!(
			"`{}` has duplicate or obsolete messages:\n  {}",
			pot_path.display(),
			pot.structure_problems.join("\n  ")
		));
	}

	let languages = language_dirs(&translations_dir)?;

	if languages.is_empty() {
		return Err(format!(
			"No translations found in `{}`",
			translations_dir.display()
		));
	}

	let mut failures = Vec::new();

	for (language, po_path) in &languages {
		let catalog = read_catalog(po_path)?;
		let mut problems = catalog.structure_problems;

		for key in catalog.messages.keys() {
			if !pot.messages.contains_key(key) {
				problems.push(format!("unused translation: {}", describe(key)));
			}
		}

		for key in pot.messages.keys() {
			let source = &key.1;

			let Some(translation) = catalog.messages.get(key) else {
				problems.push(format!("missing translation: {}", describe(key)));
				continue;
			};

			if translation.trim().is_empty() {
				problems.push(format!("empty translation: {}", describe(key)));
				continue;
			}

			if language == "en" {
				if translation != source {
					problems.push(format!(
						"English translation differs from the UI string: {}",
						describe(key)
					));
				}
				continue;
			}

			let expected = placeholders(source);
			let found = placeholders(translation);

			if expected != found {
				problems.push(format!(
					"placeholder mismatch for {}: expected {:?}, found {:?} in `{translation}`",
					describe(key),
					expected,
					found
				));
			}
		}

		if problems.is_empty() {
			println!("{language}: {} strings ok", pot.messages.len());
		} else {
			println!("{language}: {} problems", problems.len());
			for problem in &problems {
				println!("  {problem}");
			}
			failures.push(language.clone());
		}
	}

	if failures.is_empty() {
		println!(
			"Translations ok: {} strings, {} languages",
			pot.messages.len(),
			languages.len()
		);
		Ok(())
	} else {
		Err(format!(
			"Translation check failed for: {}",
			failures.join(", ")
		))
	}
}

fn parse_args() -> Result<(PathBuf, PathBuf), String> {
	let mut arguments = std::env::args().skip(1);
	let mut pot = None;
	let mut translations = PathBuf::from(DEFAULT_TRANSLATIONS);

	while let Some(argument) = arguments.next() {
		match argument.as_str() {
			"--pot" => {
				let Some(path) = arguments.next() else {
					return Err("`--pot` needs a path".to_owned());
				};
				pot = Some(PathBuf::from(path));
			}
			"--translations" => {
				let Some(path) = arguments.next() else {
					return Err("`--translations` needs a path".to_owned());
				};
				translations = PathBuf::from(path);
			}
			other => return Err(format!("Unknown argument `{other}`")),
		}
	}

	let Some(pot) = pot else {
		return Err("Missing `--pot <extracted .pot file>`".to_owned());
	};

	Ok((pot, translations))
}

fn language_dirs(translations: &Path) -> Result<Vec<(String, PathBuf)>, String> {
	let entries = fs_err::read_dir(translations)
		.map_err(|err| format!("Failed to read `{}`: {err}", translations.display()))?;

	let mut languages = Vec::new();

	for entry in entries {
		let entry =
			entry.map_err(|err| format!("Failed to read `{}`: {err}", translations.display()))?;
		let po_path = entry
			.path()
			.join("LC_MESSAGES")
			.join(format!("{DOMAIN}.po"));

		if !po_path.is_file() {
			continue;
		}

		let language = entry.file_name().to_string_lossy().into_owned();
		languages.push((language, po_path));
	}

	languages.sort_by(|left, right| left.0.cmp(&right.0));

	Ok(languages)
}

fn read_catalog(path: &Path) -> Result<Catalog, String> {
	let file = pofile(FileOptions::from(path))
		.map_err(|err| format!("Failed to parse `{}`: {err}", path.display()))?;

	let mut messages = BTreeMap::new();
	let mut structure_problems = Vec::new();

	for entry in file.entries {
		if entry.msgid.is_empty() {
			continue;
		}

		let key = (entry.msgctxt.unwrap_or_default(), entry.msgid);

		if entry.obsolete {
			structure_problems.push(format!("obsolete entry: {}", describe(&key)));
			continue;
		}

		if entry.flags.iter().any(|flag| flag == "fuzzy") {
			structure_problems.push(format!("fuzzy entry: {}", describe(&key)));
		}

		if messages
			.insert(key.clone(), entry.msgstr.unwrap_or_default())
			.is_some()
		{
			structure_problems.push(format!("duplicate entry: {}", describe(&key)));
		}
	}

	Ok(Catalog {
		messages,
		structure_problems,
	})
}

fn describe(key: &Key) -> String {
	format!("[{}] {}", key.0, key.1)
}

fn placeholders(text: &str) -> Vec<String> {
	let mut found = Vec::new();
	let mut characters = text.chars().peekable();

	while let Some(character) = characters.next() {
		match character {
			'{' => {
				if characters.peek() == Some(&'{') {
					characters.next();
					continue;
				}

				let mut placeholder = String::from("{");

				for inner in characters.by_ref() {
					placeholder.push(inner);
					if inner == '}' {
						break;
					}
				}

				found.push(placeholder);
			}
			'}' if characters.peek() == Some(&'}') => {
				characters.next();
			}
			_ => {}
		}
	}

	found.sort();

	found
}
