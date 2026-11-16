fn main() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../keyboard.toml");
    println!("cargo:rerun-if-changed={}", path.display());
    let config = rmk_config::KeyboardTomlConfig::new_from_toml_path(&path);
    let constants = config.build_constants(&["split", "_ble"]).unwrap();
    let source = format!(
        "const CENTRAL_DESCRIPTION: &str = {:?};\nconst PERIPHERAL_IDS: &[usize] = &{:?};\nconst PERIPHERAL_DESCRIPTIONS: &[&str] = &{:?};\n",
        constants.central_battery_user_description,
        constants.split_battery_peripheral_ids,
        constants.split_battery_peripheral_user_descriptions,
    );
    std::fs::write(
        std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("battery_config.rs"),
        source,
    )
    .unwrap();
}
