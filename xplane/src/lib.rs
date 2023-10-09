/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::cell::RefCell;
use std::rc::Rc;

use image::{ImageError, RgbaImage};
use imgui::{Condition, Context, TextureId, WindowFlags};
use xplm_ext::ui::{Decoration, Delegate, Gravity, Layer, PositioningMode, Ref, Window};

use imgui_support::events::Event;
use imgui_support::geometry::Rect;
use imgui_support::App;

use crate::platform::Platform;
use crate::renderer::{bind_texture, Renderer};
pub use crate::utils::get_screen_bounds;

mod platform;
mod renderer;
mod utils;

pub struct System {
    window: Ref,
}

impl System {
    #[must_use]
    pub fn window(&self) -> &Ref {
        &self.window
    }

    #[must_use]
    pub fn window_mut(&mut self) -> &mut Ref {
        &mut self.window
    }
}

#[must_use]
pub fn init<A: App + 'static>(
    title: &'static str,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    app: Rc<RefCell<A>>,
) -> System {
    let mut imgui = Context::create();
    let platform = Platform::init(&mut imgui).expect("Unable to create platform");
    let renderer = Renderer::new(&mut imgui).expect("Unable to create renderer");
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    let bounds = get_screen_bounds();
    #[allow(clippy::cast_possible_wrap)]
    let rect = {
        let left = bounds.left + x as i32;
        let top = bounds.top - y as i32;
        let right = left + width as i32;
        let bottom = top - height as i32;
        Rect::new(left, top, right, bottom)
    };

    let mut window = Window::create(
        title,
        rect,
        Decoration::RoundRectangle,
        Layer::FloatingWindows,
        PositioningMode::Free,
        WindowDelegate::new(imgui, platform, renderer, app),
    );

    window.set_visible(false);

    window.set_gravity(Gravity {
        left: 0.0,
        top: 1.0,
        right: 1.0,
        bottom: 0.0,
    });

    System { window }
}

/// # Errors
///
/// Returns `ImageError` if the image could not be loaded.
pub fn create_texture(image: &RgbaImage) -> Result<TextureId, ImageError> {
    let texture_id = bind_texture();
    imgui_support::create_texture(texture_id, image)
}

struct WindowDelegate<A: App> {
    imgui: Context,
    platform: Platform,
    renderer: Renderer,
    app: Rc<RefCell<A>>,
}

impl<A: App> WindowDelegate<A> {
    fn new(
        imgui: Context,
        platform: Platform,
        renderer: Renderer,
        app: Rc<RefCell<A>>,
    ) -> WindowDelegate<A> {
        WindowDelegate {
            imgui,
            platform,
            renderer,
            app,
        }
    }
}

impl<A: App + 'static> Delegate for WindowDelegate<A> {
    fn draw(&mut self, window: &mut Window) {
        let geometry = window.geometry();

        self.platform.prepare_frame(self.imgui.io_mut(), window);

        self.imgui.style_mut().window_padding = [0.0, 0.0];
        let display_size = self.imgui.io().display_size;

        let ui = self.imgui.new_frame();
        #[allow(clippy::cast_precision_loss)]
        ui.window(window.title())
            .position([0.0, 0.0], Condition::Always)
            .size(display_size, Condition::Always)
            .flags(WindowFlags::NO_BACKGROUND | WindowFlags::NO_DECORATION | WindowFlags::NO_INPUTS)
            .build(|| self.app.borrow().draw_ui(ui));
        self.renderer.render(&mut self.imgui, geometry);
    }

    fn handle_event(&mut self, window: &Window, event: Event) {
        let consumed = self.app.borrow_mut().handle_event(event.clone());
        if !consumed {
            platform::handle_event(self.imgui.io_mut(), window, event);
        }
    }
}
