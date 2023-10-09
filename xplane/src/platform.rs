/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use std::primitive;

use imgui::{sys, Context, Io, Key, MouseButton};
use xplm::data::borrowed::{DataRef, FindError};
use xplm::data::DataRead;
use xplm_ext::ui::Window;

use imgui_support::events;
use imgui_support::events::{Action, Event, Modifiers};
use imgui_support::geometry::Rect;

pub struct Platform {
    frame_rate_period: DataRef<f32>,
}

impl Platform {
    pub fn init(imgui: &mut Context) -> Result<Platform, FindError> {
        imgui.set_platform_name(Some(format!(
            "imgui-xplane-platform {}",
            env!("CARGO_PKG_VERSION")
        )));

        let io = imgui.io_mut();
        io.config_mac_os_behaviors = false;

        Ok(Platform {
            frame_rate_period: DataRef::find("sim/operation/misc/frame_rate_period")?,
        })
    }

    pub fn prepare_frame(&self, io: &mut Io, window: &mut Window) {
        io.display_framebuffer_scale = [1.0, 1.0];

        let geometry = window.geometry();
        #[allow(clippy::cast_precision_loss)]
        {
            io.display_size = geometry.into();
        }

        let frame_rate_period = self.frame_rate_period.get();
        if frame_rate_period <= 0.0 {
            io.delta_time = 1.0 / 60.0;
        } else {
            io.delta_time = frame_rate_period;
        }

        let has_keyboard_focus = window.has_keyboard_focus();

        if io.want_capture_keyboard && !has_keyboard_focus {
            window.take_keyboard_focus();
        } else if !io.want_capture_keyboard && has_keyboard_focus {
            window.release_keyboard_focus();
            // lift all keys
            io.keys_down = [false; sys::ImGuiKey_COUNT as usize];
            io.add_key_event(Key::ModCtrl, false);
            io.add_key_event(Key::ModAlt, false);
            io.add_key_event(Key::ModShift, false);
        }
    }
}

pub fn handle_event(io: &mut Io, window: &Window, event: Event) {
    match event {
        Event::Key(key, ch, action, modifiers) => {
            let pressed = action == Action::Press;
            if let Some(key) = key {
                io.add_key_event(key, pressed);
            }

            let Modifiers {
                control,
                option,
                shift,
            } = modifiers;

            if pressed && !control && !option && ch != '\u{7f}' {
                io.add_input_character(ch);
            }

            io.add_key_event(Key::ModCtrl, control);
            io.add_key_event(Key::ModAlt, option);
            io.add_key_event(Key::ModShift, shift);
        }
        Event::CursorPos(x, y) => {
            let (x, y) = translate_to_imgui_space(window, x, y);
            io.add_mouse_pos_event([x as _, y as _]);
        }
        Event::Scroll(x, y) => {
            #[allow(clippy::cast_precision_loss)]
            io.add_mouse_wheel_event([x as _, y as _]);
        }
        Event::MouseButton(button, action) => {
            let button = match button {
                events::MouseButton::Left => MouseButton::Left,
                events::MouseButton::Right => MouseButton::Right,
            };
            io.add_mouse_button_event(button, action != Action::Release);
        }
    }
}

#[allow(clippy::cast_precision_loss)]
fn translate_to_imgui_space(window: &Window, x: i32, y: i32) -> (f32, f32) {
    let Rect {
        left,
        top,
        right,
        bottom,
    } = window.geometry();

    let out_x = x - left;
    if out_x < 0 || out_x > right - left {
        return (primitive::f32::MIN, primitive::f32::MIN);
    }

    let out_y = top - y;
    if out_y < 0 || out_y > top - bottom {
        return (primitive::f32::MIN, primitive::f32::MIN);
    }
    (out_x as f32, out_y as f32)
}
