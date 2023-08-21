/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use std::mem;

use gl::types::GLuint;
use gl21 as gl;
use imgui::{Context, DrawIdx};

use crate::renderer_common::{
    add_fonts, configure_imgui, render as common_render, return_param, FontStyles,
};

pub struct Renderer {
    font_texture: GLuint,
}

impl Renderer {
    pub fn new(imgui: &mut Context) -> Self {
        configure_imgui(imgui, "standalone");
        let font_texture = bind_texture();
        add_fonts(font_texture, imgui.fonts(), 14.0, &FontStyles::default());
        Self { font_texture }
    }
}

pub fn render(ctx: &mut Context) {
    let [width, height] = ctx.io().display_size;
    let [scale_w, scale_h] = ctx.io().display_framebuffer_scale;

    let fb_width = width * scale_w;
    let fb_height = height * scale_h;

    let draw_data = ctx.render();

    setup_render_state(
        fb_width,
        fb_height,
        draw_data.display_size,
        draw_data.display_pos,
    );

    common_render(
        draw_data,
        |count, clip_rect, texture_id, idx_buffer, idx_offset| {
            let [x, y, z, w] = clip_rect;
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            unsafe {
                gl::BindTexture(gl::TEXTURE_2D, texture_id.id() as _);
                gl::Scissor(
                    (x * scale_w) as _,
                    (fb_height - w * scale_h) as _,
                    ((z - x) * scale_w) as _,
                    ((w - y) * scale_h) as _,
                );
                let idx_size = if mem::size_of::<DrawIdx>() == 2 {
                    gl::UNSIGNED_SHORT
                } else {
                    gl::UNSIGNED_INT
                };
                gl::DrawElements(
                    gl::TRIANGLES,
                    count as _,
                    idx_size,
                    (idx_buffer.as_ptr() as usize + idx_offset * mem::size_of::<DrawIdx>()) as _,
                );
            }
        },
    );

    restore_render_state();
}

fn setup_render_state(
    fb_width: f32,
    fb_height: f32,
    display_size: [f32; 2],
    display_pos: [f32; 2],
) {
    unsafe {
        gl::PushAttrib(gl::ENABLE_BIT | gl::COLOR_BUFFER_BIT | gl::TRANSFORM_BIT);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Disable(gl::CULL_FACE);
        gl::Disable(gl::DEPTH_TEST);
        gl::Disable(gl::STENCIL_TEST);
        gl::Disable(gl::LIGHTING);
        gl::Disable(gl::COLOR_MATERIAL);
        gl::Enable(gl::SCISSOR_TEST);
        gl::EnableClientState(gl::VERTEX_ARRAY);
        gl::EnableClientState(gl::TEXTURE_COORD_ARRAY);
        gl::EnableClientState(gl::COLOR_ARRAY);
        gl::DisableClientState(gl::NORMAL_ARRAY);
        gl::Enable(gl::TEXTURE_2D);
        gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
        gl::ShadeModel(gl::SMOOTH);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        {
            gl::TexEnvi(gl::TEXTURE_ENV, gl::TEXTURE_ENV_MODE, gl::MODULATE as _);
            gl::Viewport(0, 0, fb_width as _, fb_height as _);
        }
        gl::MatrixMode(gl::PROJECTION);
        gl::PushMatrix();
        gl::LoadIdentity();
        gl::Ortho(
            f64::from(display_pos[0]),
            f64::from(display_pos[0] + display_size[0]),
            f64::from(display_pos[1] + display_size[1]),
            f64::from(display_pos[1]),
            -1.0,
            1.0,
        );
        gl::MatrixMode(gl::MODELVIEW);
        gl::PushMatrix();
        gl::LoadIdentity();
    }
}

fn restore_render_state() {
    unsafe {
        gl::DisableClientState(gl::COLOR_ARRAY);
        gl::DisableClientState(gl::TEXTURE_COORD_ARRAY);
        gl::DisableClientState(gl::VERTEX_ARRAY);
        gl::MatrixMode(gl::MODELVIEW);
        gl::PopMatrix();
        gl::MatrixMode(gl::PROJECTION);
        gl::PopMatrix();
        gl::PopAttrib();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.font_texture);
        }
    }
}

fn bind_texture() -> GLuint {
    unsafe {
        let texture = return_param(|x| gl::GenTextures(1, x));
        gl::BindTexture(gl::TEXTURE_2D, texture);
        texture
    }
}
