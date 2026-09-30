//! E8 component previews.

use gpui_pre::{
    Bounds, Context, Entity, IntoElement, KeyBinding, Pixels, Render, Window, actions, div, fill,
    point, prelude::*, px, size,
};
use mkit::{
    colour_tools::{Colour, ColourTools, Srgb},
    command_palette::{CommandAction, CommandPalette},
    core::theme::Theme,
    curve_editor::{CurveChannel, CurveEditor, CurvePoint, Interpolation},
    gradient_editor::{Colour as GradientColour, Gradient, GradientEditor, GradientStop},
    histogram::Histogram,
    layer_panel::{LayerKind, LayerNode, LayerPanel},
    node_editor::{GraphConnection, GraphData, GraphNode, GraphPort, GraphTransform, NodeEditor},
    precision_slider::PrecisionSlider,
    property_inspector::{Property, PropertyGroup, PropertyInspector, PropertyKind, PropertyValue},
    shortcut_editor::{ShortcutAction, ShortcutEditor},
    timeline::{Clip, Keyframe, KeyframeId, TimeRange, Timeline, TimelineObject, Track},
    viewport::{Guide, ViewTransform, Viewport},
};
use std::collections::BTreeMap;

// ANCHOR: shortcut_editor_host_keymap
actions!(shortcut_editor_host, [OpenFile, SaveFile, TogglePalette]);
/// Adapt a loaded shortcut-editor keymap to this host's statically typed GPUI actions.
pub fn shortcut_editor_host_bindings(bindings: &BTreeMap<String, String>) -> Vec<KeyBinding> {
    let mut key_bindings = Vec::new();
    if let Some(chord) = bindings.get("open") {
        key_bindings.push(KeyBinding::new(chord, OpenFile, None));
    }
    if let Some(chord) = bindings.get("save") {
        key_bindings.push(KeyBinding::new(chord, SaveFile, None));
    }
    if let Some(chord) = bindings.get("palette") {
        key_bindings.push(KeyBinding::new(chord, TogglePalette, None));
    }
    key_bindings
}
// ANCHOR_END: shortcut_editor_host_keymap

#[cfg(test)]
mod shortcut_editor_host_tests {
    use super::shortcut_editor_host_bindings;
    use std::collections::BTreeMap;

    #[test]
    fn host_adapter_installs_only_known_actions_present_in_loaded_keymap() {
        let bindings = BTreeMap::from([
            ("open".into(), "cmd-o".into()),
            ("save".into(), "cmd-p".into()),
            ("unknown".into(), "cmd-u".into()),
        ]);
        assert_eq!(shortcut_editor_host_bindings(&bindings).len(), 2);
        assert!(shortcut_editor_host_bindings(&BTreeMap::new()).is_empty());
    }
}

// ANCHOR: e8_preview
#[derive(Default)]
pub struct E8Preview {
    viewport: Option<Entity<Viewport>>,
    precision_slider: Option<Entity<PrecisionSlider>>,
}

impl Render for E8Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let viewport = self
            .viewport
            .get_or_insert_with(|| {
                cx.new(|_| {
                    Viewport::new(
                        "Artwork canvas",
                        point(400.0, 260.0),
                        move |bounds, transform, window| {
                            paint_artwork(bounds, transform, window, theme);
                        },
                    )
                    .show_rulers(true)
                    .guides(vec![Guide::Vertical(135.0), Guide::Horizontal(72.0)])
                })
            })
            .clone();
        let precision_slider = self
            .precision_slider
            .get_or_insert_with(|| {
                cx.new(|_| {
                    PrecisionSlider::new("Tone balance", -30.0, -100.0, 100.0, 1.0)
                        .bipolar(true)
                        .reset_value(0.0)
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.large))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Pro-app components"))
            .child(
                div()
                    .text_size(px(theme.typography.body))
                    .text_color(theme.colors.text_muted)
                    .child(
                        "Drag the canvas to pan. Wheel to zoom. Use Fit or 100% in the toolbar.",
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(theme.spacing.large))
                    .child(div().w(px(610.0)).h(px(450.0)).child(viewport))
                    .child(
                        div()
                            .w(px(280.0))
                            .flex()
                            .flex_col()
                            .gap(px(theme.spacing.small))
                            .child(
                                div()
                                    .text_size(px(theme.typography.heading_small))
                                    .child("Exposure"),
                            )
                            .child(
                                Histogram::new(
                                    "Exposure histogram",
                                    "Most tones are in the middle; highlights are clipped.",
                                )
                                .luminance(vec![
                                    0.12, 0.16, 0.2, 0.28, 0.42, 0.62, 0.87, 1.0, 0.92, 0.76, 0.64,
                                    0.48, 0.34, 0.29, 0.38, 0.46, 0.51, 0.39, 0.24, 0.18, 0.12,
                                    0.08, 0.05, 0.03,
                                ])
                                .clipping(false, true)
                                .height(100.0),
                            )
                            .child(
                                div()
                                    .mt(px(theme.spacing.medium))
                                    .text_size(px(theme.typography.heading_small))
                                    .child("Tone balance"),
                            )
                            .child(precision_slider),
                    ),
            )
    }
}
// ANCHOR_END: e8_preview

// ANCHOR: command_palette_preview
#[derive(Default)]
pub struct CommandPalettePreview {
    palette: Option<Entity<CommandPalette>>,
}

impl Render for CommandPalettePreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let palette = self
            .palette
            .get_or_insert_with(|| {
                cx.new(|_| {
                    CommandPalette::new(
                        vec![
                            CommandAction::new("open-file", "Open File")
                                .group("File")
                                .keybinding("⌘O"),
                            CommandAction::new("save-file", "Save File")
                                .group("File")
                                .keybinding("⌘S"),
                            CommandAction::new("toggle-sidebar", "Toggle Sidebar")
                                .group("View")
                                .keybinding("⌘B"),
                            CommandAction::new("show-settings", "Open Settings")
                                .group("App")
                                .keybinding("⌘,"),
                        ],
                        true,
                    )
                })
            })
            .clone();
        div().size_full().bg(theme.colors.background).child(palette)
    }
}
// ANCHOR_END: command_palette_preview

// ANCHOR: curve_editor_preview
#[derive(Default)]
pub struct CurveEditorPreview {
    curve: Option<Entity<CurveEditor>>,
}

impl Render for CurveEditorPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let curve = self
            .curve
            .get_or_insert_with(|| {
                cx.new(|_| {
                    CurveEditor::new(
                        "Tone curve",
                        vec![
                            CurveChannel::new(
                                "Master",
                                vec![
                                    CurvePoint::new(0.23, 0.14),
                                    CurvePoint::new(0.55, 0.72),
                                    CurvePoint::new(0.82, 0.91),
                                ],
                            ),
                            CurveChannel::new("Red", Vec::new()),
                            CurveChannel::new("Green", Vec::new()),
                            CurveChannel::new("Blue", Vec::new()),
                        ],
                    )
                    .interpolation(Interpolation::Smooth)
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Tone curve"))
            .child(div().w(px(560.0)).p(px(theme.spacing.medium)).child(curve))
    }
}
// ANCHOR_END: curve_editor_preview

// ANCHOR: property_inspector_preview
#[derive(Default)]
pub struct PropertyInspectorPreview {
    inspector: Option<Entity<PropertyInspector>>,
}

pub struct PropertyInspectorMatrixPreview {
    pub state: &'static str,
    inspector: Option<Entity<PropertyInspector>>,
}

impl PropertyInspectorMatrixPreview {
    pub fn for_state(state: &'static str) -> Self {
        Self { state, inspector: None }
    }

    pub fn inspector(&self) -> Option<Entity<PropertyInspector>> {
        self.inspector.clone()
    }
}

fn property_inspector_groups(state: &str) -> Vec<PropertyGroup> {
    let mixed = state == "mixed";
    let edited = state == "edited";
    let appearance = PropertyGroup::new(
        "appearance",
        "Appearance",
        vec![
            Property::new(
                "name",
                "Name",
                PropertyKind::Text,
                if mixed {
                    PropertyValue::Mixed
                } else {
                    PropertyValue::Text(if edited { "Hero card 2" } else { "Hero card" }.into())
                },
                PropertyValue::Text("Layer".into()),
            ),
            Property::new(
                "fill",
                "Fill",
                PropertyKind::Color,
                PropertyValue::Color(if edited { "#5881d7" } else { "#4776d0" }.into()),
                PropertyValue::Color("#ffffff".into()),
            ),
            Property::new(
                "blend",
                "Blend mode",
                PropertyKind::Enum {
                    options: vec!["Normal".into(), "Multiply".into(), "Screen".into()],
                },
                PropertyValue::Enum(if edited { "Multiply" } else { "Normal" }.into()),
                PropertyValue::Enum("Normal".into()),
            ),
        ],
    );
    let appearance = if state == "collapsed" { appearance.collapsed() } else { appearance };
    vec![
        PropertyGroup::new(
            "transform",
            "Transform",
            vec![
                Property::new(
                    "position",
                    "Position",
                    PropertyKind::Vector { dimensions: 2, step: 1.0 },
                    if mixed {
                        PropertyValue::Mixed
                    } else {
                        PropertyValue::Vector(vec![128.0, 64.0])
                    },
                    PropertyValue::Vector(vec![0.0, 0.0]),
                ),
                Property::new(
                    "opacity",
                    "Opacity",
                    PropertyKind::Number { step: 0.05 },
                    if mixed {
                        PropertyValue::Mixed
                    } else {
                        PropertyValue::Number(if edited { 0.70 } else { 0.85 })
                    },
                    PropertyValue::Number(1.0),
                ),
                Property::new(
                    "visible",
                    "Visible",
                    PropertyKind::Boolean,
                    PropertyValue::Boolean(true),
                    PropertyValue::Boolean(true),
                ),
            ],
        ),
        appearance,
        PropertyGroup::new(
            "selection",
            "Multi-selection",
            vec![Property::new(
                "rotation",
                "Rotation",
                PropertyKind::Number { step: 1.0 },
                PropertyValue::Mixed,
                PropertyValue::Number(0.0),
            )],
        ),
    ]
}

impl Render for PropertyInspectorMatrixPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let inspector = self
            .inspector
            .get_or_insert_with(|| {
                cx.new(|_| {
                    PropertyInspector::new(property_inspector_groups(state))
                        .disabled(state == "disabled")
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Property inspector"))
            .child(div().w(px(640.0)).child(inspector))
    }
}

impl Render for PropertyInspectorPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let inspector = self
            .inspector
            .get_or_insert_with(|| {
                cx.new(|_| PropertyInspector::new(property_inspector_groups("expanded")))
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Property inspector"))
            .child(div().w(px(640.0)).child(inspector))
    }
}
// ANCHOR_END: property_inspector_preview

// ANCHOR: colour_tools_preview
#[derive(Default)]
pub struct ColourToolsPreview {
    tools: Option<Entity<ColourTools>>,
}

impl Render for ColourToolsPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let tools = self
            .tools
            .get_or_insert_with(|| {
                cx.new(|_| {
                    ColourTools::new(
                        "Artwork colour",
                        Colour::from_srgb(Srgb { red: 71, green: 118, blue: 208 }),
                    )
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Colour tools"))
            .child(div().w(px(610.0)).child(tools))
    }
}
// ANCHOR_END: colour_tools_preview

// ANCHOR: layer_panel_preview
#[derive(Default)]
pub struct LayerPanelPreview {
    panel: Option<Entity<LayerPanel>>,
}

impl Render for LayerPanelPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let panel = self
            .panel
            .get_or_insert_with(|| {
                cx.new(|_| {
                    LayerPanel::new(
                        "Artwork layers",
                        vec![
                            LayerNode::group(
                                "hero",
                                "Hero composition",
                                vec![
                                    LayerNode::layer("title", "Title and caption")
                                        .with_swatch(gpui_pre::rgb(0x4776d0)),
                                    LayerNode::layer("shape", "Blue shape")
                                        .with_swatch(gpui_pre::rgb(0x7d9de5)),
                                    LayerNode::layer("shadow", "Soft shadow")
                                        .with_swatch(gpui_pre::rgb(0x9ca3af)),
                                ],
                            ),
                            LayerNode::layer("background", "Background")
                                .with_swatch(gpui_pre::rgb(0xf5f5f4)),
                        ],
                    )
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Layer panel"))
            .child(
                div()
                    .w(px(520.0))
                    .p(px(theme.spacing.medium))
                    .rounded(px(theme.radii.medium))
                    .border(px(theme.borders.hairline))
                    .border_color(theme.colors.border)
                    .bg(theme.colors.elevated_surface)
                    .child(panel),
            )
    }
}
// ANCHOR_END: layer_panel_preview

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayerPanelVisualState {
    #[default]
    MixedTree,
    SelectedLayer,
    CollapsedGroup,
    Reordered,
    Dragging,
    Focused,
}

#[derive(Default)]
pub struct LayerPanelMatrixPreview {
    panel: Option<Entity<LayerPanel>>,
    pub state: LayerPanelVisualState,
}

impl LayerPanelMatrixPreview {
    pub fn for_state(state: LayerPanelVisualState) -> Self {
        Self { panel: None, state }
    }

    pub fn panel(&self) -> Option<Entity<LayerPanel>> {
        self.panel.clone()
    }
}

fn layer_panel_matrix_data(state: LayerPanelVisualState) -> (Vec<LayerNode>, String) {
    let mut children = vec![
        LayerNode::layer("title", "Title and caption").with_swatch(gpui_pre::rgb(0x4776d0)),
        LayerNode::layer("shape", "Blue shape").with_swatch(gpui_pre::rgb(0x7d9de5)),
        LayerNode::layer("shadow", "Soft shadow").with_swatch(gpui_pre::rgb(0x9ca3af)),
    ];
    // A hidden layer and a locked layer, so every state shows both toggle glyphs.
    if let LayerKind::Layer { visible, .. } = &mut children[2].kind {
        *visible = false;
    }
    let mut background =
        LayerNode::layer("background", "Background").with_swatch(gpui_pre::rgb(0xf5f5f4));
    if let LayerKind::Layer { locked, .. } = &mut background.kind {
        *locked = true;
    }
    let active = match state {
        LayerPanelVisualState::SelectedLayer => "shape",
        LayerPanelVisualState::Reordered => {
            children.reverse();
            "shadow"
        }
        LayerPanelVisualState::Dragging => "shadow",
        _ => "hero",
    };
    let mut group = LayerNode::group("hero", "Hero composition", children);
    if state == LayerPanelVisualState::CollapsedGroup
        && let LayerKind::Group { expanded, .. } = &mut group.kind
    {
        *expanded = false;
    }
    (
        vec![group, background],
        active.into(),
    )
}

impl Render for LayerPanelMatrixPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let (layers, active) = layer_panel_matrix_data(state);
        let panel = self
            .panel
            .get_or_insert_with(|| {
                cx.new(move |cx| {
                    let mut panel = LayerPanel::new("Artwork layers", layers);
                    panel.set_active(Some(active), cx);
                    panel
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.medium))
            .child(panel)
    }
}

// ANCHOR: gradient_editor_preview
#[derive(Default)]
pub struct GradientEditorPreview {
    editor: Option<Entity<GradientEditor>>,
}

impl Render for GradientEditorPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let editor = self
            .editor
            .get_or_insert_with(|| {
                cx.new(|_| {
                    GradientEditor::new(
                        "Hero gradient",
                        Gradient::new(vec![
                            GradientStop::new(0.0, GradientColour::new(0.08, 0.27, 0.72)),
                            GradientStop::new(0.38, GradientColour::new(0.30, 0.53, 0.89)),
                            GradientStop::new(0.72, GradientColour::new(0.74, 0.40, 0.72)),
                            GradientStop::new(1.0, GradientColour::new(0.94, 0.62, 0.47)),
                        ]),
                    )
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Gradient editor"))
            .child(div().w(px(700.0)).child(editor))
    }
}
// ANCHOR_END: gradient_editor_preview

// ANCHOR: shortcut_editor_preview
#[derive(Default)]
pub struct ShortcutEditorPreview {
    editor: Option<Entity<ShortcutEditor>>,
}

impl Render for ShortcutEditorPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let editor = self
            .editor
            .get_or_insert_with(|| {
                cx.new(|_| {
                    ShortcutEditor::new(
                        vec![
                            ShortcutAction::new("open", "Open file").category("File"),
                            ShortcutAction::new("save", "Save file").category("File"),
                            ShortcutAction::new("palette", "Command palette").category("View"),
                            ShortcutAction::new("sidebar", "Toggle sidebar").category("View"),
                        ],
                        BTreeMap::from([
                            ("open".into(), "cmd-o".into()),
                            ("save".into(), "cmd-s".into()),
                            ("palette".into(), "cmd-k".into()),
                            ("sidebar".into(), "cmd-b".into()),
                        ]),
                    )
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Keyboard shortcuts"))
            .child(div().w(px(660.0)).child(editor))
    }
}
// ANCHOR_END: shortcut_editor_preview

// ANCHOR: timeline_t1_preview
#[derive(Default)]
pub struct TimelineT1Preview {
    timeline: Option<Entity<Timeline>>,
}

impl Render for TimelineT1Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let timeline = self
            .timeline
            .get_or_insert_with(|| {
                cx.new(|_| {
                    Timeline::new(
                        "Scene timeline",
                        vec![
                            Track::new(
                                1,
                                "Video",
                                vec![
                                    Clip::new(11, 0.0, 4.5, "Opening shot"),
                                    Clip::new(12, 5.0, 9.5, "City view"),
                                ],
                            ),
                            Track::new(2, "Music", vec![Clip::new(21, 0.0, 12.0, "Ambient score")]),
                            Track::new(
                                3,
                                "Voice",
                                vec![
                                    Clip::new(31, 1.2, 3.5, "Introduction"),
                                    Clip::new(32, 6.0, 10.8, "Narration"),
                                ],
                            ),
                            Track::new(4, "Titles", vec![Clip::new(41, 0.5, 4.0, "Welcome")]),
                        ],
                        TimeRange::new(0.0, 12.0).expect("valid time range"),
                    )
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div().text_size(px(theme.typography.heading)).child("Timeline: tracks and ruler"),
            )
            .child(div().w(px(800.0)).child(timeline))
    }
}
// ANCHOR_END: timeline_t1_preview

// ANCHOR: timeline_t2_preview
#[derive(Default)]
pub struct TimelineT2Preview {
    timeline: Option<Entity<Timeline>>,
}

impl Render for TimelineT2Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let timeline = self
            .timeline
            .get_or_insert_with(|| {
                cx.new(|cx| {
                    let mut timeline = Timeline::new(
                        "Scene timeline",
                        vec![
                            Track::new(
                                1,
                                "Video",
                                vec![
                                    Clip::new(11, 0.0, 4.5, "Opening shot"),
                                    Clip::new(12, 5.0, 9.5, "City view"),
                                ],
                            ),
                            Track::new(2, "Music", vec![Clip::new(21, 0.0, 12.0, "Ambient score")]),
                            Track::new(3, "Voice", vec![Clip::new(31, 2.0, 7.0, "Narration")]),
                        ],
                        TimeRange::new(0.0, 12.0).expect("valid time range"),
                    );
                    timeline.set_playhead(5.6, cx);
                    timeline.set_selection(Some(TimelineObject::Clip(12)), cx);
                    timeline
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading))
                    .child("Timeline: playhead and selection"),
            )
            .child(div().w(px(800.0)).child(timeline))
    }
}
// ANCHOR_END: timeline_t2_preview

// ANCHOR: timeline_t3_t4_preview
#[derive(Default)]
pub struct TimelineT3T4Preview {
    timeline: Option<Entity<Timeline>>,
}

impl Render for TimelineT3T4Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let timeline = self
            .timeline
            .get_or_insert_with(|| {
                cx.new(|cx| {
                    let mut view = Timeline::new(
                        "Animation timeline",
                        vec![
                            Track::new(
                                1,
                                "Video",
                                vec![
                                    Clip::new(11, 0.0, 4.5, "Opening shot"),
                                    Clip::new(12, 5.0, 9.5, "City view"),
                                ],
                            ),
                            Track::new(2, "Music", vec![Clip::new(21, 0.0, 12.0, "Ambient score")]),
                        ],
                        TimeRange::new(0.0, 12.0).expect("valid time range"),
                    );
                    view.set_playhead(5.6, cx);
                    view.set_selection(Some(TimelineObject::Clip(12)), cx);
                    view.set_keyframes(
                        vec![
                            Keyframe::new(
                                KeyframeId(101),
                                TimelineObject::Clip(12),
                                5.6,
                                "Opacity",
                            ),
                            Keyframe::new(
                                KeyframeId(102),
                                TimelineObject::Clip(12),
                                7.25,
                                "Opacity",
                            ),
                        ],
                        cx,
                    );
                    view.request_clip_move(12, 1, 6.5, cx);
                    view
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading))
                    .child("Timeline: clip editing and keyframes"),
            )
            .child(div().w(px(800.0)).child(timeline))
    }
}
// ANCHOR_END: timeline_t3_t4_preview

// ANCHOR: node_editor_n1_preview
#[derive(Default)]
pub struct NodeEditorN1Preview {
    editor: Option<Entity<NodeEditor>>,
    show_minimap: bool,
}

impl Render for NodeEditorN1Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let editor = self
            .editor
            .get_or_insert_with(|| {
                let show_minimap = self.show_minimap;
                cx.new(move |cx| {
                    let mut editor = NodeEditor::new(
                        "Image processing graph",
                        GraphData::new(
                            vec![
                                GraphNode::new("source", "Image source", point(32.0, 105.0))
                                    .outputs(vec![GraphPort::output(
                                        "source.image",
                                        "Image",
                                        "RGBA",
                                    )]),
                                GraphNode::new("adjust", "Colour adjustment", point(300.0, 50.0))
                                    .inputs(vec![GraphPort::input("adjust.image", "Image", "RGBA")])
                                    .outputs(vec![GraphPort::output(
                                        "adjust.output",
                                        "Adjusted",
                                        "RGBA",
                                    )]),
                                GraphNode::new("preview", "Preview", point(565.0, 150.0)).inputs(
                                    vec![GraphPort::input("preview.image", "Image", "RGBA")],
                                ),
                            ],
                            vec![
                                GraphConnection::new("c1", "source.image", "adjust.image"),
                                GraphConnection::new("c2", "adjust.output", "preview.image"),
                            ],
                        ),
                    );
                    if show_minimap {
                        editor.set_minimap_visible(true, cx);
                    }
                    editor
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div().text_size(px(theme.typography.heading)).child("Node editor: graph surface"),
            )
            .child(div().w(px(850.0)).h(px(420.0)).child(editor))
    }
}
// ANCHOR_END: node_editor_n1_preview

pub struct NodeEditorMatrixPreview {
    pub state: &'static str,
    editor: Option<Entity<NodeEditor>>,
}

impl NodeEditorMatrixPreview {
    pub fn for_state(state: &'static str) -> Self {
        Self { state, editor: None }
    }

    pub fn editor(&self) -> Option<Entity<NodeEditor>> {
        self.editor.clone()
    }
}

fn node_editor_matrix_graph(state: &str) -> GraphData {
    if state == "empty" {
        return GraphData::new(Vec::new(), Vec::new());
    }
    let nodes =
        vec![
            GraphNode::new("source", "Image source", point(35.0, 25.0)).outputs(vec![
                GraphPort::output("source.rgba", "RGBA", "Image"),
                GraphPort::output("source.mask", "Mask", "Mask"),
            ]),
            GraphNode::new("adjust", "Colour adjustment", point(360.0, 125.0))
                .inputs(vec![
                    GraphPort::input("adjust.image", "Image", "Image"),
                    GraphPort::input("adjust.mask", "Mask", "Mask"),
                ])
                .outputs(vec![GraphPort::output("adjust.out", "Adjusted", "Image")]),
            GraphNode::new("preview", "Preview", point(690.0, 85.0))
                .inputs(vec![GraphPort::input("preview.image", "Image", "Image")]),
        ];
    let connections = if matches!(
        state,
        "connected"
            | "selected"
            | "zoomed"
            | "connection_selected"
            | "minimap_visible"
            | "minimap_hidden"
    ) {
        vec![
            GraphConnection::new("c1", "source.rgba", "adjust.image"),
            GraphConnection::new("c2", "adjust.out", "preview.image"),
        ]
    } else {
        Vec::new()
    };
    GraphData::new(nodes, connections)
}

impl Render for NodeEditorMatrixPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let state = self.state;
        let editor = self
            .editor
            .get_or_insert_with(|| {
                cx.new(|cx| {
                    let mut editor =
                        NodeEditor::new("Image processing graph", node_editor_matrix_graph(state));
                    if state == "selected" || state == "drag_preview" {
                        editor.set_selection(vec!["source".into(), "adjust".into()], cx);
                    }
                    if state == "zoomed" {
                        editor.set_transform(
                            GraphTransform { scale: 1.35, offset_x: -40.0, offset_y: -18.0 },
                            cx,
                        );
                    }
                    if state == "minimap_visible" {
                        editor.set_minimap_visible(true, cx);
                    }
                    editor
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(div().text_size(px(theme.typography.heading)).child("Node editor"))
            .child(div().w(px(850.0)).h(px(420.0)).child(editor))
    }
}

// ANCHOR: node_editor_n3_preview
pub type NodeEditorN3Preview = NodeEditorN1Preview;
// ANCHOR_END: node_editor_n3_preview

// ANCHOR: node_editor_n4_preview
#[derive(Default)]
pub struct NodeEditorN4Preview {
    editor: Option<Entity<NodeEditor>>,
}

impl Render for NodeEditorN4Preview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let editor = self
            .editor
            .get_or_insert_with(|| {
                cx.new(|cx| {
                    let mut editor = NodeEditor::new(
                        "Large compositing graph",
                        GraphData::new(
                            vec![
                                GraphNode::new("source", "Image source", point(-180.0, 60.0))
                                    .outputs(vec![GraphPort::output(
                                        "source.out",
                                        "Image",
                                        "RGBA",
                                    )]),
                                GraphNode::new("grade", "Colour grade", point(220.0, -80.0))
                                    .inputs(vec![GraphPort::input("grade.in", "Image", "RGBA")])
                                    .outputs(vec![GraphPort::output(
                                        "grade.out",
                                        "Adjusted",
                                        "RGBA",
                                    )]),
                                GraphNode::new("blur", "Blur", point(220.0, 220.0))
                                    .inputs(vec![GraphPort::input("blur.in", "Image", "RGBA")])
                                    .outputs(vec![GraphPort::output(
                                        "blur.out", "Softened", "RGBA",
                                    )]),
                                GraphNode::new("merge", "Merge", point(660.0, 60.0))
                                    .inputs(vec![
                                        GraphPort::input("merge.a", "A", "RGBA"),
                                        GraphPort::input("merge.b", "B", "RGBA"),
                                    ])
                                    .outputs(vec![GraphPort::output("merge.out", "Image", "RGBA")]),
                                GraphNode::new("output", "Output", point(1040.0, 60.0))
                                    .inputs(vec![GraphPort::input("output.in", "Image", "RGBA")]),
                            ],
                            vec![
                                GraphConnection::new("c1", "source.out", "grade.in"),
                                GraphConnection::new("c2", "grade.out", "merge.a"),
                                GraphConnection::new("c3", "source.out", "blur.in"),
                                GraphConnection::new("c4", "blur.out", "merge.b"),
                                GraphConnection::new("c5", "merge.out", "output.in"),
                            ],
                        ),
                    );
                    editor.set_minimap_visible(true, cx);
                    editor
                })
            })
            .clone();
        let _ = window;
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading))
                    .child("Node editor: minimap navigation"),
            )
            .child(div().w(px(850.0)).h(px(420.0)).child(editor))
    }
}
// ANCHOR_END: node_editor_n4_preview

// ANCHOR: node_editor_n2_preview
#[derive(Default)]
pub struct NodeEditorN2Preview {
    editor: Option<Entity<NodeEditor>>,
}

impl Render for NodeEditorN2Preview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let editor = self
            .editor
            .get_or_insert_with(|| {
                cx.new(|cx| {
                    let mut editor = NodeEditor::new(
                        "Compositing graph",
                        GraphData::new(
                            vec![
                                GraphNode::new("image", "Image", point(34.0, 130.0)).outputs(vec![
                                    GraphPort::output("image.rgba", "RGBA", "Image"),
                                ]),
                                GraphNode::new("exposure", "Exposure", point(300.0, 42.0))
                                    .inputs(vec![GraphPort::input("exposure.in", "Image", "Image")])
                                    .outputs(vec![GraphPort::output(
                                        "exposure.out",
                                        "Adjusted",
                                        "Image",
                                    )]),
                                GraphNode::new("contrast", "Contrast", point(300.0, 205.0))
                                    .inputs(vec![GraphPort::input("contrast.in", "Image", "Image")])
                                    .outputs(vec![GraphPort::output(
                                        "contrast.out",
                                        "Adjusted",
                                        "Image",
                                    )]),
                                GraphNode::new("output", "Output", point(566.0, 130.0))
                                    .inputs(vec![GraphPort::input("output.in", "Image", "Image")]),
                            ],
                            vec![
                                GraphConnection::new("c1", "image.rgba", "exposure.in"),
                                GraphConnection::new("c2", "exposure.out", "output.in"),
                                GraphConnection::new("c3", "image.rgba", "contrast.in"),
                            ],
                        ),
                    );
                    editor.set_selection(vec!["exposure".into(), "contrast".into()], cx);
                    editor
                })
            })
            .clone();
        div()
            .size_full()
            .bg(theme.colors.background)
            .text_color(theme.colors.text)
            .p(px(theme.spacing.xlarge))
            .flex()
            .flex_col()
            .gap(px(theme.spacing.medium))
            .child(
                div()
                    .text_size(px(theme.typography.heading))
                    .child("Node editor: moving a selection"),
            )
            .child(div().w(px(850.0)).h(px(420.0)).child(editor))
    }
}
// ANCHOR_END: node_editor_n2_preview

fn paint_artwork(
    bounds: Bounds<Pixels>,
    transform: ViewTransform,
    window: &mut Window,
    theme: Theme,
) {
    window.paint_quad(fill(bounds, theme.colors.background));
    let origin = transform.world_to_screen(point(0.0, 0.0));
    let left = bounds.origin.x.as_f32() + origin.x;
    let top = bounds.origin.y.as_f32() + origin.y;
    let width = 400.0 * transform.scale;
    let height = 260.0 * transform.scale;
    window.paint_quad(fill(
        Bounds::new(point(px(left), px(top)), size(px(width), px(height))),
        theme.colors.surface,
    ));
    for index in 1..8 {
        let x = left + index as f32 * 50.0 * transform.scale;
        window.paint_quad(fill(
            Bounds::new(point(px(x), px(top)), size(px(theme.borders.hairline), px(height))),
            theme.colors.border,
        ));
    }
    for index in 1..6 {
        let y = top + index as f32 * 43.0 * transform.scale;
        window.paint_quad(fill(
            Bounds::new(point(px(left), px(y)), size(px(width), px(theme.borders.hairline))),
            theme.colors.border,
        ));
    }
    let accent = transform.world_to_screen(point(135.0, 72.0));
    window.paint_quad(
        fill(
            Bounds::new(
                point(
                    px(bounds.origin.x.as_f32() + accent.x),
                    px(bounds.origin.y.as_f32() + accent.y),
                ),
                size(px(130.0 * transform.scale), px(105.0 * transform.scale)),
            ),
            theme.colors.accent,
        )
        .corner_radii(px(theme.radii.medium)),
    );
}
