pub fn get_steam_thumbnail(app_id: &str) -> String {
	format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/capsule_231x87.jpg")
}
