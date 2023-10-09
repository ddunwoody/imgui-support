/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */
use mint::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    #[must_use]
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        (self.right - self.left).unsigned_abs()
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        (self.top - self.bottom).unsigned_abs()
    }
}

impl From<Rect> for Vector2<f32> {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: Rect) -> Self {
        let v: [f32; 2] = value.into();
        Vector2::from(v)
    }
}

impl From<Rect> for [f32; 2] {
    #[allow(clippy::cast_precision_loss)]
    fn from(value: Rect) -> Self {
        [value.width() as f32, value.height() as f32]
    }
}
