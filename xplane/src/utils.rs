/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

use xplm_sys::XPLMGetScreenBoundsGlobal;

use dcommon::ui::geometry::Rect;

#[must_use]
pub fn get_screen_bounds() -> Rect {
    let mut bounds = [0; 4];
    unsafe {
        XPLMGetScreenBoundsGlobal(
            &mut bounds[0],
            &mut bounds[1],
            &mut bounds[2],
            &mut bounds[3],
        );
    }
    Rect::new(bounds[0], bounds[1], bounds[2], bounds[3])
}
