/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::sync::mpsc::Receiver;
use std::time::Instant;

use dcommon::ui::events::{Action, Event, Modifiers, MouseButton};
use gl21 as gl;
use glfw::{Context, Glfw, Window, WindowEvent};
use image::{ImageError, RgbaImage};
use imgui::{Condition, TextureId, WindowFlags};

use imgui_support::App;

use crate::keymap::to_imgui_key;
use crate::platform::Platform;
use crate::renderer::{bind_texture, render, Renderer};
pub use crate::utils::get_screen_bounds;

mod keymap;
mod platform;
mod renderer;
mod utils;

pub struct System {
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,
    imgui: imgui::Context,
    platform: Platform,
    _renderer: Renderer,
    last_frame_time: Instant,
    app: Box<dyn App>,
}

#[must_use]
pub fn init<A: App + 'static>(
    mut glfw: Glfw,
    title: &'static str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    app: A,
) -> System {
    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw
        .create_window(width, height, title, glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    #[allow(clippy::cast_possible_wrap)]
    {
        window.set_pos(x as _, y as _);
    }

    // Make the window's context current
    window.make_current();
    window.set_all_polling(true);

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    let mut platform = Platform::init(&mut imgui);

    platform.attach_window(imgui.io_mut(), &window);

    let renderer = Renderer::new(&mut imgui);

    System {
        glfw,
        window,
        events,
        imgui,
        platform,
        _renderer: renderer,
        last_frame_time: Instant::now(),
        app: Box::new(app),
    }
}

/// # Errors
///
/// Returns `ImageError` if the image could not be loaded.
pub fn create_texture(image: &RgbaImage) -> Result<TextureId, ImageError> {
    let texture_id = bind_texture();
    imgui_support::create_texture(texture_id, image)
}

impl System {
    pub fn main_loop(&mut self) {
        let System {
            glfw,
            window,
            events,
            platform,
            mut last_frame_time,
            ..
        } = self;
        while !window.should_close() {
            glfw.wait_events_timeout(0.1);
            for (_timestamp, event) in events.try_iter() {
                let mut consumed = false;
                if let Some(app_event) = from_event(&event) {
                    consumed = self.app.handle_event(app_event);
                }
                if !consumed {
                    platform.handle_event(self.imgui.io_mut(), window, &event);
                }
            }

            let now = Instant::now();
            self.imgui.io_mut().update_delta_time(now - last_frame_time);
            last_frame_time = now;

            self.imgui.style_mut().window_padding = [0.0, 0.0];
            let display_size = self.imgui.io().display_size;

            let ui = self.imgui.new_frame();
            ui.window("ImGui Window")
                .position([0.0, 0.0], Condition::Always)
                .size(display_size, Condition::Always)
                .flags(
                    WindowFlags::NO_BACKGROUND
                        | WindowFlags::NO_DECORATION
                        | WindowFlags::NO_INPUTS,
                )
                .build(|| self.app.draw_ui(ui));

            unsafe {
                gl::ClearColor(0.2, 0.2, 0.2, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            render(&mut self.imgui);

            // Swap front and back buffers
            window.swap_buffers();
        }
    }
}

fn from_event(event: &WindowEvent) -> Option<Event> {
    #[allow(clippy::cast_possible_truncation)]
    match *event {
        WindowEvent::MouseButton(button, action, _) => {
            let action = to_common_action(action);
            if let Some(action) = action {
                let button = match button {
                    glfw::MouseButton::Button1 => Some(MouseButton::Left),
                    glfw::MouseButton::Button2 => Some(MouseButton::Right),
                    _ => None,
                };
                button.map(|button| Event::MouseButton(button, action))
            } else {
                None
            }
        }
        WindowEvent::CursorPos(x, y) => Some(Event::CursorPos(x as _, y as _)),
        WindowEvent::Scroll(x, y) => Some(Event::Scroll(x as _, y as _)),
        WindowEvent::Key(key, _scancode, action, modifiers) => match to_common_action(action) {
            Some(action) => {
                let key = to_imgui_key(key);
                let modifiers = Modifiers {
                    control: modifiers & glfw::Modifiers::Control != glfw::Modifiers::empty(),
                    option: modifiers & glfw::Modifiers::Alt != glfw::Modifiers::empty(),
                    shift: modifiers & glfw::Modifiers::Shift != glfw::Modifiers::empty(),
                };
                Some(Event::Key(key, '\u{0}', action, modifiers))
            }
            None => None,
        },
        _ => None,
    }
}

fn to_common_action(action: glfw::Action) -> Option<Action> {
    match action {
        glfw::Action::Release => Some(Action::Release),
        glfw::Action::Press => Some(Action::Press),
        glfw::Action::Repeat => None,
    }
}
