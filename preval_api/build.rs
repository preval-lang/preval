use cbindgen::{Config, ExportConfig};
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let output_file = crate_dir.clone() + "/../preval.h";

    let config = Config {
        language: cbindgen::Language::C,
        cpp_compat: true,
        export: ExportConfig {
            include: vec!["API".to_string()],
            ..Default::default()
        },
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}
