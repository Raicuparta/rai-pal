fn contains_ignore_case(text: &str, term: &str) -> bool {
	let trimmed_term = term.trim();
	if trimmed_term.is_empty() {
		return true;
	}
	text.to_lowercase()
		.trim()
		.contains(&trimmed_term.to_lowercase())
}

pub fn any_contains(texts: &[&str], term: &str) -> bool {
	texts.iter().any(|text| contains_ignore_case(text, term))
}
