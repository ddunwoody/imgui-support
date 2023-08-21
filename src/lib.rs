/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use std::ffi::c_void;

use cfg_if::cfg_if;
use gl21 as gl;
use image::{EncodableLayout, ImageError, RgbaImage};
use imgui::{TextureId, Ui};
use tracing::debug;

use dcommon::ui::events::Event;

use crate::renderer_common::return_param;

#[cfg(not(any(feature = "standalone", feature = "xplane")))]
compile_error!("At least one of the features ['standalone', 'xplane'] must be enabled");

#[cfg(feature = "standalone")]
pub mod standalone;

mod renderer_common;
#[cfg(feature = "xplane")]
pub mod xplane;

pub trait App {
    fn draw_ui(&self, _ui: &Ui) {}
    /// return true to consume the event
    fn handle_event(&mut self, event: Event) -> bool;
}

/// # Errors
///
/// Returns `ImageError` if the image could not be loaded.
pub fn create_texture(image: &RgbaImage) -> Result<TextureId, ImageError> {
    let (width, height) = image.dimensions();
    let texture_id = gen_texture();
    debug!(texture_id, "Creating texture");
    bind_texture(texture_id);
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

fn gen_texture() -> u32 {
    cfg_if! {
        if #[cfg(feature = "standalone")] {
            #[allow(clippy::cast_sign_loss)]
            unsafe {
                return_param(|x| gl21::GenTextures(1, x)) as _
            }
        } else if #[cfg(feature = "xplane")] {
            #[allow(clippy::cast_sign_loss)]
            unsafe {
                return_param(|x| xplm_sys::XPLMGenerateTextureNumbers(x, 1)) as _
            }
        }
    }
}

fn bind_texture(texture_id: u32) {
    cfg_if! {
        if #[cfg(feature = "standalone")] {
            unsafe {
                gl21::BindTexture(gl21::TEXTURE_2D, texture_id as _);
            }
        } else if #[cfg(feature = "xplane")] {
            #[allow(clippy::cast_possible_wrap)]
            unsafe {
                xplm_sys::XPLMBindTexture2d(texture_id as _, 0);
            }
        }
    }
}
