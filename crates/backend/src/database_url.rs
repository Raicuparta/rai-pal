const RELEASE_BASE_URL: &str = "https://rai-pal-db.raicuparta.com";
const PRERELEASE_BASE_URL: &str = "https://rai-pal-db-pre.raicuparta.com";

pub fn base_url() -> &'static str {
	if env!("CARGO_PKG_VERSION").contains("-pre") {
		PRERELEASE_BASE_URL
	} else {
		RELEASE_BASE_URL
	}
}
