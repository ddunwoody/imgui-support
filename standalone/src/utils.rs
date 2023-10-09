/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use glfw::Glfw;

use dcommon::ui::geometry::Rect;

#[must_use]
pub fn get_screen_bounds(glfw: &mut Glfw) -> Rect {
    #[allow(clippy::cast_possible_wrap)]
    glfw.with_primary_monitor(|_, m| {
        let mode = m
            .expect("Failed to get primary monitor")
            .get_video_mode()
            .expect("Failed to get video mode");
        Rect::new(0, 0, mode.width as _, mode.height as _)
    })
}
