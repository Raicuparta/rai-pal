use lazy_regex::{regex, regex_replace_all};
use rai_pal_proc_macros::serializable_struct;

#[serializable_struct]
pub struct GameTitle {
	pub display: String,
	pub normalized: Vec<String>,
}

static DEMO_REGEX: &lazy_regex::Lazy<regex::Regex> = regex!(r"(?i).*[\((\s+)]demo\)?$");
static BRACKETS_REGEX: &lazy_regex::Lazy<regex::Regex> = regex!(r"(?i).*[\((\s+)]demo\)?$");

impl GameTitle {
	pub fn new(display: &str) -> Self {
		Self {
			display: display.to_string(),
			normalized: get_normalized_titles(display),
		}
	}

	pub fn is_probably_demo(&self) -> bool {
		DEMO_REGEX.is_match(&self.display.to_lowercase())
	}
}

// Titles given by the game providers can have all sorts of trash.
// But we want to be able to use the titles to match some local and remote games.
// So we need to normalize the titles.
// Some ways of normalizing the titles work for some games/providers, some work for others.
// So we have a list of different normalization methods, so we can try each one later.
fn get_normalized_titles(title: &str) -> Vec<String> {
	vec![
		normalize_title(title),
		normalize_title(&BRACKETS_REGEX.replace_all(title, "")),
		normalize_title(&DEMO_REGEX.replace_all(title, "")),
	]
}

fn normalize_title(title: &str) -> String {
	regex_replace_all!(r"\W+", title, "").to_lowercase()
}
