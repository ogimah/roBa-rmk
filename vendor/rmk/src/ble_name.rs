//! A standard Vial Text macro transports a name; only User18 applies it.
//! Kept separate from macro execution and from USB/device identity.
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

pub(crate) const APPLY_USER: u8 = 18;
const PREFIX: &[u8] = b"BLE_NAME=";
const MAX_LEN: usize = 16; // Space remaining in the existing 31-byte advertisement.
static MACRO_EDITED: AtomicBool = AtomicBool::new(false);
static LAST_MACRO_EDIT: AtomicU32 = AtomicU32::new(0);

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    postcard::experimental::max_size::MaxSize,
)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct BleName {
    bytes: [u8; MAX_LEN],
    len: u8,
}

impl BleName {
    fn new(text: &[u8]) -> Result<Self, NameError> {
        if text.is_empty()
            || text.len() > MAX_LEN
            || !text.iter().all(|b| (0x20..=0x7e).contains(b))
            || text.iter().all(|b| *b == b' ')
        {
            return Err(NameError::Invalid);
        }
        let mut bytes = [0; MAX_LEN];
        bytes[..text.len()].copy_from_slice(text);
        Ok(Self {
            bytes,
            len: text.len() as u8,
        })
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        let bytes = self.bytes.get(..usize::from(self.len))?;
        Self::new(bytes).ok()?;
        core::str::from_utf8(bytes).ok()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NameError {
    Missing,
    Duplicate,
    Invalid,
    Truncated,
}

pub(crate) fn from_macro_buffer(buffer: &[u8]) -> Result<BleName, NameError> {
    let mut found = None;
    let mut start = 0;
    // The existing VIA MacroGetCount advertises 32 slots. No slot is reserved.
    for _ in 0..32 {
        let rest = buffer.get(start..).ok_or(NameError::Truncated)?;
        let end = rest
            .iter()
            .position(|b| *b == 0)
            .ok_or(NameError::Truncated)?;
        let frame = &rest[..end];
        if let Some(text) = frame.strip_prefix(PREFIX) {
            if found.is_some() {
                return Err(NameError::Duplicate);
            }
            found = Some(BleName::new(text)?);
        }
        start += end + 1;
    }
    found.ok_or(NameError::Missing)
}

pub(crate) fn macro_written() {
    LAST_MACRO_EDIT.store(
        embassy_time::Instant::now().as_millis() as u32,
        Ordering::Relaxed,
    );
    MACRO_EDITED.store(true, Ordering::Release);
}

pub(crate) fn macro_ready() -> bool {
    !MACRO_EDITED.load(Ordering::Acquire)
        || (embassy_time::Instant::now().as_millis() as u32)
            .wrapping_sub(LAST_MACRO_EDIT.load(Ordering::Relaxed))
            >= 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    fn buffer(frames: &[&[u8]]) -> [u8; 512] {
        let mut out = [0; 512];
        let mut at = 0;
        for frame in frames {
            out[at..at + frame.len()].copy_from_slice(frame);
            at += frame.len() + 1;
        }
        out
    }
    #[test]
    fn name_text_preserves_ascii_case_punctuation_and_any_slot() {
        for slot in [0, 7, 31] {
            let mut frames = vec![&b"ordinary"[..]; 32];
            frames[slot] = b"BLE_NAME=moNa2-COROPIT";
            let b = buffer(&frames);
            assert_eq!(
                from_macro_buffer(&b).unwrap().as_str(),
                Some("moNa2-COROPIT")
            );
            assert_eq!(b, buffer(&frames), "parser must not modify macro data");
        }
        assert_eq!(
            from_macro_buffer(&buffer(&[b"BLE_NAME=A_1-+ ()"]))
                .unwrap()
                .as_str(),
            Some("A_1-+ ()")
        );
    }
    #[test]
    fn name_parser_rejects_ambiguous_mixed_and_non_ascii_data() {
        for text in [
            &b"BLE_NAME="[..],
            b"BLE_NAME=                 ",
            b"BLE_NAME=12345678901234567",
            b"BLE_NAME=x\x01\x01A",
            "BLE_NAME=モナ".as_bytes(),
            b"BLE_NAME=x\n",
        ] {
            assert_eq!(from_macro_buffer(&buffer(&[text])), Err(NameError::Invalid));
        }
        assert_eq!(
            from_macro_buffer(&buffer(&[b"BLE_NAME=A", b"BLE_NAME=B"])),
            Err(NameError::Duplicate)
        );
        assert_eq!(
            from_macro_buffer(&buffer(&[b"text BLE_NAME=A", b"\x01\x01A"])),
            Err(NameError::Missing)
        );
        assert_eq!(
            from_macro_buffer(b"BLE_NAME=unfinished"),
            Err(NameError::Truncated)
        );
        assert_eq!(
            from_macro_buffer(&buffer(&[b"BLE_NAME=1234567890123456"]))
                .unwrap()
                .as_str()
                .unwrap()
                .len(),
            16
        );
    }
    #[test]
    fn corrupt_saved_name_is_rejected_without_panicking() {
        assert_eq!(
            BleName {
                bytes: [b'A'; 16],
                len: 17
            }
            .as_str(),
            None
        );
        assert_eq!(
            BleName {
                bytes: [0xff; 16],
                len: 1
            }
            .as_str(),
            None
        );
    }
    #[test]
    fn partial_macro_transfer_needs_a_quiet_interval() {
        embassy_time::MockDriver::get().reset();
        assert!(macro_ready());
        macro_written();
        assert!(!macro_ready());
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(999));
        assert!(!macro_ready());
        macro_written();
        embassy_time::MockDriver::get().advance(embassy_time::Duration::from_millis(1000));
        assert!(macro_ready());
    }
}
