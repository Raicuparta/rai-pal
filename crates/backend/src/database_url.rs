const PRODUCTION_BASE_URL: &str = "https://rai-pal-db.raicuparta.com";
const STAGING_BASE_URL: &str = "https://rai-pal-db-staging.raicuparta.com";

pub fn base_url() -> &'static str {
	if env!("CARGO_PKG_VERSION").contains("-pre") {
		STAGING_BASE_URL
	} else {
		PRODUCTION_BASE_URL
	}
}
