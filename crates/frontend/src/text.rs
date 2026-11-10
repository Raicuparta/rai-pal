/// Up to two initials derived from a display name.
pub fn initials(name: &str) -> Option<String> {
	let words = name.split_whitespace().collect::<Vec<_>>();

	if words.is_empty() {
		return None;
	}

	let initials = if words.len() == 1 {
		words[0]
			.chars()
			.filter(char::is_ascii_alphanumeric)
			.collect::<String>()
	} else {
		words
			.iter()
			.filter_map(|word| word.chars().find(char::is_ascii_alphanumeric))
			.collect::<String>()
	};

	let initials = initials.chars().take(2).collect::<String>().to_uppercase();

	if initials.is_empty() {
		None
	} else {
		Some(initials)
	}
}
