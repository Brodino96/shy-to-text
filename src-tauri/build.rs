use std::fs;

fn main() {
    let pkg: String = fs::read_to_string("../package.json").unwrap();
    let json: serde_json::Value = serde_json::from_str(&pkg).unwrap();
    let version = json["version"].as_str().unwrap();
    println!("cargo:rustc-env=PKG_VERSION={}", version);
    println!("cargo:rerun-if-changed=../package.json");
    tauri_build::build()
}
