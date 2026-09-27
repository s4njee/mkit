//! Visual preview of the pro-app component drafts in the neutral shadcn palette.
//!
//! Each scene composes the compiling E8 example previews so the gallery shows
//! every pro component in the same window a reader can open. The headless
//! screenshot test captures every scene in the shadcn light and dark themes at
//! 1x and 2x scales.

use gpui_pre::{Context, Entity, Render, Window, div, prelude::*, px};
use mkit::core::theme::{HIGH_CONTRAST, SHADCN_DARK, SHADCN_LIGHT, Theme, set_theme};
use mkit_example_e8_components::{
    ColourToolsPreview, CommandPalettePreview, CurveEditorPreview, E8Preview,
    GradientEditorPreview, LayerPanelPreview, NodeEditorN1Preview, NodeEditorN2Preview,
    PropertyInspectorPreview, ShortcutEditorPreview, TimelineT1Preview, TimelineT2Preview,
};

pub struct ProGallery {
    scene: E8Scene,
    canvas: Option<Entity<E8Preview>>,
    curve: Option<Entity<CurveEditorPreview>>,
    colour: Option<Entity<ColourToolsPreview>>,
    gradient: Option<Entity<GradientEditorPreview>>,
    inspector: Option<Entity<PropertyInspectorPreview>>,
    layers: Option<Entity<LayerPanelPreview>>,
    palette: Option<Entity<CommandPalettePreview>>,
    shortcuts: Option<Entity<ShortcutEditorPreview>>,
    timeline_t1: Option<Entity<TimelineT1Preview>>,
    timeline_t2: Option<Entity<TimelineT2Preview>>,
    nodes_n1: Option<Entity<NodeEditorN1Preview>>,
    nodes_n2: Option<Entity<NodeEditorN2Preview>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum E8Scene {
    Canvas,
    Adjust,
    Inspect,
    Command,
    Time,
    Graph,
}

fn card(t: Theme, title: &'static str) -> gpui_pre::Div {
    div()
        .w_full()
        .p(px(t.spacing.large))
        .flex()
        .flex_col()
        .gap(px(t.spacing.medium))
        .rounded(px(t.radii.medium))
        .border(px(t.borders.hairline))
        .border_color(t.colors.border)
        .bg(t.colors.surface)
        .child(div().text_size(px(t.typography.heading_small)).child(title))
}

fn theme_switcher(t: Theme, cx: &mut Context<ProGallery>) -> gpui_pre::Div {
    let button = |id: &'static str, label: &'static str, selected: bool, theme: Theme| {
        div()
            .id(id)
            .debug_selector(move || id.into())
            .px(px(t.spacing.medium))
            .py(px(t.spacing.xsmall))
            .rounded(px(t.radii.small))
            .border(px(t.borders.hairline))
            .border_color(t.colors.border)
            .bg(if selected { t.colors.accent } else { t.colors.surface })
            .text_color(if selected { t.colors.accent_text } else { t.colors.text })
            .child(label)
            .on_click(cx.listener(move |_, _, _, cx| {
                set_theme(cx, theme);
                cx.notify();
            }))
    };
    div()
        .flex()
        .items_center()
        .gap(px(t.spacing.small))
        .child(div().text_color(t.colors.text_muted).child("Theme"))
        .child(button("e8-theme-light", "Light", t.name == SHADCN_LIGHT.name, SHADCN_LIGHT))
        .child(button("e8-theme-dark", "Dark", t.name == SHADCN_DARK.name, SHADCN_DARK))
        .child(button(
            "e8-theme-contrast",
            "High contrast",
            t.name == HIGH_CONTRAST.name,
            HIGH_CONTRAST,
        ))
}

fn render_scene(
    scene: E8Scene,
    t: Theme,
    owner: &mut ProGallery,
    cx: &mut Context<ProGallery>,
) -> gpui_pre::AnyElement {
    let heading = match scene {
        E8Scene::Canvas => "Canvas",
        E8Scene::Adjust => "Adjust",
        E8Scene::Inspect => "Inspect",
        E8Scene::Command => "Command",
        E8Scene::Time => "Time",
        E8Scene::Graph => "Graph",
    };
    let body = match scene {
        E8Scene::Canvas => {
            let canvas =
                owner.canvas.get_or_insert_with(|| cx.new(|_| E8Preview::default())).clone();
            div().flex().flex_col().gap(px(t.spacing.large)).child(
                card(t, "Viewport · histogram · precision slider").h(px(620.0)).child(canvas),
            )
        }
        E8Scene::Adjust => {
            let curve = owner
                .curve
                .get_or_insert_with(|| cx.new(|_| CurveEditorPreview::default()))
                .clone();
            let colour = owner
                .colour
                .get_or_insert_with(|| cx.new(|_| ColourToolsPreview::default()))
                .clone();
            let gradient = owner
                .gradient
                .get_or_insert_with(|| cx.new(|_| GradientEditorPreview::default()))
                .clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card(t, "Curve editor").h(px(540.0)).child(curve))
                .child(card(t, "Colour tools").h(px(640.0)).child(colour))
                .child(card(t, "Gradient editor").h(px(500.0)).child(gradient))
        }
        E8Scene::Inspect => {
            let inspector = owner
                .inspector
                .get_or_insert_with(|| cx.new(|_| PropertyInspectorPreview::default()))
                .clone();
            let layers = owner
                .layers
                .get_or_insert_with(|| cx.new(|_| LayerPanelPreview::default()))
                .clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card(t, "Property inspector").h(px(560.0)).child(inspector))
                .child(card(t, "Layer panel").h(px(440.0)).child(layers))
        }
        E8Scene::Command => {
            let palette = owner
                .palette
                .get_or_insert_with(|| cx.new(|_| CommandPalettePreview::default()))
                .clone();
            let shortcuts = owner
                .shortcuts
                .get_or_insert_with(|| cx.new(|_| ShortcutEditorPreview::default()))
                .clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card(t, "Command palette · open").h(px(420.0)).child(palette))
                .child(card(t, "Shortcut editor").h(px(460.0)).child(shortcuts))
        }
        E8Scene::Time => {
            let t1 = owner
                .timeline_t1
                .get_or_insert_with(|| cx.new(|_| TimelineT1Preview::default()))
                .clone();
            let t2 = owner
                .timeline_t2
                .get_or_insert_with(|| cx.new(|_| TimelineT2Preview::default()))
                .clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card(t, "Timeline · tracks and ruler").h(px(540.0)).child(t1))
                .child(card(t, "Timeline · playhead and selection").h(px(540.0)).child(t2))
        }
        E8Scene::Graph => {
            let n1 = owner
                .nodes_n1
                .get_or_insert_with(|| cx.new(|_| NodeEditorN1Preview::default()))
                .clone();
            let n2 = owner
                .nodes_n2
                .get_or_insert_with(|| cx.new(|_| NodeEditorN2Preview::default()))
                .clone();
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.large))
                .child(card(t, "Node editor · graph surface").h(px(600.0)).child(n1))
                .child(card(t, "Node editor · movement and box selection").h(px(600.0)).child(n2))
        }
    };
    div()
        .id("mkit-e8-scene")
        .size_full()
        .overflow_y_scroll()
        .bg(t.colors.background)
        .text_color(t.colors.text)
        .p(px(t.spacing.xlarge))
        .flex()
        .flex_col()
        .gap(px(t.spacing.large))
        .child(div().text_size(px(t.typography.heading_large)).child(heading))
        .child(
            div()
                .text_color(t.colors.text_muted)
                .child("E8 pro-app components in the shadcn theme."),
        )
        .child(theme_switcher(t, cx))
        .child(body)
        .into_any_element()
}

impl ProGallery {
    pub fn new() -> Self {
        Self {
            scene: E8Scene::Canvas,
            canvas: None,
            curve: None,
            colour: None,
            gradient: None,
            inspector: None,
            layers: None,
            palette: None,
            shortcuts: None,
            timeline_t1: None,
            timeline_t2: None,
            nodes_n1: None,
            nodes_n2: None,
        }
    }

    pub fn canvas_preview() -> Self {
        Self { scene: E8Scene::Canvas, ..Self::new() }
    }
    pub fn adjust_preview() -> Self {
        Self { scene: E8Scene::Adjust, ..Self::new() }
    }
    pub fn inspect_preview() -> Self {
        Self { scene: E8Scene::Inspect, ..Self::new() }
    }
    pub fn command_preview() -> Self {
        Self { scene: E8Scene::Command, ..Self::new() }
    }
    pub fn time_preview() -> Self {
        Self { scene: E8Scene::Time, ..Self::new() }
    }
    pub fn graph_preview() -> Self {
        Self { scene: E8Scene::Graph, ..Self::new() }
    }
}

impl Default for ProGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for ProGallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *cx.global::<Theme>();
        render_scene(self.scene, t, self, cx)
    }
}
