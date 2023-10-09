/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]

use std::ffi::c_void;

use gl21 as gl;
use image::{EncodableLayout, ImageError, RgbaImage};
use imgui::{TextureId, Ui};
use tracing::debug;

use dcommon::ui::events::Event;

pub mod renderer_common;

pub trait App {
    fn draw_ui(&self, _ui: &Ui) {}
    /// return true to consume the event
    fn handle_event(&mut self, event: Event) -> bool;
}

/// Use `imgui_support_(standalone|xplane)::create_texture` in preference to this.
///
/// # Errors
///
/// Returns `ImageError` if the image could not be loaded.
pub fn create_texture(texture_id: u32, image: &RgbaImage) -> Result<TextureId, ImageError> {
    let (width, height) = image.dimensions();
    #[allow(clippy::cast_possible_wrap)]
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as _,
            width as _,
            height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            image.as_bytes().as_ptr().cast::<c_void>(),
        );
    }
    Ok(TextureId::new(texture_id as _))
}

pub fn deallocate_texture(texture_id: TextureId) {
    debug!(id = texture_id.id(), "Deallocating texture");
    unsafe {
        gl::DeleteTextures(1, [texture_id.id()].as_ptr().cast());
    }
}
