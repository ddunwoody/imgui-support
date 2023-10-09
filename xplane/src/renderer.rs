/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use std::mem;

use gl21 as gl;
use gl::types::GLuint;
use imgui::{Context, DrawIdx};
use xplm::data::ArrayRead;
use xplm::data::borrowed::{DataRef, FindError};
use xplm_sys::{XPLMBindTexture2d, XPLMGenerateTextureNumbers, XPLMSetGraphicsState};

use imgui_support::geometry::Rect;
use imgui_support::renderer_common::{
    add_fonts, configure_imgui, FontStyles, render, return_param,
};

pub struct Renderer {
    font_texture: GLuint,
    modelview_matrix: DataRef<[f32]>,
    viewport: DataRef<[i32]>,
    projection_matrix: DataRef<[f32]>,
}

impl Renderer {
    pub fn new(imgui: &mut Context) -> Result<Renderer, FindError> {
        configure_imgui(imgui, "xplane");
        let font_texture = bind_texture();
        add_fonts(font_texture, imgui.fonts(), 14.0, &FontStyles::default());

        Ok(Renderer {
            font_texture,
            modelview_matrix: DataRef::find("sim/graphics/view/modelview_matrix")?,
            viewport: DataRef::find("sim/graphics/view/viewport")?,
            projection_matrix: DataRef::find("sim/graphics/view/projection_matrix")?,
        })
    }

    pub fn render(&self, imgui: &mut Context, rect: Rect) {
        let Rect { left, top, .. } = rect;
        setup_render_state(left, top);
        let mut modelview = [0.0; 16];
        let mut projection = [0.0; 16];
        let mut viewport = [0; 4];

        self.modelview_matrix.get(&mut modelview);
        self.projection_matrix.get(&mut projection);
        self.viewport.get(&mut viewport);

        let draw_data = imgui.render();
        render(
            draw_data,
            |count, clip_rect, texture_id, idx_buffer, idx_offset| {
                let [x, y, z, w] = clip_rect;
                unsafe {
                    XPLMBindTexture2d(
                        texture_id
                            .id()
                            .try_into()
                            .unwrap_or_else(|e| panic!("Unable to convert texture ID: {e}")),
                        0,
                    );
                    let (b_left, b_top) = translate_imgui_to_boxel(left, top, x, y);
                    let (b_right, b_bottom) = translate_imgui_to_boxel(left, top, z, w);
                    let (n_left, n_top) =
                        boxels_to_native(b_left, b_top, modelview, projection, viewport);
                    let (n_right, n_bottom) =
                        boxels_to_native(b_right, b_bottom, modelview, projection, viewport);
                    gl::Scissor(n_left, n_bottom, n_right - n_left, n_top - n_bottom);
                    let idx_size = if mem::size_of::<DrawIdx>() == 2 {
                        gl::UNSIGNED_SHORT
                    } else {
                        gl::UNSIGNED_INT
                    };
                    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                    gl::DrawElements(
                        gl::TRIANGLES,
                        count as _,
                        idx_size,
                        (idx_buffer.as_ptr() as usize + idx_offset * mem::size_of::<DrawIdx>())
                            as _,
                    );
                }
            },
        );
        restore_render_state();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.font_texture);
        }
    }
}

fn setup_render_state(left: i32, top: i32) {
    unsafe {
        XPLMSetGraphicsState(0, 1, 0, 1, 1, 0, 0);
        gl::PushClientAttrib(gl::CLIENT_ALL_ATTRIB_BITS);
        gl::PushAttrib(gl::ENABLE_BIT | gl::COLOR_BUFFER_BIT | gl::TRANSFORM_BIT);
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::SCISSOR_TEST);
        gl::EnableClientState(gl::VERTEX_ARRAY);
        gl::EnableClientState(gl::TEXTURE_COORD_ARRAY);
        gl::EnableClientState(gl::COLOR_ARRAY);
        gl::Enable(gl::TEXTURE_2D);

        gl::MatrixMode(gl::PROJECTION);
        gl::PushMatrix();
        gl::Scalef(1.0, -1.0, 1.0);
        #[allow(clippy::cast_precision_loss)]
        gl::Translatef(left as _, -top as _, 0.0);
    }
}

fn restore_render_state() {
    unsafe {
        gl::MatrixMode(gl::PROJECTION);
        gl::PopMatrix();
        // Restore modified state
        gl::DisableClientState(gl::VERTEX_ARRAY);
        gl::DisableClientState(gl::COLOR_ARRAY);
        gl::DisableClientState(gl::TEXTURE_COORD_ARRAY);
        gl::PopAttrib();
        gl::PopClientAttrib();
    }
}

#[allow(clippy::cast_possible_truncation)]
fn translate_imgui_to_boxel(left: i32, top: i32, x: f32, y: f32) -> (i32, i32) {
    (left + x as i32, top - y as i32)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn boxels_to_native(
    x: i32,
    y: i32,
    modelview: [f32; 16],
    projection: [f32; 16],
    viewport: [i32; 4],
) -> (i32, i32) {
    let eye = mult_matrix_vec4f(modelview, [x as f32, y as f32, 0.0, 1.0]);
    let mut ndc = mult_matrix_vec4f(projection, eye);
    ndc[3] = 1.0 / ndc[3];
    ndc[0] *= ndc[3];
    ndc[1] *= ndc[3];

    let out_x = (ndc[0] * 0.5 + 0.5) * viewport[2] as f32 + viewport[0] as f32;
    let out_y = (ndc[1] * 0.5 + 0.5) * viewport[3] as f32 + viewport[1] as f32;
    (out_x as i32, out_y as i32)
}

fn mult_matrix_vec4f(m: [f32; 16], v: [f32; 4]) -> [f32; 4] {
    let mut out = [0.0f32; 4];
    out[0] = v[0] * m[0] + v[1] * m[4] + v[2] * m[8] + v[3] * m[12];
    out[1] = v[0] * m[1] + v[1] * m[5] + v[2] * m[9] + v[3] * m[13];
    out[2] = v[0] * m[2] + v[1] * m[6] + v[2] * m[10] + v[3] * m[14];
    out[3] = v[0] * m[3] + v[1] * m[7] + v[2] * m[11] + v[3] * m[15];
    out
}

pub(crate) fn bind_texture() -> GLuint {
    #[allow(clippy::cast_sign_loss)]
    unsafe {
        let texture = return_param(|x| XPLMGenerateTextureNumbers(x, 1));
        XPLMBindTexture2d(texture, 0);
        texture as _
    }
}
