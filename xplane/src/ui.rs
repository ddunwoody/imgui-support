/*
 * Copyright (c) 2023 David Dunwoody.
 *
 * All rights reserved.
 */

#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void, CString};
use std::mem::size_of;
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;

use xplm_sys::{
    xplm_ControlFlag, xplm_CursorDefault, xplm_MouseUp, xplm_OptionAltFlag, xplm_ShiftFlag,
    xplm_UpFlag, xplm_WindowCenterOnMonitor, xplm_WindowDecorationNone,
    xplm_WindowDecorationRoundRectangle, xplm_WindowDecorationSelfDecorated,
    xplm_WindowDecorationSelfDecoratedResizable, xplm_WindowFullScreenOnAllMonitors,
    xplm_WindowFullScreenOnMonitor, xplm_WindowLayerFlightOverlay, xplm_WindowLayerFloatingWindows,
    xplm_WindowLayerGrowlNotifications, xplm_WindowLayerModal, xplm_WindowPopOut,
    xplm_WindowPositionFree, xplm_WindowVR, XPLMGetWindowGeometryOS, XPLMGetWindowGeometryVR,
    XPLMSetWindowGeometry, XPLMSetWindowGeometryOS, XPLMSetWindowGeometryVR, XPLMSetWindowGravity,
    XPLMSetWindowPositioningMode, XPLMSetWindowResizingLimits, XPLMWindowIsInVR,
    XPLMWindowIsPoppedOut, XPLMWindowPositioningMode,
};
use xplm_sys::{
    XPLMBringWindowToFront, XPLMCreateWindow_t, XPLMCreateWindowEx, XPLMCursorStatus,
    XPLMDestroyWindow, XPLMGetWindowGeometry, XPLMGetWindowIsVisible, XPLMHasKeyboardFocus,
    XPLMIsWindowInFront, XPLMKeyFlags, XPLMMouseStatus, XPLMSetWindowIsVisible, XPLMSetWindowTitle,
    XPLMTakeKeyboardFocus, XPLMWindowDecoration, XPLMWindowID, XPLMWindowLayer,
};

use imgui_support::events::{Action, Event, Modifiers, MouseButton};
use imgui_support::geometry::Rect;

use crate::ui::keymap::to_imgui_key;

mod keymap;

pub trait Delegate: 'static {
    /// Draws the window contents
    fn draw(&mut self, window: &mut Window);

    fn handle_event(&mut self, window: &Window, event: Event);
}

pub struct Ref {
    window: Box<Window>,
}

impl Deref for Ref {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

impl DerefMut for Ref {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window
    }
}

pub struct Window {
    id: XPLMWindowID,
    delegate: Box<dyn Delegate>,
    title: String,
    gravity: Gravity,
    resizing_limits: Option<ResizingLimits>,
}

impl Window {
    pub fn create<D: Delegate>(
        title: &str,
        rect: Rect,
        decoration: Decoration,
        layer: Layer,
        positioning_mode: PositioningMode,
        delegate: D,
    ) -> Ref {
        let mut window_box = Box::new(Window {
            id: null_mut(),
            delegate: Box::new(delegate),
            title: String::from(title),
            gravity: Gravity::default(),
            resizing_limits: None,
        });
        let window_ptr: *mut Window = &mut *window_box;

        let Rect {
            left,
            top,
            right,
            bottom,
        } = rect;
        let mut params = XPLMCreateWindow_t {
            structSize: size_of::<XPLMCreateWindow_t>() as _,
            left,
            top,
            right,
            bottom,
            visible: 1,
            drawWindowFunc: Some(draw_window),
            handleMouseClickFunc: Some(handle_mouse_click),
            handleKeyFunc: Some(handle_key),
            handleCursorFunc: Some(handle_cursor),
            handleMouseWheelFunc: Some(handle_mouse_wheel),
            refcon: window_ptr.cast(),
            decorateAsFloatingWindow: decoration.into(),
            layer: layer.into(),
            handleRightClickFunc: Some(handle_right_click),
        };

        window_box.id = unsafe {
            let id = XPLMCreateWindowEx(&mut params);
            XPLMSetWindowPositioningMode(id, positioning_mode.into(), -1);
            id
        };
        set_title(window_box.id, title);
        Ref { window: window_box }
    }

    pub fn set_title(&mut self, title: &str) {
        set_title(self.id, title);
        self.title = String::from(title);
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn geometry(&self) -> Rect {
        get_geometry(self, XPLMGetWindowGeometry)
    }

    pub fn set_geometry(&mut self, rect: &Rect) {
        set_geometry(self, XPLMSetWindowGeometry, rect);
    }

    #[must_use]
    pub fn geometry_os(&self) -> Rect {
        get_geometry(self, XPLMGetWindowGeometryOS)
    }

    pub fn set_geometry_os(&mut self, rect: &Rect) {
        set_geometry(self, XPLMSetWindowGeometryOS, rect);
    }

    #[must_use]
    pub fn geometry_vr(&self) -> (i32, i32) {
        let mut width = 0;
        let mut height = 0;
        unsafe {
            XPLMGetWindowGeometryVR(self.id, &mut width, &mut height);
        }
        (width, height)
    }

    pub fn set_geometry_vr(&self, width: i32, height: i32) {
        unsafe {
            XPLMSetWindowGeometryVR(self.id, width, height);
        }
    }

    #[must_use]
    pub fn current_geometry(&self) -> (&PositioningMode, Rect) {
        let positioning_mode = self.positioning_mode();
        let geometry = match positioning_mode {
            PositioningMode::VR => {
                let (width, height) = self.geometry_vr();
                Rect::new(0, 0, width, height)
            }
            PositioningMode::PopOut => self.geometry_os(),
            _ => self.geometry(),
        };
        (positioning_mode, geometry)
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        unsafe { XPLMGetWindowIsVisible(self.id) != 0 }
    }

    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            XPLMSetWindowIsVisible(self.id, i32::from(visible));
        }
    }

    pub fn toggle_visible(&mut self) -> bool {
        let new_visibility = !self.visible();
        self.set_visible(new_visibility);
        new_visibility
    }

    #[must_use]
    pub fn popped_out(&self) -> bool {
        unsafe { XPLMWindowIsPoppedOut(self.id) != 0 }
    }

    #[must_use]
    pub fn in_vr(&self) -> bool {
        unsafe { XPLMWindowIsInVR(self.id) != 0 }
    }

    #[must_use]
    pub fn gravity(&self) -> &Gravity {
        &self.gravity
    }

    pub fn set_gravity(&mut self, gravity: Gravity) {
        unsafe {
            XPLMSetWindowGravity(
                self.id,
                gravity.left,
                gravity.top,
                gravity.right,
                gravity.bottom,
            );
        }
    }

    pub fn set_resizing_limits(&mut self, resizing_limits: ResizingLimits) {
        unsafe {
            XPLMSetWindowResizingLimits(
                self.id,
                resizing_limits.min_width,
                resizing_limits.min_height,
                resizing_limits.max_width,
                resizing_limits.max_height,
            );
        }
        self.resizing_limits = Some(resizing_limits);
    }

    #[must_use]
    pub fn positioning_mode(&self) -> &PositioningMode {
        if self.in_vr() {
            &PositioningMode::VR
        } else if self.popped_out() {
            &PositioningMode::PopOut
        } else {
            &PositioningMode::Free
        }
    }

    pub fn set_positioning_mode(&mut self, positioning_mode: PositioningMode) {
        unsafe {
            XPLMSetWindowPositioningMode(self.id, positioning_mode.clone().into(), -1);
        }
    }

    #[must_use]
    pub fn has_keyboard_focus(&self) -> bool {
        unsafe { XPLMHasKeyboardFocus(self.id) == 1 }
    }

    pub fn take_keyboard_focus(&mut self) {
        unsafe {
            XPLMTakeKeyboardFocus(self.id);
        }
    }

    pub fn release_keyboard_focus(&mut self) {
        unsafe {
            if self.has_keyboard_focus() {
                XPLMTakeKeyboardFocus(null_mut());
            }
        }
    }

    #[must_use]
    pub fn is_in_front(&self) -> bool {
        unsafe { XPLMIsWindowInFront(self.id) == 1 }
    }

    pub fn bring_to_front(&mut self) {
        unsafe {
            XPLMBringWindowToFront(self.id);
        }
    }
}

fn set_title(id: XPLMWindowID, title: &str) {
    let title_c = CString::new(title).expect("Could not create string from {title}");
    unsafe {
        XPLMSetWindowTitle(id, title_c.as_ptr());
    }
}

fn get_geometry(
    window: &Window,
    f: unsafe extern "C" fn(XPLMWindowID, *mut c_int, *mut c_int, *mut c_int, *mut c_int),
) -> Rect {
    let mut left = 0;
    let mut top = 0;
    let mut right = 0;
    let mut bottom = 0;
    unsafe {
        f(window.id, &mut left, &mut top, &mut right, &mut bottom);
    }
    Rect::new(left, top, right, bottom)
}

fn set_geometry(
    window: &mut Window,
    f: unsafe extern "C" fn(XPLMWindowID, c_int, c_int, c_int, c_int),
    rect: &Rect,
) {
    unsafe {
        f(window.id, rect.left, rect.top, rect.right, rect.bottom);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            XPLMDestroyWindow(self.id);
        }
    }
}

#[derive(Debug)]
pub enum Decoration {
    None,
    RoundRectangle,
    SelfDecorated,
    SelfDecoratedResizable,
}

impl From<Decoration> for XPLMWindowDecoration {
    fn from(value: Decoration) -> Self {
        match value {
            Decoration::None => xplm_WindowDecorationNone as XPLMWindowDecoration,
            Decoration::RoundRectangle => {
                xplm_WindowDecorationRoundRectangle as XPLMWindowDecoration
            }
            Decoration::SelfDecorated => xplm_WindowDecorationSelfDecorated as XPLMWindowDecoration,
            Decoration::SelfDecoratedResizable => {
                xplm_WindowDecorationSelfDecoratedResizable as XPLMWindowDecoration
            }
        }
    }
}

#[derive(Debug)]
pub enum Layer {
    FlightOverlay,
    FloatingWindows,
    Modal,
    GrowlNotifications,
}

impl From<Layer> for XPLMWindowLayer {
    fn from(value: Layer) -> Self {
        match value {
            Layer::FlightOverlay => xplm_WindowLayerFlightOverlay as XPLMWindowLayer,
            Layer::FloatingWindows => xplm_WindowLayerFloatingWindows as XPLMWindowLayer,
            Layer::Modal => xplm_WindowLayerModal as XPLMWindowLayer,
            Layer::GrowlNotifications => xplm_WindowLayerGrowlNotifications as XPLMWindowLayer,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PositioningMode {
    Free,
    CenterOnMonitor,
    FullScreenOnMonitor,
    FullScreenOnAllMonitors,
    PopOut,
    VR,
}

impl From<PositioningMode> for XPLMWindowPositioningMode {
    fn from(value: PositioningMode) -> Self {
        match value {
            PositioningMode::Free => xplm_WindowPositionFree as XPLMWindowPositioningMode,
            PositioningMode::CenterOnMonitor => {
                xplm_WindowCenterOnMonitor as XPLMWindowPositioningMode
            }
            PositioningMode::FullScreenOnMonitor => {
                xplm_WindowFullScreenOnMonitor as XPLMWindowPositioningMode
            }
            PositioningMode::FullScreenOnAllMonitors => {
                xplm_WindowFullScreenOnAllMonitors as XPLMWindowPositioningMode
            }
            PositioningMode::PopOut => xplm_WindowPopOut as XPLMWindowPositioningMode,
            PositioningMode::VR => xplm_WindowVR as XPLMWindowPositioningMode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Gravity {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Gravity {
    #[must_use]
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl Default for Gravity {
    fn default() -> Self {
        Gravity::new(0.0, 1.0, 0.0, 1.0)
    }
}

pub struct ResizingLimits {
    pub min_width: i32,
    pub min_height: i32,
    pub max_width: i32,
    pub max_height: i32,
}

impl ResizingLimits {
    #[must_use]
    pub fn new(min_width: i32, min_height: i32, max_width: i32, max_height: i32) -> Self {
        Self {
            min_width,
            min_height,
            max_width,
            max_height,
        }
    }
}

unsafe extern "C" fn draw_window(_window: XPLMWindowID, refcon: *mut c_void) {
    let window: *mut Window = refcon.cast();
    (*window).delegate.draw(&mut *window);
}

unsafe extern "C" fn handle_mouse_click(
    _window: XPLMWindowID,
    _x: c_int,
    _y: c_int,
    status: XPLMMouseStatus,
    refcon: *mut c_void,
) -> c_int {
    let action = if status == xplm_MouseUp as _ {
        Action::Release
    } else {
        Action::Press
    };

    let event = Event::MouseButton(MouseButton::Left, action);
    let window: *mut Window = refcon.cast();
    (*window).delegate.handle_event(&*window, event);
    1
}

#[allow(clippy::cast_sign_loss)]
unsafe extern "C" fn handle_key(
    _window: XPLMWindowID,
    key: c_char,
    flags: XPLMKeyFlags,
    virtual_key: c_char,
    refcon: *mut c_void,
    losing_focus: c_int,
) {
    if losing_focus == 0 {
        let ch = key as u8 as char;

        let action = if flag_set(flags, xplm_UpFlag as XPLMKeyFlags) {
            Action::Release
        } else {
            Action::Press
        };

        let modifiers = Modifiers {
            control: flag_set(flags, xplm_ControlFlag as XPLMKeyFlags),
            option: flag_set(flags, xplm_OptionAltFlag as XPLMKeyFlags),
            shift: flag_set(flags, xplm_ShiftFlag as XPLMKeyFlags),
        };

        let event = Event::Key(to_imgui_key(virtual_key), ch, action, modifiers);
        let window: *mut Window = refcon.cast();
        (*window).delegate.handle_event(&*window, event);
    }
}

fn flag_set(flags: XPLMKeyFlags, flag: XPLMKeyFlags) -> bool {
    flags & flag as XPLMKeyFlags != 0
}

unsafe extern "C" fn handle_cursor(
    _window: XPLMWindowID,
    x: c_int,
    y: c_int,
    refcon: *mut c_void,
) -> XPLMCursorStatus {
    let event = Event::CursorPos(x, y);
    let window: *mut Window = refcon.cast();
    (*window).delegate.handle_event(&*window, event);
    xplm_CursorDefault as _
}

unsafe extern "C" fn handle_mouse_wheel(
    _window: XPLMWindowID,
    _x: c_int,
    _y: c_int,
    wheel: c_int,
    clicks: c_int,
    refcon: *mut c_void,
) -> c_int {
    let (x, y) = if wheel == 0 { (0, clicks) } else { (clicks, 0) };
    let event = Event::Scroll(x, y);
    let window: *mut Window = refcon.cast();
    (*window).delegate.handle_event(&*window, event);
    1
}

unsafe extern "C" fn handle_right_click(
    _window: XPLMWindowID,
    _x: c_int,
    _y: c_int,
    status: XPLMMouseStatus,
    refcon: *mut c_void,
) -> c_int {
    let action = if status == xplm_MouseUp as _ {
        Action::Release
    } else {
        Action::Press
    };
    let event = Event::MouseButton(MouseButton::Right, action);
    let window: *mut Window = refcon.cast();
    (*window).delegate.handle_event(&*window, event);
    1
}
