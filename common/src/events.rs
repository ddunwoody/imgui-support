/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use imgui::Key;

#[derive(Clone, Debug)]
pub enum Event {
    MouseButton(MouseButton, Action),
    CursorPos(i32, i32),
    Scroll(i32, i32),
    Key(Option<Key>, char, Action, Modifiers),
}

#[derive(Clone, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Press,
    Release,
}

#[derive(Clone, Debug, Default)]
pub struct Modifiers {
    pub control: bool,
    pub option: bool,
    pub shift: bool,
}
