include!(concat!(env!("OUT_DIR"), "/battery_config.rs"));

#[test]
fn actual_rmk_config_exposes_the_left_battery_through_the_central() {
    assert_eq!(CENTRAL_DESCRIPTION, "Right");
    assert_eq!(PERIPHERAL_IDS, &[0]);
    assert_eq!(PERIPHERAL_DESCRIPTIONS, &["Left"]);
}
