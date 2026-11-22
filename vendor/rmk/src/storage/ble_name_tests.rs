mod ble_name_tests {
    use super::*;
    use crate::{
        event::KeyboardEvent,
        host::HostService,
        keyboard::Keyboard,
        keymap::{KeyMap, KeymapData},
    };
    use rmk_types::action::Action;

    fn bytes(v: &StorageData) -> std::vec::Vec<u8> {
        let mut buffer = [0; 2048];
        postcard::to_slice(v, &mut buffer).unwrap().to_vec()
    }

    async fn packet(host: &HostService<'_>, data: &[u8]) {
        let mut request = [0; 32];
        request[..data.len()].copy_from_slice(data);
        let mut source = &request[..];
        let mut response = [0; 32];
        let mut sink = &mut response[..];
        host.run_session(&mut source, &mut sink).await;
        assert!(sink.is_empty());
        assert_ne!(response[0], 0xff);
    }

    async fn upload(host: &HostService<'_>, buffer: &[u8]) {
        for (chunk, data) in buffer.chunks(28).enumerate() {
            let offset = chunk * 28;
            let mut request = [0; 32];
            request[..4].copy_from_slice(&[
                0x0f,
                (offset >> 8) as u8,
                offset as u8,
                data.len() as u8,
            ]);
            request[4..4 + data.len()].copy_from_slice(data);
            packet(host, &request).await;
        }
    }

    #[test]
    fn ble_name_vial_upload_apply_real_flash_reboot_and_legacy_bonds_survive() {
        use crate::ble::profile::ProfileInfo;
        use crate::config::PositionalConfig;
        use crate::core_traits::Runnable;
        use crate::split::ble::PeerAddress;
        block_on(async {
            FLASH_CHANNEL.clear();
            BLE_NAME_RESPONSE.reset();
            BLE_NAME_SAVED.reset();
            type Flash = TestFlash<65536, 4096, 4>;
            let mut initial = [[[KeyAction::No; 11]; 5]; 8];
            initial[0][0][2] = KeyAction::Single(Action::User(14));
            let mut encoder_map = [[EncoderAction::new(
                KeyAction::Single(Action::User(13)),
                KeyAction::No,
            )]; 8];
            let encoders = Some(&mut encoder_map);
            let config = RuntimeStorageConfig {
                num_sectors: 8,
                ..Default::default()
            };
            let mut storage = Storage::<Flash, 5, 11, 8, 1>::new(
                Flash::new(),
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            let mut bond = ProfileInfo::default();
            bond.slot_num = 2;
            bond.info.ltk = trouble_host::LongTermKey(0x12345678);
            let mut macros = [0; MACRO_SPACE_SIZE];
            macros[..5].copy_from_slice(b"hello");
            let seeds = [
                (
                    StorageKey::keymap(0, 0, 2),
                    StorageData::KeyAction(initial[0][0][2]),
                ),
                (
                    StorageKey::encoder(0, 0),
                    StorageData::EncoderAction(EncoderAction::new(
                        KeyAction::Single(Action::User(13)),
                        KeyAction::No,
                    )),
                ),
                (
                    StorageKey::LayoutConfig,
                    StorageData::LayoutConfig(LayoutConfig {
                        default_layer: 0,
                        layout_option: 1,
                    }),
                ),
                (StorageKey::MacroData, StorageData::MacroData(macros)),
                (StorageKey::BondInfo(2), StorageData::BondInfo(bond)),
                (
                    StorageKey::ActiveBleProfile,
                    StorageData::ActiveBleProfile(2),
                ),
                (
                    StorageKey::ConnectionType,
                    StorageData::ConnectionType(ConnectionType::Usb),
                ),
                (
                    StorageKey::PeerAddress(0),
                    StorageData::PeerAddress(PeerAddress::new(0, true, [1, 2, 3, 4, 5, 6])),
                ),
                (StorageKey::Fork(31), StorageData::Fork(Fork::default())),
                (
                    StorageKey::Morse(31),
                    StorageData::Morse(Morse::new_from_vial(
                        Action::User(10),
                        Action::No,
                        Action::No,
                        Action::No,
                        MorseProfile::default(),
                    )),
                ),
                (
                    StorageKey::Combo(7),
                    StorageData::Combo(ComboConfig::new(
                        [KeyAction::Single(Action::User(11))],
                        KeyAction::Single(Action::User(12)),
                        None,
                    )),
                ),
            ];
            let mut preserved = std::vec::Vec::new();
            for (key, value) in seeds {
                preserved.push((key, bytes(&value)));
                storage.store_data(key, &value).await.unwrap();
            }
            storage
                .store_data(
                    StorageKey::StorageConfig,
                    &StorageData::StorageConfig(LocalStorageConfig {
                        enable: true,
                        build_hash: BUILD_HASH.wrapping_sub(1),
                    }),
                )
                .await
                .unwrap();
            let (flash, _) = storage.flash.destroy();
            // Same source schema, different build: old bank must be migrated, not erased.
            let mut storage = Storage::<Flash, 5, 11, 8, 1>::new(
                flash,
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            for (key, expected) in &preserved {
                assert_eq!(
                    bytes(&storage.fetch_data(*key).await.unwrap()),
                    *expected,
                    "upgrade changed {key:?}"
                );
            }
            let mut data = KeymapData::new_with_encoder(
                [[[KeyAction::No; 11]; 5]; 8],
                [[EncoderAction::new(KeyAction::No, KeyAction::No)]; 8],
            );
            let mut behavior = RuntimeBehaviorConfig::default();
            let positional = PositionalConfig::default();
            let keymap =
                KeyMap::new_from_storage(&mut data, Some(&mut storage), &mut behavior, &positional)
                    .await;
            assert_eq!(keymap.layout_option(), 1);
            assert_eq!(keymap.action_at_pos(0, 0, 2), initial[0][0][2]);
            let host = HostService::new(&keymap, &crate::config::RmkConfig::default());
            let mut keyboard = Keyboard::new(&keymap);
            // Place the new User18 through actual standard VIA keymap communication.
            let sequence = async {
                packet(&host, &[0x05, 0, 0, 0, 0x7e, 0x12]).await;
                assert_eq!(
                    keymap.action_at_pos(0, 0, 0),
                    KeyAction::Single(Action::User(18))
                );
                macros[7..7 + b"name=roBa_Name1".len()].copy_from_slice(b"name=roBa_Name1");
                upload(&host, &macros).await;
                embassy_time::Timer::after_millis(1100).await;
                keyboard.process_inner(KeyboardEvent::key(0, 0, true)).await;
                keyboard
                    .process_inner(KeyboardEvent::key(0, 0, false))
                    .await;
                assert_eq!(read_ble_name().await.unwrap().as_str(), Some("roBa_Name1"));
                assert_eq!(
                    keymap.get_macro_sequences(),
                    macros,
                    "applying a name modified ordinary macros"
                );
                // Invalid apply must not erase the saved value.
                let mut invalid = [0; MACRO_SPACE_SIZE];
                invalid[..5].copy_from_slice(b"name=");
                upload(&host, &invalid).await;
                embassy_time::Timer::after_millis(1100).await;
                keyboard.process_inner(KeyboardEvent::key(0, 0, true)).await;
                keyboard
                    .process_inner(KeyboardEvent::key(0, 0, false))
                    .await;
                assert_eq!(read_ble_name().await.unwrap().as_str(), Some("roBa_Name1"));
                // An ordinary macro restore/deletion is not a name command.
                upload(&host, &[0; MACRO_SPACE_SIZE]).await;
                assert_eq!(read_ble_name().await.unwrap().as_str(), Some("roBa_Name1"));
            };
            assert!(matches!(
                embassy_futures::select::select(storage.run(), sequence).await,
                embassy_futures::select::Either::Second(())
            ));
            let (flash, _) = storage.flash.destroy();
            let mut rebooted = Storage::<Flash, 5, 11, 8, 1>::new(
                flash,
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            assert_eq!(
                match rebooted.fetch_data(StorageKey::BleName).await.unwrap() {
                    StorageData::BleName(n) => n,
                    _ => panic!("missing name"),
                }
                .as_str(),
                Some("roBa_Name1")
            );
            for (key, expected) in preserved
                .iter()
                .filter(|(key, _)| *key != StorageKey::MacroData)
            {
                assert_eq!(
                    bytes(&rebooted.fetch_data(*key).await.unwrap()),
                    *expected,
                    "name apply changed {key:?}"
                );
            }
            let usb_defaults = crate::config::DeviceConfig {
                product_name: "compiled default",
                serial_number: "stable-serial",
                ..Default::default()
            };
            let usb = rebooted.usb_config_with_saved_name(usb_defaults).await;
            assert_eq!(usb.product_name, "roBa_Name1");
            assert_eq!(
                (usb.vid, usb.pid, usb.manufacturer, usb.serial_number),
                (
                    usb_defaults.vid,
                    usb_defaults.pid,
                    usb_defaults.manufacturer,
                    usb_defaults.serial_number
                )
            );
            let boot = async {
                assert_eq!(read_ble_name().await.unwrap().as_str(), Some("roBa_Name1"));
            };
            assert!(matches!(
                embassy_futures::select::select(rebooted.run(), boot).await,
                embassy_futures::select::Either::Second(())
            ));
        });
    }

    #[test]
    fn ble_name_usb_boot_preserves_old_saved_encoding_and_missing_name_defaults() {
        block_on(async {
            type Flash = TestFlash<65536, 4096, 4>;
            let initial = [[[KeyAction::No]]];
            let encoders: Option<&mut [[EncoderAction; 0]; 1]> = None;
            let config = RuntimeStorageConfig {
                num_sectors: 8,
                ..Default::default()
            };
            let mut storage = Storage::<Flash, 1, 1, 1, 0>::new(
                Flash::new(),
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            let default = crate::config::DeviceConfig {
                product_name: "fixed",
                serial_number: "serial-unchanged",
                ..Default::default()
            };
            let fallback = storage.usb_config_with_saved_name(default).await;
            assert_eq!(fallback.product_name, default.product_name);
            // Exact previous name record: appended variant 13, [u8;16], len.
            // Do not generate this through the new name= parser.
            let mut legacy_wire = [0; 18];
            legacy_wire[0] = 13;
            legacy_wire[1..4].copy_from_slice(b"Old");
            legacy_wire[17] = 3;
            let legacy: StorageData = postcard::from_bytes(&legacy_wire).unwrap();
            assert_eq!(bytes(&legacy), legacy_wire);
            storage
                .store_data(StorageKey::BleName, &legacy)
                .await
                .unwrap();
            let mut old_macro = [0; 512];
            old_macro[..12].copy_from_slice(b"BLE_NAME=Old");
            assert_eq!(
                crate::ble_name::from_macro_buffer(&old_macro),
                Err(crate::ble_name::NameError::Missing)
            );
            let usb = storage.usb_config_with_saved_name(default).await;
            assert_eq!(usb.product_name, "Old");
            assert_eq!(
                (usb.vid, usb.pid, usb.manufacturer, usb.serial_number),
                (
                    default.vid,
                    default.pid,
                    default.manufacturer,
                    default.serial_number
                )
            );
        });
    }

    #[test]
    fn ble_name_storage_key_and_value_discriminants_are_append_only() {
        let mut buffer = [0; 512];
        for (key, index) in [
            (StorageKey::MacroData, 4),
            (
                StorageKey::Keymap {
                    layer: 0,
                    row: 0,
                    col: 0,
                },
                5,
            ),
            (StorageKey::PeerAddress(0), 10),
            (StorageKey::ActiveBleProfile, 11),
            (StorageKey::BondInfo(0), 12),
            (StorageKey::BleName, 13),
        ] {
            let encoded = postcard::to_slice(&key, &mut buffer).unwrap();
            assert_eq!(encoded[0], index);
        }
        assert_eq!(
            bytes(&StorageData::BondInfo(
                crate::ble::profile::ProfileInfo::default()
            ))[0],
            11
        );
        assert_eq!(bytes(&StorageData::ActiveBleProfile(0))[0], 12);
    }

    #[test]
    fn ble_name_flash_write_failure_reports_failure_and_keeps_previous_name() {
        use crate::core_traits::Runnable;
        block_on(async {
            FLASH_CHANNEL.clear();
            BLE_NAME_SAVED.reset();
            BLE_NAME_RESPONSE.reset();
            type Flash = TestFlash<65536, 4096, 4>;
            let initial = [[[KeyAction::No]]];
            let encoders: Option<&mut [[EncoderAction; 0]; 1]> = None;
            let config = RuntimeStorageConfig {
                num_sectors: 8,
                ..Default::default()
            };
            let mut storage = Storage::<Flash, 1, 1, 1, 0>::new(
                Flash::new(),
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            let old = crate::ble_name::from_macro_buffer(&{
                let mut b = [0; 512];
                b[..8].copy_from_slice(b"name=Old");
                b
            })
            .unwrap();
            storage
                .store_data(StorageKey::BleName, &StorageData::BleName(old))
                .await
                .unwrap();
            let (mut flash, _) = storage.flash.destroy();
            flash.fail_writes = true;
            let mut storage = Storage::<Flash, 1, 1, 1, 0>::new(
                flash,
                &initial,
                &encoders,
                &config,
                &RuntimeBehaviorConfig::default(),
            )
            .await;
            let new = crate::ble_name::from_macro_buffer(&{
                let mut b = [0; 512];
                b[..8].copy_from_slice(b"name=New");
                b
            })
            .unwrap();
            let attempt = async {
                assert!(!save_ble_name(new).await);
                assert_eq!(read_ble_name().await, Some(old));
            };
            assert!(matches!(
                embassy_futures::select::select(storage.run(), attempt).await,
                embassy_futures::select::Either::Second(())
            ));
        });
    }
}
