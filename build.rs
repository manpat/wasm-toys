use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

const INDEX_HTML_TEMPLATE: &'static str = include_str!("assets/template.html");

const MAPPING_HEADER: &'static str = 
r##"
/wasm-toys/main.js => bindings/main.js [text/javascript]
/wasm-toys/input.js => bindings/input.js [text/javascript]
/wasm-toys/gl.js => bindings/gl.js [text/javascript]
"##;

const MAPPING_TEMPLATE: &'static str = 
r##"/wasm-toys/[[binary_name]] => target/html/[[binary_name]].html
/wasm-toys/[[binary_name]]-debug.wasm => target/wasm32-unknown-unknown/debug/[[binary_name]].wasm
/wasm-toys/[[binary_name]]-release.wasm => target/wasm32-unknown-unknown/release/[[binary_name]].wasm

"##;

fn read_bin_dir() -> io::Result<Option<fs::ReadDir>> {
	match fs::read_dir("src/bin") {
		Ok(dir) => Ok(Some(dir)),
		Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
		Err(err) => Err(err),
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let profile = env::var("PROFILE").unwrap();

	let html_target_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
	let html_target_dir = Path::new(&html_target_dir).join("target/html");
	std::fs::create_dir_all(&html_target_dir).unwrap();

	let mut binaries = Vec::new();

	if let Some(bin_dir) = read_bin_dir()? {
		for entry in bin_dir {
			use std::ffi::OsStr;

			let path = entry?.path();

			if path.extension() != Some(OsStr::new("rs"))
				&& !path.is_dir()
			{
				continue
			}

			if let Some(path) = path.file_stem().and_then(OsStr::to_str) {
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
		let mut html_file = File::create(&path)?;

		html_file.write_all(index_html.as_bytes())?;
		mappings_file.write_all(mapping.as_bytes())?;
	}

	if profile == "debug" {
		println!("cargo:rustc-cfg=debug");
	}

	// println!("cargo:rustc-cfg=dom_console");
	Ok(())
}
