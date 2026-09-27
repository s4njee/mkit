//! Platform setup, accessible semantics, and local diagnostics in one small GPUI app.

use gpui_pre::{
    App, Bounds, Context, Element, ElementId, IntoElement, LayoutId, Pixels, Render, Role, Style,
    TitlebarOptions, Window, WindowAppearance, WindowBounds, WindowDecorations, WindowOptions, div,
    fill, point, prelude::*, px, rgb, size,
};
use std::sync::{Arc, Mutex, OnceLock};

// ANCHOR: window_options
/// Build explicit, conservative options for the primary app window.
pub fn primary_window_options() -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(80.), px(60.)),
            size(px(760.), px(540.)),
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Platform shipping example".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        window_min_size: Some(size(px(600.), px(420.))),
        window_decorations: Some(WindowDecorations::Server),
        ..WindowOptions::default()
    }
}

/// Options for the companion diagnostics window.
pub fn diagnostics_window_options() -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(180.), px(130.)),
            size(px(420.), px(280.)),
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Diagnostics".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        ..WindowOptions::default()
    }
}
// ANCHOR_END: window_options

// ANCHOR: accessible_custom_element
/// Custom element with a stable id and explicit AccessKit semantics.
pub struct ShippingStatus;

impl IntoElement for ShippingStatus {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ShippingStatus {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(ElementId::Name("shipping-status".into()))
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = px(12.).into();
        style.size.height = px(12.).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _: Option<&gpui_pre::GlobalElementId>,
        _: Option<&gpui_pre::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        _: &mut App,
    ) {
        window.paint_quad(fill(bounds, rgb(0x2d6b51)));
    }

    fn a11y_role(&self) -> Option<Role> {
        Some(Role::Status)
    }

    fn write_a11y_info(&self, node: &mut gpui_pre::accesskit::Node) {
        node.set_label("Shipping status: ready");
        node.set_description("The local example is ready to run.");
    }
}
// ANCHOR_END: accessible_custom_element

#[derive(Default)]
pub struct ShippingDemo {
    activations: u32,
}

impl Render for ShippingDemo {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &gpui_kit::component::ActiveTheme::theme(&**cx).colors;
        let appearance = match window.appearance() {
            WindowAppearance::Light | WindowAppearance::VibrantLight => "Light",
            WindowAppearance::Dark | WindowAppearance::VibrantDark => "Dark",
        };
        let scale = window.scale_factor();
        let activations = self.activations;

        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(div().text_2xl().child("Platform shipping"))
            .child("Window options, appearance, scale, accessibility, and diagnostics")
            .child(format!("Appearance: {appearance} · Scale factor: {scale:.2}"))
            .child(div().flex().items_center().gap_2().child(ShippingStatus).child("Ready to ship"))
            .child(
                div()
                    .id("diagnostics-action")
                    .role(Role::Button)
                    .aria_label("Record a local diagnostic")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.activations += 1;
                        cx.notify();
                    }))
                    .px_3()
                    .py_2()
                    .bg(colors.primary)
                    .text_color(colors.primary_foreground)
                    .child(format!("Record local diagnostic · {activations}")),
            )
    }
}

pub struct DiagnosticsWindow;

impl Render for DiagnosticsWindow {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &gpui_kit::component::ActiveTheme::theme(&**cx).colors;
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child("Diagnostics stay on this machine")
    }
}

// ANCHOR: multiple_windows
/// Open a companion window; its typed handle is returned independently from the main window.
pub fn open_diagnostics(
    cx: &mut App,
) -> gpui_pre::Result<gpui_pre::WindowHandle<DiagnosticsWindow>> {
    cx.open_window(diagnostics_window_options(), |_, cx| cx.new(|_| DiagnosticsWindow))
}
// ANCHOR_END: multiple_windows

// ANCHOR: appearance_scale
/// Read the per-window appearance and scale values used by GPUI layout and rendering.
pub fn window_environment(window: &Window) -> (WindowAppearance, f32) {
    (window.appearance(), window.scale_factor())
}
// ANCHOR_END: appearance_scale

// ANCHOR: diagnostics_setup
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticEntry {
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Default)]
pub struct LocalDiagnosticSink(Arc<Mutex<Vec<DiagnosticEntry>>>);

impl LocalDiagnosticSink {
    pub fn entries(&self) -> Vec<DiagnosticEntry> {
        self.0.lock().expect("diagnostic lock poisoned").clone()
    }

    fn push(&self, entry: DiagnosticEntry) {
        self.0.lock().expect("diagnostic lock poisoned").push(entry);
    }
}

impl log::Log for LocalDiagnosticSink {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            self.push(DiagnosticEntry {
                level: record.level().to_string(),
                target: record.target().to_owned(),
                message: record.args().to_string(),
            });
        }
    }

    fn flush(&self) {}
}

static DIAGNOSTIC_SINK: OnceLock<LocalDiagnosticSink> = OnceLock::new();

/// Install a local-only logger. The app intentionally does not upload or send diagnostics.
pub fn install_diagnostics() -> Result<LocalDiagnosticSink, &'static str> {
    let sink = DIAGNOSTIC_SINK.get_or_init(LocalDiagnosticSink::default);
    log::set_logger(sink).map_err(|_| "another global logger is already installed")?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(sink.clone())
}

/// Report the example's tagged local error through the installed logging facade.
pub fn record_tagged_error(tag: &str) {
    log::error!(target: "mkit_example_platform_shipping", "example error tag={tag}");
}

/// Convert a panic payload into a local diagnostic entry for a panic hook to retain.
pub fn record_panic_message(sink: &LocalDiagnosticSink, message: &str) {
    sink.push(DiagnosticEntry {
        level: "PANIC".into(),
        target: "panic-hook".into(),
        message: message.into(),
    });
}

/// Preserve the platform's normal panic report while retaining a local summary.
pub fn install_panic_hook(sink: LocalDiagnosticSink) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(String::as_str))
            .unwrap_or("non-string panic payload");
        record_panic_message(&sink, message);
        previous(info);
    }));
}
// ANCHOR_END: diagnostics_setup

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::TestAppContext;

    #[gpui_kit::test]
    fn window_options_and_two_typed_windows_have_independent_roots(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let first = cx.add_window(|_, _| ShippingDemo::default());
        let second = cx.add_window(|_, _| ShippingDemo::default());
        cx.update(|app| {
            first.update(app, |view, _, _| view.activations += 1).unwrap();
            assert_eq!(first.read(app).unwrap().activations, 1);
            assert_eq!(second.read(app).unwrap().activations, 0);
        });
        assert_eq!(primary_window_options().window_min_size.unwrap().width, px(600.));
        assert_eq!(
            diagnostics_window_options().titlebar.unwrap().title.unwrap().as_ref(),
            "Diagnostics"
        );
    }

    #[gpui_kit::test]
    fn scale_and_appearance_are_read_from_the_window(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (_, visual) = cx.add_window_view(|_, _| ShippingDemo::default());
        visual.update(|window, cx| {
            window.set_scale_factor(1.5);
            let (appearance, scale) = window_environment(window);
            assert!(matches!(
                appearance,
                WindowAppearance::Light
                    | WindowAppearance::VibrantLight
                    | WindowAppearance::Dark
                    | WindowAppearance::VibrantDark
            ));
            assert_eq!(scale, 1.5);
            window.draw(cx).clear(cx);
        });
    }

    #[gpui_kit::test]
    fn custom_accessibility_semantics_have_stable_id_and_action(cx: &mut TestAppContext) {
        cx.update(gpui_kit::init);
        let (view, visual) = cx.add_window_view(|_, _| ShippingDemo::default());
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let element = ShippingStatus;
        assert_eq!(element.id(), Some(ElementId::Name("shipping-status".into())));
        let mut node = gpui_pre::accesskit::Node::new(Role::Status);
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Shipping status: ready"));
        assert_eq!(node.description(), Some("The local example is ready to run."));
        assert_eq!(node.role(), Role::Status);
        assert!(!node.supports_action(gpui_pre::accesskit::Action::Click));
        let tree = visual.update(|window, _| {
            let active = window.is_a11y_active();
            (active, window.debug_a11y_tree_json())
        });
        if let Some(json) = tree.1 {
            assert!(json.contains("Shipping status: ready"));
        } else {
            assert!(!tree.0, "no accessibility tree without an active adapter");
        }
        let _ = view;
    }

    #[test]
    fn diagnostics_retain_tagged_errors_and_panic_summary_locally() {
        let sink = install_diagnostics().expect("this fixture installs the one process logger");
        record_tagged_error("E211");
        let panic = std::panic::catch_unwind(|| panic!("fixture panic summary"))
            .expect_err("the fixture panic is caught for this local diagnostic test");
        let message =
            panic.downcast_ref::<&str>().copied().expect("the fixture panic has a string payload");
        record_panic_message(&sink, message);
        let entries = sink.entries();
        assert!(entries.iter().any(|entry| {
            entry.target == "mkit_example_platform_shipping" && entry.message.contains("E211")
        }));
        assert!(
            entries.iter().any(|entry| {
                entry.level == "PANIC" && entry.message == "fixture panic summary"
            })
        );
    }
}
