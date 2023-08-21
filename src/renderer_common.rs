/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use std::ffi::c_void;
use std::mem;

use gl21 as gl;
use imgui::{
    Context, DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, FontAtlas, FontConfig,
    FontGlyphRanges, FontSource, TextureId,
};

use crate::renderer_common::berkeley_mono::RANGES;

mod berkeley_mono {
    pub const REGULAR: &[u8] = include_bytes!("../resources/BerkeleyMono-Regular.ttf");
    pub const BOLD: &[u8] = include_bytes!("../resources/BerkeleyMono-Bold.ttf");
    pub const ITALIC: &[u8] = include_bytes!("../resources/BerkeleyMono-Italic.ttf");
    pub const BOLD_ITALIC: &[u8] = include_bytes!("../resources/BerkeleyMono-BoldItalic.ttf");
    pub const RANGES: &[u32] = &[
        0x1, 0xFF, 0x2000, 0x22FF, 0x2500, 0x25FF, 0xE000, 0xE0FF, 0xFF00, 0xFFFF, 0,
    ];
}

#[allow(clippy::struct_excessive_bools)]
pub struct FontStyles {
    pub regular: bool,
    pub bold: bool,
    pub italic: bool,
    pub bold_italic: bool,
}

impl Default for FontStyles {
    fn default() -> Self {
        FontStyles {
            regular: true,
            bold: false,
            italic: false,
            bold_italic: false,
        }
    }
}

pub fn add_fonts(font_texture: u32, atlas: &mut FontAtlas, size_pixels: f32, styles: &FontStyles) {
    unsafe {
        #[allow(clippy::cast_possible_wrap)]
        {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        }
        gl::PixelStorei(gl::UNPACK_ROW_LENGTH, 0);
    }

    if styles.regular {
        add_font(atlas, "Regular", size_pixels, berkeley_mono::REGULAR);
    }
    if styles.bold {
        add_font(atlas, "Bold", size_pixels, berkeley_mono::BOLD);
    }
    if styles.italic {
        add_font(atlas, "Italic", size_pixels, berkeley_mono::ITALIC);
    }
    if styles.bold_italic {
        add_font(
            atlas,
            "Bold Italic",
            size_pixels,
            berkeley_mono::BOLD_ITALIC,
        );
    }
    let texture = atlas.build_rgba32_texture();

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    unsafe {
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as _,
            texture.width as _,
            texture.height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            texture.data.as_ptr().cast::<c_void>(),
        );
    }
    atlas.tex_id = TextureId::new(font_texture as usize);
}

fn add_font(atlas: &mut FontAtlas, name: &str, size_pixels: f32, data: &[u8]) {
    let size_str = size_pixels.to_string();

    atlas.add_font(&[FontSource::TtfData {
        data,
        size_pixels,
        config: Some(FontConfig {
            name: Some(format!("Berkeley Mono {name} {size_str}")),
            oversample_v: 4,
            oversample_h: 4,
            ellipsis_char: Some('\u{2026}'),
            glyph_ranges: FontGlyphRanges::from_slice(RANGES),
            ..FontConfig::default()
        }),
    }]);
}

pub fn configure_imgui(imgui: &mut Context, name: &str) {
    imgui.set_renderer_name(Some(format!(
        "imgui-{name}-renderer {}",
        env!("CARGO_PKG_VERSION")
    )));

    {
        let style = imgui.style_mut();
        style.window_rounding = 3.0;
        style.frame_rounding = 2.0;
    }
}

pub fn render<F: Fn(usize, [f32; 4], TextureId, &[DrawIdx], usize)>(
    draw_data: &DrawData,
    draw_element_fn: F,
) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    unsafe {
        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            gl::VertexPointer(
                2,
                gl::FLOAT,
                mem::size_of::<DrawVert>() as _,
                vtx_buffer.as_ptr().cast(),
            );

            gl::TexCoordPointer(
                2,
                gl::FLOAT,
                mem::size_of::<DrawVert>() as _,
                (vtx_buffer.as_ptr() as usize + mem::size_of::<[f32; 2]>()) as _,
            );

            gl::ColorPointer(
                4,
                gl::UNSIGNED_BYTE,
                mem::size_of::<DrawVert>() as _,
                (vtx_buffer.as_ptr() as usize + mem::size_of::<[f32; 4]>()) as _,
            );

            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                idx_offset,
                                ..
                            },
                    } => {
                        draw_element_fn(count, clip_rect, texture_id, idx_buffer, idx_offset);
                    }
                    DrawCmd::ResetRenderState => {
                        unimplemented!("Haven't implemented DrawCmd::ResetRenderState yet");
                    }
                    DrawCmd::RawCallback { .. } => {
                        unimplemented!("Haven't implemented user callbacks yet");
                    }
                }
            }
        }
    }
}

pub fn return_param<T, F>(f: F) -> T
where
    F: FnOnce(&mut T),
{
    let mut val = unsafe { mem::zeroed() };
    f(&mut val);
    val
}
