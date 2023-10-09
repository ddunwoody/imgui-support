/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

// based on https://github.com/aloucks/imgui-glfw-support
#![allow(clippy::pedantic)]

//! Provides a glfw-based backend platform for imgui-rs. This crate is modeled
//! after the winit version.
//!
//! ## Usage
//!
//! 1. Initialize a `GlfwPlatform`
//! 2. Attach it to a glfw `Window`
//! 3. Optionally, enable platform clipboard integration
//! 4. Pass events to the platform (every frame)
//! 5. Call frame preparation (every frame)
//! 6. Call render preperation (every frame)
//!
//! ## Examples
//!
//! The [examples](https://github.com/aloucks/imgui-glfw-support/tree/master/examples) can be found on github.

use crate::keymap::to_imgui_key;
use glfw::{Action, Window, WindowEvent};
use imgui::{Context, Io, Key, MouseButton};

pub struct Platform;

impl Platform {
    /// Initializes a glfw platform instance and configures imgui.
    ///
    /// * backend flgs are updated
    /// * keys are configured
    /// * platform name is set
    pub fn init(imgui: &mut Context) -> Platform {
        imgui.set_platform_name(Some(format!(
            "imgui-standalone-platform {}",
            env!("CARGO_PKG_VERSION")
        )));

        Platform {}
    }

    /// Attaches the platform instance to a glfw window.
    ///
    /// * framebuffer scale (i.e. DPI factor) is set
    /// * display size is set
    pub fn attach_window(&mut self, io: &mut Io, window: &Window) {
        let (scale_factor_x, _scale_factor_y) = window.get_content_scale();
        let hidpi_factor = scale_factor_x.round();
        io.display_framebuffer_scale = [hidpi_factor, hidpi_factor];
        let (width, height) = window.get_size();
        io.display_size = [width as f32, height as f32];
    }

    /// Handles a glfw window event
    ///
    /// * keyboard state is updated
    /// * mouse state is updated
    pub fn handle_event(&self, io: &mut Io, _window: &Window, event: &WindowEvent) {
        match *event {
            WindowEvent::Key(key, _scancode, action, _modifiers) => {
                let pressed = match action {
                    Action::Release => Some(false),
                    Action::Press => Some(true),
                    Action::Repeat => None,
                };
                if let Some(pressed) = pressed {
                    if let Some(key) = to_imgui_key(key) {
                        io.add_key_event(key, pressed);
                    }

                    if key == glfw::Key::LeftShift || key == glfw::Key::RightShift {
                        io.add_key_event(Key::ModShift, pressed);
                    }

                    if key == glfw::Key::LeftControl || key == glfw::Key::RightControl {
                        io.add_key_event(Key::ModCtrl, pressed);
                    }

                    if key == glfw::Key::LeftAlt || key == glfw::Key::RightAlt {
                        io.add_key_event(Key::ModAlt, pressed);
                    }

                    if key == glfw::Key::LeftSuper || key == glfw::Key::RightSuper {
                        io.add_key_event(Key::ModSuper, pressed);
                    }
                }
            }
            WindowEvent::Size(width, height) => {
                io.display_size = [width as _, height as _];
            }
            WindowEvent::Char(ch) => {
                // Exclude the backspace key
                if ch != '\u{7f}' {
                    io.add_input_character(ch);
                }
            }
            WindowEvent::CursorPos(x, y) => {
                io.add_mouse_pos_event([x as _, y as _]);
            }
            WindowEvent::Scroll(x, y) => {
                io.add_mouse_wheel_event([x as _, y as _]);
            }
            WindowEvent::MouseButton(button, action, _modifiers) => {
                let button = match button {
                    glfw::MouseButton::Button1 => MouseButton::Left,
                    glfw::MouseButton::Button2 => MouseButton::Right,
                    glfw::MouseButton::Button3 => MouseButton::Middle,
                    glfw::MouseButton::Button4 => MouseButton::Extra1,
                    _ => MouseButton::Extra2,
                };
                io.add_mouse_button_event(button, action == Action::Press);
            }
            _ => {}
        }
    }
}
