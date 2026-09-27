//! Headless screenshot capture using the real platform renderer when available.

use gpui_pre::{
    AnyWindowHandle, App, AppContext, HeadlessAppContext, Keystroke, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, PlatformInput, Point, Render, Size, WindowHandle,
};
use image::RgbaImage;
use std::{fmt, sync::Arc};

/// Errors returned when a headless screenshot cannot be rendered.
#[derive(Debug)]
pub enum ScreenshotError {
    /// The current GPUI platform does not provide an offscreen renderer.
    UnsupportedPlatform,
    /// The requested scale is zero, negative, or not finite.
    InvalidScale,
    /// GPUI failed while creating or rendering the window.
    Gpui(String),
}

/// Perceptual screenshot comparison tolerance. A pixel differs when any RGB
/// channel differs by more than `channel_delta`; at most `max_different_pixels`
/// such pixels may differ.
#[derive(Clone, Copy, Debug, Default)]
pub struct PixelTolerance {
    pub channel_delta: u8,
    pub max_different_pixels: u64,
}

impl PixelTolerance {
    /// Compare same-sized images using this tolerance.
    pub fn matches(self, expected: &RgbaImage, actual: &RgbaImage) -> bool {
        if expected.dimensions() != actual.dimensions() {
            return false;
        }
        let differing = expected
            .pixels()
            .zip(actual.pixels())
            .filter(|(left, right)| {
                (0..3).any(|channel| left[channel].abs_diff(right[channel]) > self.channel_delta)
            })
            .count() as u64;
        differing <= self.max_different_pixels
    }
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(
                f,
                "gpui-pre 0.3.5 does not provide a headless screenshot renderer on this platform"
            ),
            Self::InvalidScale => write!(f, "screenshot scale must be a positive finite number"),
            Self::Gpui(error) => write!(f, "headless screenshot failed: {error}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}

/// Render a GPUI view into an RGBA image.
///
/// `scale` is the window's device scale factor. `theme` runs before the window is
/// created so callers can initialize GPUI globals and install or configure a
/// theme. Applications using GPUI Base or Components should initialize that
/// layer in this callback (for example, `gpui_base::init` or `gpui_component::init`)
/// so text and theme-backed content render as they do in the application. GPUI
/// 0.3.5 exposes a real headless renderer only on macOS; other platforms return
/// [`ScreenshotError::UnsupportedPlatform`].
pub fn screenshot<V, T>(
    view: V,
    size: Size<Pixels>,
    scale: f32,
    theme: T,
) -> Result<RgbaImage, ScreenshotError>
where
    V: Render + 'static,
    T: FnOnce(&mut App),
{
    let mut session = HeadlessSession::new(view, size, scale, theme)?;
    session.capture()
}

/// A persistent headless window that can be updated and captured repeatedly.
pub struct HeadlessSession<V: Render + 'static> {
    context: HeadlessAppContext,
    window: WindowHandle<V>,
    logical_size: Size<Pixels>,
    scale: f32,
}

impl<V: Render + 'static> HeadlessSession<V> {
    /// Create a headless window containing `view`.
    pub fn new<T>(
        view: V,
        size: Size<Pixels>,
        scale: f32,
        theme: T,
    ) -> Result<Self, ScreenshotError>
    where
        T: FnOnce(&mut App),
    {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(ScreenshotError::InvalidScale);
        }
        if !cfg!(target_os = "macos") {
            return Err(ScreenshotError::UnsupportedPlatform);
        }
        let platform = gpui_platform::current_platform(true);
        let mut context = HeadlessAppContext::with_platform(
            platform.text_system(),
            Arc::new(()),
            gpui_platform::current_headless_renderer,
        );
        context.update(theme);
        let window = context
            .open_window(size, |_, cx| cx.new(|_| view))
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
        context
            .update_window(window.into(), |_, window, cx| {
                let native_scale = window.scale_factor();
                window.set_scale_factor(native_scale);
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
        context.run_until_parked();
        context
            .update_window(window.into(), |_, window, cx| {
                // The backing layer determines the actual Metal target size. Keep
                // GPUI at that native scale while drawing, then resize the image
                // to the requested logical scale in `capture`.
                let native_scale = window.scale_factor();
                window.set_scale_factor(native_scale);
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
        Ok(Self { context, window, logical_size: size, scale })
    }

    /// Capture the currently rendered window.
    pub fn capture(&mut self) -> Result<RgbaImage, ScreenshotError> {
        let image = self
            .context
            .capture_screenshot(self.window.into())
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
        let width = (self.logical_size.width.as_f32() * self.scale).ceil().max(1.0) as u32;
        let height = (self.logical_size.height.as_f32() * self.scale).ceil().max(1.0) as u32;
        if image.dimensions() == (width, height) {
            Ok(image)
        } else {
            Ok(image::imageops::resize(
                &image,
                width,
                height,
                image::imageops::FilterType::Lanczos3,
            ))
        }
    }

    /// The GPUI window handle for input dispatch and accessibility inspection.
    pub fn window_handle(&self) -> AnyWindowHandle {
        self.window.into()
    }

    /// Access and update the root view, window, and app, then redraw.
    pub fn update<R>(
        &mut self,
        update: impl FnOnce(gpui_pre::Entity<V>, &mut gpui_pre::Window, &mut App) -> R,
    ) -> Result<R, ScreenshotError> {
        self.context
            .update_window(self.window.into(), |root, window, cx| {
                let root = root.downcast::<V>().expect("headless root type changed");
                let result = update(root, window, cx);
                window.refresh();
                window.draw(cx).clear(cx);
                result
            })
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))
    }

    /// Type text through GPUI's keyboard dispatch into the focused input handler.
    pub fn simulate_input(&mut self, input: &str) -> Result<(), ScreenshotError> {
        for character in input.chars() {
            let key = character.to_string();
            let keystroke =
                Keystroke::parse(&key).map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
            self.context
                .update_window(self.window.into(), |_, window, cx| {
                    window.dispatch_keystroke(keystroke, cx);
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
            self.context.run_until_parked();
        }
        Ok(())
    }

    /// Dispatch GPUI keystrokes such as `cmd-k`, `down`, or `escape` to the focused view.
    pub fn simulate_keystrokes(&mut self, keys: &str) -> Result<(), ScreenshotError> {
        for key in keys.split_whitespace() {
            let keystroke =
                Keystroke::parse(key).map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
            self.context
                .update_window(self.window.into(), |_, window, cx| {
                    window.dispatch_keystroke(keystroke, cx);
                    window.refresh();
                    window.draw(cx).clear(cx);
                })
                .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
            self.context.run_until_parked();
        }
        Ok(())
    }

    /// Press the primary pointer button at a window-local point.
    pub fn simulate_pointer_down(
        &mut self,
        position: Point<Pixels>,
    ) -> Result<(), ScreenshotError> {
        self.dispatch_pointer(PlatformInput::MouseDown(MouseDownEvent {
            button: MouseButton::Left,
            position,
            ..Default::default()
        }))
    }

    /// Move the primary pointer while the primary button remains down.
    pub fn simulate_pointer_drag_to(
        &mut self,
        position: Point<Pixels>,
    ) -> Result<(), ScreenshotError> {
        self.dispatch_pointer(PlatformInput::MouseMove(MouseMoveEvent {
            position,
            pressed_button: Some(MouseButton::Left),
            ..Default::default()
        }))
    }

    /// Release the primary pointer button at a window-local point.
    pub fn simulate_pointer_up(&mut self, position: Point<Pixels>) -> Result<(), ScreenshotError> {
        self.dispatch_pointer(PlatformInput::MouseUp(MouseUpEvent {
            button: MouseButton::Left,
            position,
            ..Default::default()
        }))
    }

    fn dispatch_pointer(&mut self, input: PlatformInput) -> Result<(), ScreenshotError> {
        self.context
            .update_window(self.window.into(), |_, window, cx| {
                window.dispatch_event(input, cx);
                window.refresh();
                window.draw(cx).clear(cx);
            })
            .map_err(|error| ScreenshotError::Gpui(error.to_string()))?;
        self.context.run_until_parked();
        Ok(())
    }
}
