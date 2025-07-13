use std::ffi::c_void;

use core_foundation::base::CFRelease;
use core_foundation::data::{CFDataGetBytePtr, CFDataRef};
use objc2::rc::Retained;
use objc2_foundation::{run_on_main, MainThreadMarker, NSString};
use objc2_ui_kit::{UIKey, UIKeyModifierFlags, UIPress, UIPressPhase, UIPressType};
use smol_str::SmolStr;

use crate::event::{ElementState, KeyEvent, Modifiers};
use crate::keyboard::{
    Key, KeyCode, KeyLocation, ModifiersKeys, ModifiersState, NamedKey, NativeKey, NativeKeyCode,
    PhysicalKey,
};
use crate::platform_impl::KeyEventExtra;

/// Ignores ALL modifiers.
pub fn get_modifierless_char(key: &UIKey) -> Option<Key> {
    let char = unsafe { key.charactersIgnoringModifiers() };

    let string = char.to_string();
    if string.is_empty() {
        return None;
    }

    return Some(Key::Character(SmolStr::new(string)));
}

/// Create `KeyEvent` for the given `UIPress`.
pub(crate) fn create_key_event(ui_press: &UIPress, mtm: MainThreadMarker) -> KeyEvent {
    use ElementState::{Pressed, Released};
    let phase = unsafe { ui_press.phase() };
    let state = match phase {
        UIPressPhase::Began | UIPressPhase::Changed => Pressed,
        UIPressPhase::Ended | UIPressPhase::Cancelled => Released,
        _ => Pressed, // this should not happen, but we default to Pressed
    };

    let key = unsafe { ui_press.key(mtm) };

    let (logical_key, physical_key, location, text) = if let Some(key) = key {
        let keycode = unsafe { key.keyCode() }.0 as i64;

        let logical_key = get_modifierless_char(&key)
            .unwrap_or_else(|| Key::Unidentified(NativeKey::IOS(keycode)));
        let physical_key = PhysicalKey::Unidentified(NativeKeyCode::IOS(keycode));

        // For now, we assume the location is standard
        let location = KeyLocation::Standard;

        // Get the text representation of the key
        let text = None;

        (logical_key, physical_key, location, text)
    } else {
        (
            Key::Dead(None),
            PhysicalKey::Unidentified(NativeKeyCode::IOS(0)),
            KeyLocation::Standard,
            None,
        )
    };

    KeyEvent {
        location,
        logical_key,
        physical_key,
        repeat: false,
        state,
        text,
        platform_specific: KeyEventExtra {},
    }
}
