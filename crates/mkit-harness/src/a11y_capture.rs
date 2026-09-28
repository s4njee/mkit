//! Headless AccessKit capture through a harness-owned GPUI platform window.
//!
//! GPUI 0.3.5 only builds its AccessKit tree after the platform adapter calls
//! the activation callback it receives through `PlatformWindow::a11y_init`.
//! The crate-private `TestWindow` behind `TestAppContext` and
//! `HeadlessAppContext` ignores that callback, so those windows never activate
//! accessibility.
//!
//! This module supplies a minimal, public-API-only `Platform` whose windows
//! keep the callbacks and record each `TreeUpdate` GPUI sends to
//! `PlatformWindow::a11y_tree_update`. It is mounted through GPUI's public
//! `VisualTestAppContext::new(Rc<dyn Platform>)`, which wraps it in
//! `VisualTestPlatform` for deterministic `TestDispatcher` scheduling. Nothing
//! here builds accessibility nodes: the recorded tree is the exact payload that
//! GPUI's own frame lifecycle hands to a platform adapter.
//!
//! The windows draw nothing. Use [`crate::HeadlessSession`] for pixels.

use crate::accessibility::{AccessibilityError, AccessibilityTree};
use futures_channel::oneshot;
use gpui_pre::{
    A11yCallbacks, Action, ActivityGuard, AnyView, AnyWindowHandle, App, AppContext as _, AtlasKey,
    AtlasTextureId, AtlasTextureKind, AtlasTile, BackgroundExecutor, Bounds, Capslock,
    ClipboardItem, CursorStyle, DevicePixels, DispatchEventResult, DummyKeyboardMapper, Entity,
    ForegroundExecutor, GpuSpecs, Keymap, Keystroke, Menu, MenuItem, Modifiers, OwnedMenu,
    PathPromptOptions, Pixels, Platform, PlatformAtlas, PlatformDisplay, PlatformInput,
    PlatformInputHandler, PlatformKeyboardLayout, PlatformKeyboardMapper, PlatformTextSystem,
    PlatformWindow, Point, PromptButton, PromptLevel, Render, RequestFrameOptions, Scene, Size,
    Task, ThermalState, TileId, VisualTestAppContext, Window, WindowAppearance,
    WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowHandle, WindowParams,
    WindowVisibility, accesskit, private::anyhow,
};
use raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

/// A headless GPUI window whose accessibility adapter is active.
///
/// Each [`AccessibilitySession::tree`] call normalizes the last AccessKit
/// `TreeUpdate` GPUI produced for the window. Use [`Self::update`] or
/// [`Self::dispatch_keystroke`] to move the view between states; both redraw.
pub struct AccessibilitySession<V: Render + 'static> {
    context: VisualTestAppContext,
    window: WindowHandle<V>,
    probe: Rc<A11yProbe>,
}

impl<V: Render + 'static> AccessibilitySession<V> {
    /// Open `view` in an accessibility-capturing window of `size`.
    ///
    /// `setup` runs before the window opens so callers can install themes,
    /// globals, and key bindings, as with [`crate::HeadlessSession::new`].
    pub fn new(
        view: V,
        size: Size<Pixels>,
        setup: impl FnOnce(&mut App),
    ) -> Result<Self, AccessibilityError> {
        Self::build(size, setup, |_, cx| cx.new(|_| view))
    }

    /// Open a window whose root view is built with access to the window.
    pub fn build(
        size: Size<Pixels>,
        setup: impl FnOnce(&mut App),
        build_root: impl FnOnce(&mut Window, &mut App) -> Entity<V>,
    ) -> Result<Self, AccessibilityError> {
        // Real text shaping keeps layout faithful; everything else is owned by
        // the capture platform or GPUI's `VisualTestPlatform` wrapper.
        let text_platform = gpui_platform::current_platform(true);
        let platform = Rc::new(CapturePlatform::new(text_platform.text_system()));
        let mut context = VisualTestAppContext::new(platform.clone());
        context.update(setup);
        let window = context
            .open_offscreen_window(size, build_root)
            .map_err(|error| AccessibilityError::Gpui(error.to_string()))?;
        let probe = platform.probe(window.into()).ok_or(AccessibilityError::Inactive)?;
        probe.activate()?;
        let mut session = Self { context, window, probe };
        // Activation schedules a refresh task; run it, then draw so GPUI's
        // active frame produces and delivers a full TreeUpdate.
        session.context.run_until_parked();
        session.redraw()?;
        Ok(session)
    }

    /// The window handle, for helpers that take an `AnyWindowHandle`.
    pub fn window_handle(&self) -> AnyWindowHandle {
        self.window.into()
    }

    /// Update the root view, window, or app, then draw a fresh frame.
    pub fn update<R>(
        &mut self,
        update: impl FnOnce(Entity<V>, &mut Window, &mut App) -> R,
    ) -> Result<R, AccessibilityError> {
        let result = self.with_window(|root, window, cx| {
            let root = root.downcast::<V>().expect("accessibility root type changed");
            update(root, window, cx)
        })?;
        self.context.run_until_parked();
        self.redraw()?;
        Ok(result)
    }

    /// Dispatch a GPUI keystroke string such as `"space"` or `"shift-tab"`,
    /// then draw a fresh frame.
    pub fn dispatch_keystroke(&mut self, keystroke: &str) -> Result<(), AccessibilityError> {
        let keystroke = Keystroke::parse(keystroke)
            .map_err(|error| AccessibilityError::Gpui(error.to_string()))?;
        self.with_window(|_, window, cx| window.dispatch_keystroke(keystroke, cx))?;
        self.context.run_until_parked();
        self.redraw()
    }

    /// Normalize the last AccessKit tree GPUI delivered to this window.
    pub fn tree(&self) -> Result<AccessibilityTree, AccessibilityError> {
        let update = self.probe.last_update.borrow();
        let update = update.as_ref().ok_or(AccessibilityError::NoFrame)?;
        AccessibilityTree::from_tree_update(update).map_err(AccessibilityError::InvalidTree)
    }

    /// How many tree updates GPUI has delivered since activation.
    pub fn update_count(&self) -> usize {
        self.probe.update_count.get()
    }

    /// Deactivate accessibility as a platform adapter would when the last
    /// assistive client disconnects. Later frames stop delivering trees.
    pub fn deactivate(&mut self) -> Result<(), AccessibilityError> {
        self.probe.deactivate()?;
        self.context.run_until_parked();
        self.redraw()
    }

    fn redraw(&mut self) -> Result<(), AccessibilityError> {
        self.with_window(|_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
    }

    fn with_window<R>(
        &mut self,
        f: impl FnOnce(AnyView, &mut Window, &mut App) -> R,
    ) -> Result<R, AccessibilityError> {
        self.context
            .update_window(self.window.into(), f)
            .map_err(|error| AccessibilityError::Gpui(error.to_string()))
    }
}

/// Accessibility adapter state for one window.
#[derive(Default)]
struct A11yProbe {
    callbacks: RefCell<Option<A11yCallbacks>>,
    last_update: RefCell<Option<accesskit::TreeUpdate>>,
    update_count: Cell<usize>,
}

impl A11yProbe {
    fn activate(&self) -> Result<(), AccessibilityError> {
        let callbacks = self.callbacks.borrow();
        let callbacks = callbacks.as_ref().ok_or(AccessibilityError::Inactive)?;
        // GPUI returns a placeholder root-only tree here and schedules a real
        // frame; the harness waits for that frame instead of using it.
        let _placeholder = (callbacks.activation)();
        Ok(())
    }

    fn deactivate(&self) -> Result<(), AccessibilityError> {
        let callbacks = self.callbacks.borrow();
        let callbacks = callbacks.as_ref().ok_or(AccessibilityError::Inactive)?;
        (callbacks.deactivation)();
        Ok(())
    }
}

/// A minimal headless platform. `VisualTestPlatform` supplies executors,
/// clipboard, credentials, and prompts in front of it; this type only needs to
/// provide text shaping, keyboard mapping, and capture windows.
struct CapturePlatform {
    text_system: Arc<dyn PlatformTextSystem>,
    probes: RefCell<Vec<(AnyWindowHandle, Rc<A11yProbe>)>>,
    fallback_executor: RefCell<Option<(BackgroundExecutor, ForegroundExecutor)>>,
}

impl CapturePlatform {
    fn new(text_system: Arc<dyn PlatformTextSystem>) -> Self {
        Self {
            text_system,
            probes: RefCell::new(Vec::new()),
            fallback_executor: RefCell::new(None),
        }
    }

    fn probe(&self, handle: AnyWindowHandle) -> Option<Rc<A11yProbe>> {
        self.probes
            .borrow()
            .iter()
            .find(|(candidate, _)| *candidate == handle)
            .map(|(_, probe)| probe.clone())
    }

    fn executors(&self) -> (BackgroundExecutor, ForegroundExecutor) {
        self.fallback_executor
            .borrow_mut()
            .get_or_insert_with(|| {
                let dispatcher = Arc::new(gpui_pre::TestDispatcher::new(0));
                (BackgroundExecutor::new(dispatcher.clone()), ForegroundExecutor::new(dispatcher))
            })
            .clone()
    }
}

fn ready_none<T>() -> oneshot::Receiver<anyhow::Result<Option<T>>> {
    let (tx, rx) = oneshot::channel();
    tx.send(Ok(None)).ok();
    rx
}

impl Platform for CapturePlatform {
    // `VisualTestPlatform` answers executor queries itself; these exist only to
    // satisfy the trait.
    fn background_executor(&self) -> BackgroundExecutor {
        self.executors().0
    }

    fn foreground_executor(&self) -> ForegroundExecutor {
        self.executors().1
    }

    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        self.text_system.clone()
    }

    fn run(&self, _on_finish_launching: Box<dyn 'static + FnOnce()>) {
        panic!("the accessibility capture platform has no run loop")
    }

    fn quit(&self) {}
    fn restart(&self, _binary_path: Option<PathBuf>, _arguments: Vec<std::ffi::OsString>) {}
    fn activate(&self, _ignoring_other_apps: bool) {}
    fn hide(&self) {}
    fn hide_other_apps(&self) {}
    fn unhide_other_apps(&self) {}

    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        Vec::new()
    }

    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        None
    }

    fn active_window(&self) -> Option<AnyWindowHandle> {
        None
    }

    fn open_window(
        &self,
        handle: AnyWindowHandle,
        options: WindowParams,
    ) -> anyhow::Result<Box<dyn PlatformWindow>> {
        let probe = Rc::new(A11yProbe::default());
        self.probes.borrow_mut().push((handle, probe.clone()));
        Ok(Box::new(CaptureWindow {
            bounds: Cell::new(options.bounds),
            probe,
            atlas: Arc::new(CaptureAtlas::default()),
            input_handler: None,
        }))
    }

    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn open_url(&self, _url: &str) {}
    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}

    fn register_url_scheme(&self, _url: &str) -> Task<anyhow::Result<()>> {
        Task::ready(Ok(()))
    }

    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<anyhow::Result<Option<Vec<PathBuf>>>> {
        ready_none()
    }

    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<anyhow::Result<Option<PathBuf>>> {
        ready_none()
    }

    fn can_select_mixed_files_and_dirs(&self) -> bool {
        true
    }

    fn reveal_path(&self, _path: &Path) {}
    fn open_with_system(&self, _path: &Path) {}
    fn on_quit(&self, _callback: Box<dyn FnMut() -> bool>) {}
    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}
    fn on_system_sleep(&self, _callback: Box<dyn FnMut()>) {}
    fn on_system_wake(&self, _callback: Box<dyn FnMut()>) {}
    fn set_menus(&self, _menus: Vec<Menu>, _keymap: &Keymap) {}

    fn get_menus(&self) -> Option<Vec<OwnedMenu>> {
        None
    }

    fn set_dock_menu(&self, _menu: Vec<MenuItem>, _keymap: &Keymap) {}
    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn Action)>) {}
    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}
    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn Action) -> bool>) {}

    fn thermal_state(&self) -> ThermalState {
        ThermalState::Nominal
    }

    fn on_thermal_state_change(&self, _callback: Box<dyn FnMut()>) {}

    fn prevent_idle_sleep(&self, reason: &str) -> Task<anyhow::Result<ActivityGuard>> {
        Task::ready(Err(anyhow::anyhow!(
            "idle sleep prevention for {reason:?} is unavailable in accessibility capture"
        )))
    }

    fn app_path(&self) -> anyhow::Result<PathBuf> {
        Err(anyhow::anyhow!("the accessibility capture platform has no app bundle"))
    }

    fn path_for_auxiliary_executable(&self, name: &str) -> anyhow::Result<PathBuf> {
        Err(anyhow::anyhow!("no auxiliary executable {name:?} in accessibility capture"))
    }

    fn set_cursor_style(&self, _style: CursorStyle) {}
    fn hide_cursor_until_mouse_moves(&self) {}

    fn is_cursor_visible(&self) -> bool {
        true
    }

    fn should_auto_hide_scrollbars(&self) -> bool {
        false
    }

    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        None
    }

    fn write_to_clipboard(&self, _item: ClipboardItem) {}

    fn read_from_find_pasteboard(&self) -> Option<ClipboardItem> {
        None
    }

    fn write_to_find_pasteboard(&self, _item: ClipboardItem) {}

    fn write_credentials(
        &self,
        _url: &str,
        _username: &str,
        _password: &[u8],
    ) -> Task<anyhow::Result<()>> {
        Task::ready(Ok(()))
    }

    fn read_credentials(&self, _url: &str) -> Task<anyhow::Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Ok(None))
    }

    fn delete_credentials(&self, _url: &str) -> Task<anyhow::Result<()>> {
        Task::ready(Ok(()))
    }

    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(CaptureKeyboardLayout)
    }

    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(DummyKeyboardMapper)
    }

    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
}

struct CaptureKeyboardLayout;

impl PlatformKeyboardLayout for CaptureKeyboardLayout {
    fn id(&self) -> &str {
        "mkit.harness.a11y"
    }

    fn name(&self) -> &str {
        "mkit.harness.a11y"
    }
}

/// A window with no pixels, input source, or display: GPUI still lays out and
/// paints the element tree into a `Scene`, which is discarded.
struct CaptureWindow {
    bounds: Cell<Bounds<Pixels>>,
    probe: Rc<A11yProbe>,
    atlas: Arc<CaptureAtlas>,
    input_handler: Option<PlatformInputHandler>,
}

impl HasWindowHandle for CaptureWindow {
    fn window_handle(&self) -> Result<raw_window_handle::WindowHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

impl HasDisplayHandle for CaptureWindow {
    fn display_handle(&self) -> Result<raw_window_handle::DisplayHandle<'_>, HandleError> {
        Err(HandleError::NotSupported)
    }
}

impl PlatformWindow for CaptureWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        self.bounds.get()
    }

    fn is_maximized(&self) -> bool {
        false
    }

    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Windowed(self.bounds())
    }

    fn content_size(&self) -> Size<Pixels> {
        self.bounds().size
    }

    fn resize(&mut self, size: Size<Pixels>) {
        let mut bounds = self.bounds.get();
        bounds.size = size;
        self.bounds.set(bounds);
    }

    fn scale_factor(&self) -> f32 {
        1.0
    }

    fn appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        None
    }

    fn mouse_position(&self) -> Point<Pixels> {
        Point::default()
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers::default()
    }

    fn capslock(&self) -> Capslock {
        Capslock::default()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        self.input_handler = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.input_handler.take()
    }

    fn prompt(
        &self,
        _level: PromptLevel,
        _msg: &str,
        _detail: Option<&str>,
        _answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        None
    }

    fn activate(&self) {}

    fn is_active(&self) -> bool {
        false
    }

    fn visibility(&self) -> WindowVisibility {
        WindowVisibility::Visible
    }

    fn is_hovered(&self) -> bool {
        false
    }

    fn background_appearance(&self) -> WindowBackgroundAppearance {
        WindowBackgroundAppearance::Opaque
    }

    fn set_title(&mut self, _title: &str) {}
    fn set_background_appearance(&self, _background_appearance: WindowBackgroundAppearance) {}
    fn minimize(&self) {}
    fn zoom(&self) {}
    fn toggle_fullscreen(&self) {}

    fn is_fullscreen(&self) -> bool {
        false
    }

    // Frames are drawn explicitly by `AccessibilitySession`, so platform
    // frame, input, and lifecycle callbacks are intentionally dropped.
    fn on_request_frame(&self, _callback: Box<dyn FnMut(RequestFrameOptions)>) {}
    fn on_input(&self, _callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {}
    fn on_active_status_change(&self, _callback: Box<dyn FnMut(bool)>) {}
    fn on_visibility_change(&self, _callback: Box<dyn FnMut(WindowVisibility)>) {}
    fn on_hover_status_change(&self, _callback: Box<dyn FnMut(bool)>) {}
    fn on_resize(&self, _callback: Box<dyn FnMut(Size<Pixels>, f32)>) {}
    fn on_moved(&self, _callback: Box<dyn FnMut()>) {}
    fn on_should_close(&self, _callback: Box<dyn FnMut() -> bool>) {}
    fn on_hit_test_window_control(&self, _callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
    }
    fn on_close(&self, _callback: Box<dyn FnOnce()>) {}
    fn on_appearance_changed(&self, _callback: Box<dyn FnMut()>) {}
    fn draw(&self, _scene: &Scene) {}

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.atlas.clone()
    }

    fn is_subpixel_rendering_supported(&self) -> bool {
        false
    }

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        None
    }

    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {}

    fn a11y_init(&self, callbacks: A11yCallbacks) {
        *self.probe.callbacks.borrow_mut() = Some(callbacks);
    }

    fn a11y_tree_update(&self, tree_update: accesskit::TreeUpdate) {
        *self.probe.last_update.borrow_mut() = Some(tree_update);
        self.probe.update_count.set(self.probe.update_count.get() + 1);
    }
}

/// Hands out stable fake tiles so glyph and SVG paint paths run without a GPU.
#[derive(Default)]
struct CaptureAtlas {
    tiles: Mutex<(u32, HashMap<AtlasKey, AtlasTile>)>,
}

impl PlatformAtlas for CaptureAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> anyhow::Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> anyhow::Result<Option<AtlasTile>> {
        if let Some(tile) = self.tiles.lock().expect("atlas lock").1.get(key) {
            return Ok(Some(*tile));
        }
        let Some((size, _)) = build()? else {
            return Ok(None);
        };
        let mut tiles = self.tiles.lock().expect("atlas lock");
        tiles.0 += 1;
        let tile = AtlasTile {
            texture_id: AtlasTextureId { index: 0, kind: AtlasTextureKind::Monochrome },
            tile_id: TileId(tiles.0),
            padding: 0,
            bounds: Bounds { origin: Point::default(), size },
        };
        tiles.1.insert(key.clone(), tile);
        Ok(Some(tile))
    }

    fn remove(&self, key: &AtlasKey) {
        self.tiles.lock().expect("atlas lock").1.remove(key);
    }
}
