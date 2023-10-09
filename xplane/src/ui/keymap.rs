/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![allow(clippy::wildcard_imports)]

use std::ffi::c_char;

use imgui::Key;
use xplm_sys::*;

pub fn to_imgui_key(key: c_char) -> Option<Key> {
    #[allow(clippy::cast_sign_loss)]
    match key as u32 {
        XPLM_VK_TAB => Some(Key::Tab),
        XPLM_VK_LEFT => Some(Key::LeftArrow),
        XPLM_VK_RIGHT => Some(Key::RightArrow),
        XPLM_VK_UP => Some(Key::UpArrow),
        XPLM_VK_DOWN => Some(Key::DownArrow),
        XPLM_VK_PRIOR => Some(Key::PageUp),
        XPLM_VK_NEXT => Some(Key::PageDown),
        XPLM_VK_HOME => Some(Key::Home),
        XPLM_VK_END => Some(Key::End),
        XPLM_VK_INSERT => Some(Key::Insert),
        XPLM_VK_DELETE => Some(Key::Delete),
        XPLM_VK_BACK => Some(Key::Backspace),
        XPLM_VK_SPACE => Some(Key::Space),
        XPLM_VK_RETURN => Some(Key::Enter),
        XPLM_VK_ESCAPE => Some(Key::Escape),

        XPLM_VK_0 => Some(Key::Alpha0),
        XPLM_VK_1 => Some(Key::Alpha1),
        XPLM_VK_2 => Some(Key::Alpha2),
        XPLM_VK_3 => Some(Key::Alpha3),
        XPLM_VK_4 => Some(Key::Alpha4),
        XPLM_VK_5 => Some(Key::Alpha5),
        XPLM_VK_6 => Some(Key::Alpha6),
        XPLM_VK_7 => Some(Key::Alpha7),
        XPLM_VK_8 => Some(Key::Alpha8),
        XPLM_VK_9 => Some(Key::Alpha9),

        XPLM_VK_A => Some(Key::A),
        XPLM_VK_B => Some(Key::B),
        XPLM_VK_C => Some(Key::C),
        XPLM_VK_D => Some(Key::D),
        XPLM_VK_E => Some(Key::E),
        XPLM_VK_F => Some(Key::F),
        XPLM_VK_G => Some(Key::G),
        XPLM_VK_H => Some(Key::H),
        XPLM_VK_I => Some(Key::I),
        XPLM_VK_J => Some(Key::J),
        XPLM_VK_K => Some(Key::K),
        XPLM_VK_L => Some(Key::L),
        XPLM_VK_M => Some(Key::M),
        XPLM_VK_N => Some(Key::N),
        XPLM_VK_O => Some(Key::O),
        XPLM_VK_P => Some(Key::P),
        XPLM_VK_Q => Some(Key::Q),
        XPLM_VK_R => Some(Key::R),
        XPLM_VK_S => Some(Key::S),
        XPLM_VK_T => Some(Key::T),
        XPLM_VK_U => Some(Key::U),
        XPLM_VK_V => Some(Key::V),
        XPLM_VK_W => Some(Key::W),
        XPLM_VK_X => Some(Key::X),
        XPLM_VK_Y => Some(Key::Y),
        XPLM_VK_Z => Some(Key::Z),

        XPLM_VK_F1 => Some(Key::F1),
        XPLM_VK_F2 => Some(Key::F2),
        XPLM_VK_F3 => Some(Key::F3),
        XPLM_VK_F4 => Some(Key::F4),
        XPLM_VK_F5 => Some(Key::F5),
        XPLM_VK_F6 => Some(Key::F6),
        XPLM_VK_F7 => Some(Key::F7),
        XPLM_VK_F8 => Some(Key::F8),
        XPLM_VK_F9 => Some(Key::F9),
        XPLM_VK_F10 => Some(Key::F10),
        XPLM_VK_F11 => Some(Key::F11),
        XPLM_VK_F12 => Some(Key::F12),

        XPLM_VK_QUOTE => Some(Key::Apostrophe),
        XPLM_VK_COMMA => Some(Key::Comma),
        XPLM_VK_MINUS => Some(Key::Minus),
        XPLM_VK_PERIOD => Some(Key::Period),
        XPLM_VK_SLASH => Some(Key::Slash),
        XPLM_VK_SEMICOLON => Some(Key::Semicolon),
        XPLM_VK_EQUAL => Some(Key::Equal),
        XPLM_VK_LBRACE => Some(Key::LeftBracket),
        XPLM_VK_BACKSLASH => Some(Key::Backslash),
        XPLM_VK_RBRACE => Some(Key::RightBracket),
        XPLM_VK_BACKQUOTE => Some(Key::GraveAccent),

        XPLM_VK_NUMPAD0 => Some(Key::Keypad0),
        XPLM_VK_NUMPAD1 => Some(Key::Keypad1),
        XPLM_VK_NUMPAD2 => Some(Key::Keypad2),
        XPLM_VK_NUMPAD3 => Some(Key::Keypad3),
        XPLM_VK_NUMPAD4 => Some(Key::Keypad4),
        XPLM_VK_NUMPAD5 => Some(Key::Keypad5),
        XPLM_VK_NUMPAD6 => Some(Key::Keypad6),
        XPLM_VK_NUMPAD7 => Some(Key::Keypad7),
        XPLM_VK_NUMPAD8 => Some(Key::Keypad8),
        XPLM_VK_NUMPAD9 => Some(Key::Keypad9),

        XPLM_VK_DECIMAL => Some(Key::KeypadDecimal),
        XPLM_VK_DIVIDE => Some(Key::KeypadDivide),
        XPLM_VK_MULTIPLY => Some(Key::KeypadMultiply),
        XPLM_VK_SUBTRACT => Some(Key::KeypadSubtract),
        XPLM_VK_ADD => Some(Key::KeypadAdd),
        XPLM_VK_NUMPAD_ENT => Some(Key::KeypadEnter),
        XPLM_VK_NUMPAD_EQ => Some(Key::KeypadEqual),
        _ => None,
    }
}
