const RELEASE_BASE_URL: &str = "https://rai-pal.pages.dev";
const PRERELEASE_BASE_URL: &str = "https://dev.rai-pal.pages.dev";

pub fn base_url() -> &'static str {
	if env!("CARGO_PKG_VERSION").contains("-pre") {
		PRERELEASE_BASE_URL
	} else {
		RELEASE_BASE_URL
	}
}
