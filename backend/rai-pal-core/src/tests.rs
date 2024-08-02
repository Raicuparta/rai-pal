#[allow(clippy::unwrap_used)]
use crate::steam::appinfo::SteamAppInfoReader;

use std::path::Path;

#[test]
fn benchmark_thing() {
	let appinfo_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/appinfo.vdf");
	let reader = SteamAppInfoReader::new(&appinfo_path).unwrap();

	for item in reader {
		match item {
			Ok(item) => println!("item: {:?}", item.name),
			Err(err) => println!("error: {err:?}"),
		}
	}
}
