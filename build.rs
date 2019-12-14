use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const INDEX_HTML_TEMPLATE: &'static str = include_str!("assets/template.html");

const MAPPING_HEADER: &'static str = 
r##"
/wasm-toys/common.js => bindings/common.js [text/javascript]
/wasm-toys/main.js => bindings/main.js [text/javascript]
/wasm-toys/input.js => bindings/input.js [text/javascript]
/wasm-toys/gl.js => bindings/gl.js [text/javascript]
"##;

const MAPPING_TEMPLATE: &'static str = 
r##"/wasm-toys/[[binary_name]] => target/html/[[binary_name]].html
/wasm-toys/[[binary_name]]-debug.wasm => target/wasm32-unknown-unknown/debug/[[binary_name]].wasm
/wasm-toys/[[binary_name]]-release.wasm => target/wasm32-unknown-unknown/release/[[binary_name]].wasm

"##;

fn main() {
	let profile = env::var("PROFILE").unwrap();

	let html_target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let html_target_dir = Path::new(&html_target_dir).join("target/html");
	std::fs::create_dir_all(&html_target_dir).unwrap();

	let bin_dir = Path::new("src/bin");
	let mut binaries = Vec::new();

	for path in bin_dir.read_dir().expect(&format!("Couldn't read {:?}", bin_dir)) {
		use std::ffi::OsStr;

		if let Ok(path) = path {
			let path = path.path();

			if path.extension() != Some(OsStr::new("rs"))
				&& !path.is_dir()  { continue }

			if let Some(Some(path)) = path.file_stem().map(OsStr::to_str) {
				binaries.push(path.to_owned());
			}
		}
	}

	let mapping_template = MAPPING_TEMPLATE.to_owned();

	let mut mappings_file = File::create("mappings.sb").unwrap();
	mappings_file.write_all(MAPPING_HEADER.as_bytes()).unwrap();

	for binary in binaries.iter() {
		let index_html = INDEX_HTML_TEMPLATE.to_owned()
			.replace("[[build_type]]", &profile)
			.replace("[[pkg_name]]", env!("CARGO_PKG_NAME"))
			.replace("[[binary_name]]", binary);

		let mapping = mapping_template.replace("[[binary_name]]", binary);
		
		let path = html_target_dir.join(format!("{}.html", binary));
		let mut html_file = File::create(&path).unwrap();

		html_file.write_all(index_html.as_bytes()).unwrap();
		mappings_file.write_all(mapping.as_bytes()).unwrap();
	}

	if profile == "debug" {
		println!("cargo:rustc-cfg=debug");
	}

	// println!("cargo:rustc-cfg=dom_console");
}
