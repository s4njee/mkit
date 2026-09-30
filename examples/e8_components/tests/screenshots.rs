use gpui_pre::{
    App, Focusable, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PlatformInput, Render, point, px, size,
};
use mkit::core::theme::{self, HIGH_CONTRAST, LIGHT, SHADCN_DARK, SHADCN_LIGHT};
use mkit_example_e8_components::{
    ColourToolsPreview, CommandPalettePreview, CurveEditorPreview, E8Preview,
    GradientEditorPreview, LayerPanelMatrixPreview, LayerPanelVisualState, NodeEditorMatrixPreview,
    NodeEditorN1Preview, NodeEditorN2Preview, NodeEditorN3Preview, NodeEditorN4Preview,
    PropertyInspectorMatrixPreview, ShortcutEditorPreview, TimelineT1Preview, TimelineT2Preview,
    TimelineT3T4Preview,
};
use mkit_harness::{HeadlessSession, PixelTolerance, screenshot};
use std::path::PathBuf;

fn main() {
    #[cfg(target_os = "macos")]
    run();
}

#[cfg(target_os = "macos")]
fn run() {
    let cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-preview-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-preview-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-preview-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-preview-dark-2x.png"),
    ];
    for (palette, scale, relative) in cases {
        if !selected_case(relative) {
            continue;
        }
        let actual =
            screenshot(E8Preview::default(), size(px(960.0), px(560.0)), scale, |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            })
            .expect("capture E8 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save E8 screenshot baseline");
            continue;
        }
        let expected = image::open(&path).expect("E8 screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "E8 preview screenshot differs: {}",
            path.display()
        );
    }
    let palette_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-command-palette-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-command-palette-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-command-palette-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-command-palette-dark-2x.png"),
    ];
    for (palette, scale, relative) in palette_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            CommandPalettePreview::default(),
            size(px(720.0), px(380.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture command palette preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save command palette screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("command palette screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "command palette preview screenshot differs: {}",
            path.display()
        );
    }
    let curve_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-curve-editor-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-curve-editor-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-curve-editor-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-curve-editor-dark-2x.png"),
    ];
    for (palette, scale, relative) in curve_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            CurveEditorPreview::default(),
            size(px(720.0), px(430.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture curve editor preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save curve editor screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("curve editor screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "curve editor preview screenshot differs: {}",
            path.display()
        );
    }
    let inspector_states = ["expanded", "collapsed", "mixed", "edited", "disabled", "focused"];
    let inspector_themes =
        [("light", SHADCN_LIGHT), ("dark", SHADCN_DARK), ("high-contrast", HIGH_CONTRAST)];
    let inspector_baselines = [
        "../../book/src/images/e8-property-inspector-expanded-light-1x.png",
        "../../book/src/images/e8-property-inspector-expanded-light-2x.png",
        "../../book/src/images/e8-property-inspector-expanded-dark-1x.png",
        "../../book/src/images/e8-property-inspector-expanded-dark-2x.png",
        "../../book/src/images/e8-property-inspector-expanded-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-expanded-high-contrast-2x.png",
        "../../book/src/images/e8-property-inspector-collapsed-light-1x.png",
        "../../book/src/images/e8-property-inspector-collapsed-light-2x.png",
        "../../book/src/images/e8-property-inspector-collapsed-dark-1x.png",
        "../../book/src/images/e8-property-inspector-collapsed-dark-2x.png",
        "../../book/src/images/e8-property-inspector-collapsed-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-collapsed-high-contrast-2x.png",
        "../../book/src/images/e8-property-inspector-mixed-light-1x.png",
        "../../book/src/images/e8-property-inspector-mixed-light-2x.png",
        "../../book/src/images/e8-property-inspector-mixed-dark-1x.png",
        "../../book/src/images/e8-property-inspector-mixed-dark-2x.png",
        "../../book/src/images/e8-property-inspector-mixed-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-mixed-high-contrast-2x.png",
        "../../book/src/images/e8-property-inspector-edited-light-1x.png",
        "../../book/src/images/e8-property-inspector-edited-light-2x.png",
        "../../book/src/images/e8-property-inspector-edited-dark-1x.png",
        "../../book/src/images/e8-property-inspector-edited-dark-2x.png",
        "../../book/src/images/e8-property-inspector-edited-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-edited-high-contrast-2x.png",
        "../../book/src/images/e8-property-inspector-disabled-light-1x.png",
        "../../book/src/images/e8-property-inspector-disabled-light-2x.png",
        "../../book/src/images/e8-property-inspector-disabled-dark-1x.png",
        "../../book/src/images/e8-property-inspector-disabled-dark-2x.png",
        "../../book/src/images/e8-property-inspector-disabled-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-disabled-high-contrast-2x.png",
        "../../book/src/images/e8-property-inspector-focused-light-1x.png",
        "../../book/src/images/e8-property-inspector-focused-light-2x.png",
        "../../book/src/images/e8-property-inspector-focused-dark-1x.png",
        "../../book/src/images/e8-property-inspector-focused-dark-2x.png",
        "../../book/src/images/e8-property-inspector-focused-high-contrast-1x.png",
        "../../book/src/images/e8-property-inspector-focused-high-contrast-2x.png",
    ];
    for (state_index, state) in inspector_states.into_iter().enumerate() {
        for (theme_index, (_, palette)) in inspector_themes.into_iter().enumerate() {
            for (scale_index, scale) in [1.0, 2.0].into_iter().enumerate() {
                let relative =
                    inspector_baselines[(state_index * 3 + theme_index) * 2 + scale_index];
                if !selected_case(relative) {
                    continue;
                }
                let actual = capture_property_inspector(state, palette, scale);
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
                if save_candidate(&actual, &path) {
                    continue;
                }
                if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
                    actual.save(&path).expect("save property inspector screenshot baseline");
                    continue;
                }
                let expected = image::open(&path)
                    .expect("property inspector screenshot baseline exists")
                    .to_rgba8();
                assert!(
                    PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                        .matches(&expected, &actual),
                    "property inspector preview screenshot differs: {}",
                    path.display()
                );
            }
        }
    }
    let colour_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-colour-tools-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-colour-tools-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-colour-tools-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-colour-tools-dark-2x.png"),
    ];
    for (palette, scale, relative) in colour_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            ColourToolsPreview::default(),
            size(px(760.0), px(540.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture colour tools preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save colour tools screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("colour tools screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "colour tools preview screenshot differs: {}",
            path.display()
        );
    }
    let layer_states = [
        ("mixed-tree", LayerPanelVisualState::MixedTree),
        ("selected-layer", LayerPanelVisualState::SelectedLayer),
        ("collapsed-group", LayerPanelVisualState::CollapsedGroup),
        ("reordered", LayerPanelVisualState::Reordered),
        ("dragging", LayerPanelVisualState::Dragging),
        ("focused", LayerPanelVisualState::Focused),
    ];
    let layer_themes =
        [("light", SHADCN_LIGHT), ("dark", SHADCN_DARK), ("high-contrast", HIGH_CONTRAST)];
    let layer_baselines = [
        "../../book/src/images/e8-layer-panel-mixed-tree-light-1x.png",
        "../../book/src/images/e8-layer-panel-mixed-tree-light-2x.png",
        "../../book/src/images/e8-layer-panel-mixed-tree-dark-1x.png",
        "../../book/src/images/e8-layer-panel-mixed-tree-dark-2x.png",
        "../../book/src/images/e8-layer-panel-mixed-tree-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-mixed-tree-high-contrast-2x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-light-1x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-light-2x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-dark-1x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-dark-2x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-selected-layer-high-contrast-2x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-light-1x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-light-2x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-dark-1x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-dark-2x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-collapsed-group-high-contrast-2x.png",
        "../../book/src/images/e8-layer-panel-reordered-light-1x.png",
        "../../book/src/images/e8-layer-panel-reordered-light-2x.png",
        "../../book/src/images/e8-layer-panel-reordered-dark-1x.png",
        "../../book/src/images/e8-layer-panel-reordered-dark-2x.png",
        "../../book/src/images/e8-layer-panel-reordered-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-reordered-high-contrast-2x.png",
        "../../book/src/images/e8-layer-panel-dragging-light-1x.png",
        "../../book/src/images/e8-layer-panel-dragging-light-2x.png",
        "../../book/src/images/e8-layer-panel-dragging-dark-1x.png",
        "../../book/src/images/e8-layer-panel-dragging-dark-2x.png",
        "../../book/src/images/e8-layer-panel-dragging-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-dragging-high-contrast-2x.png",
        "../../book/src/images/e8-layer-panel-focused-light-1x.png",
        "../../book/src/images/e8-layer-panel-focused-light-2x.png",
        "../../book/src/images/e8-layer-panel-focused-dark-1x.png",
        "../../book/src/images/e8-layer-panel-focused-dark-2x.png",
        "../../book/src/images/e8-layer-panel-focused-high-contrast-1x.png",
        "../../book/src/images/e8-layer-panel-focused-high-contrast-2x.png",
    ];
    for (state_index, (_state_name, state)) in layer_states.into_iter().enumerate() {
        for (theme_index, (_theme_name, palette)) in layer_themes.into_iter().enumerate() {
            for (scale_index, scale) in [1.0, 2.0].into_iter().enumerate() {
                let relative = layer_baselines[(state_index * 3 + theme_index) * 2 + scale_index];
                if !selected_case(relative) {
                    continue;
                }
                let actual = capture_layer_panel(state, palette, scale);
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
                if save_candidate(&actual, &path) {
                    continue;
                }
                if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
                    actual.save(&path).expect("save layer panel screenshot baseline");
                    continue;
                }
                let expected =
                    image::open(&path).expect("layer panel screenshot baseline exists").to_rgba8();
                assert!(
                    PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                        .matches(&expected, &actual),
                    "layer panel preview screenshot differs: {}",
                    path.display()
                );
            }
        }
    }
    let gradient_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-gradient-editor-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-gradient-editor-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-gradient-editor-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-gradient-editor-dark-2x.png"),
    ];
    for (palette, scale, relative) in gradient_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            GradientEditorPreview::default(),
            size(px(820.0), px(400.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture gradient editor preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save gradient editor screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("gradient editor screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "gradient editor preview screenshot differs: {}",
            path.display()
        );
    }
    let shortcut_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-shortcut-editor-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-shortcut-editor-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-shortcut-editor-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-shortcut-editor-dark-2x.png"),
    ];
    for (palette, scale, relative) in shortcut_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            ShortcutEditorPreview::default(),
            size(px(760.0), px(400.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture shortcut editor preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save shortcut editor screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("shortcut editor screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "shortcut editor preview screenshot differs: {}",
            path.display()
        );
    }
    let timeline_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-timeline-t1-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-timeline-t1-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-timeline-t1-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-timeline-t1-dark-2x.png"),
    ];
    for (palette, scale, relative) in timeline_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            TimelineT1Preview::default(),
            size(px(900.0), px(520.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture timeline T1 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save timeline T1 screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("timeline T1 screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "timeline T1 preview screenshot differs: {}",
            path.display()
        );
    }
    let node_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-node-editor-n1-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-node-editor-n1-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-node-editor-n1-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-node-editor-n1-dark-2x.png"),
    ];
    for (palette, scale, relative) in node_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            NodeEditorN1Preview::default(),
            size(px(960.0), px(540.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture node editor N1 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save node editor screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("node editor screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "node editor preview screenshot differs: {}",
            path.display()
        );
    }
    let timeline_t2_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-timeline-t2-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-timeline-t2-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-timeline-t2-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-timeline-t2-dark-2x.png"),
    ];
    for (palette, scale, relative) in timeline_t2_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            TimelineT2Preview::default(),
            size(px(900.0), px(520.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture timeline T2 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save timeline T2 screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("timeline T2 screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "timeline T2 preview screenshot differs: {}",
            path.display()
        );
    }
    let timeline_t3_t4_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-timeline-t3-t4-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-timeline-t3-t4-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-timeline-t3-t4-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-timeline-t3-t4-dark-2x.png"),
    ];
    for (palette, scale, relative) in timeline_t3_t4_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            TimelineT3T4Preview::default(),
            size(px(900.0), px(520.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture timeline T3/T4 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save timeline T3/T4 screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("timeline T3/T4 screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "timeline T3/T4 preview screenshot differs: {}",
            path.display()
        );
    }
    let node_n2_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-node-editor-n2-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-node-editor-n2-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-node-editor-n2-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-node-editor-n2-dark-2x.png"),
    ];
    for (palette, scale, relative) in node_n2_cases {
        if !selected_case(relative) {
            continue;
        }
        let actual = screenshot(
            NodeEditorN2Preview::default(),
            size(px(960.0), px(540.0)),
            scale,
            |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
            },
        )
        .expect("capture node editor N2 preview");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save node editor N2 screenshot baseline");
            continue;
        }
        let expected =
            image::open(&path).expect("node editor N2 screenshot baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "node editor N2 preview screenshot differs: {}",
            path.display()
        );
    }
    let node_n3_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-node-editor-n3-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-node-editor-n3-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-node-editor-n3-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-node-editor-n3-dark-2x.png"),
    ];
    for (palette, scale, path) in node_n3_cases {
        capture_node_editor_case(NodeEditorN3Preview::default(), palette, scale, path, "N3");
    }
    let node_n4_cases = [
        (SHADCN_LIGHT, 1.0, "../../book/src/images/e8-node-editor-n4-light-1x.png"),
        (SHADCN_LIGHT, 2.0, "../../book/src/images/e8-node-editor-n4-light-2x.png"),
        (SHADCN_DARK, 1.0, "../../book/src/images/e8-node-editor-n4-dark-1x.png"),
        (SHADCN_DARK, 2.0, "../../book/src/images/e8-node-editor-n4-dark-2x.png"),
    ];
    for (palette, scale, path) in node_n4_cases {
        capture_node_editor_case(NodeEditorN4Preview::default(), palette, scale, path, "N4");
    }

    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../registry/node-editor/tests/conformance.json");
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&manifest_path).expect("read node editor conformance manifest"),
    )
    .expect("parse node editor conformance manifest");
    for case in manifest["screenshot_cases"].as_array().expect("screenshot cases") {
        let state = case["state"].as_str().expect("state name");
        let theme_name = case["theme"].as_str().expect("theme name");
        let scale = case["scale"].as_f64().expect("scale") as f32;
        let palette = match theme_name {
            "light" => LIGHT,
            "dark" => mkit::core::theme::DARK,
            "high-contrast" => HIGH_CONTRAST,
            _ => panic!("unknown Node Editor theme {theme_name}"),
        };
        let relative = format!(
            "../../book/src/images/e8-node-editor-matrix-{state}-{theme_name}-{}x.png",
            scale as u8
        );
        if !selected_case(&relative) {
            continue;
        }
        let actual = capture_node_editor_matrix(state, palette, scale);
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(&relative);
        if save_candidate(&actual, &path) {
            continue;
        }
        let baseline = manifest_path
            .parent()
            .expect("manifest directory")
            .join(case["baseline"].as_str().expect("baseline path"));
        if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
            actual.save(&path).expect("save node editor matrix book screenshot");
            actual.save(&baseline).expect("save node editor manifest screenshot baseline");
            continue;
        }
        let expected = image::open(&path).expect("node editor book baseline exists").to_rgba8();
        assert!(
            PixelTolerance { channel_delta: 2, max_different_pixels: 32 }
                .matches(&expected, &actual),
            "node editor matrix screenshot differs: {}",
            path.display()
        );
    }
}

#[cfg(target_os = "macos")]
fn capture_node_editor_matrix(
    state: &str,
    palette: mkit::core::theme::Theme,
    scale: f32,
) -> image::RgbaImage {
    let state: &'static str = match state {
        "empty" => "empty",
        "disconnected" => "disconnected",
        "connected" => "connected",
        "selected" => "selected",
        "port_navigation" => "port_navigation",
        "zoomed" => "zoomed",
        "drag_preview" => "drag_preview",
        "box_preview" => "box_preview",
        "connection_preview" => "connection_preview",
        "connection_selected" => "connection_selected",
        "invalid_connection_target" => "invalid_connection_target",
        "minimap_visible" => "minimap_visible",
        "minimap_hidden" => "minimap_hidden",
        _ => panic!("unknown Node Editor state {state}"),
    };
    let size = size(px(960.0), px(540.0));
    let view = NodeEditorMatrixPreview::for_state(state);
    let init = |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, palette);
        cx.bind_keys(mkit::node_editor::default_key_bindings());
    };
    let interactive = matches!(
        state,
        "port_navigation"
            | "connection_preview"
            | "connection_selected"
            | "drag_preview"
            | "box_preview"
            | "invalid_connection_target"
    );
    if !interactive {
        return screenshot(view, size, scale, init).expect("capture Node Editor state");
    }

    let mut session = HeadlessSession::new(view, size, scale, init)
        .expect("create Node Editor interaction capture");
    session
        .update(|root, window, cx| {
            let editor = root.read(cx).editor().expect("matrix editor initialized");
            let focus = editor.read(cx).focus_handle(cx);
            focus.focus(window, cx);
        })
        .expect("focus Node Editor for state capture");
    match state {
        "port_navigation" => session.simulate_keystrokes("enter").expect("enter port navigation"),
        "connection_preview" => {
            session.simulate_keystrokes("enter c ]").expect("begin keyboard connection preview")
        }
        "connection_selected" => {
            session.simulate_keystrokes("g").expect("focus existing connection")
        }
        "drag_preview" => dispatch_node_drag(
            &mut session,
            point(px(180.0), px(180.0)),
            point(px(224.0), px(214.0)),
        ),
        "box_preview" => dispatch_node_drag(
            &mut session,
            point(px(700.0), px(400.0)),
            point(px(790.0), px(462.0)),
        ),
        "invalid_connection_target" => {
            let source = point(px(272.0), px(170.0));
            let target = point(px(392.0), px(294.0));
            dispatch_node_drag(&mut session, source, target);
            session
                .update(|_, window, cx| {
                    window.dispatch_event(
                        PlatformInput::MouseUp(MouseUpEvent {
                            position: target,
                            button: MouseButton::Left,
                            modifiers: Modifiers::default(),
                            click_count: 1,
                        }),
                        cx,
                    );
                })
                .expect("release connection on incompatible input");
            let status = session
                .update(|root, _, cx| {
                    let editor = root.read(cx).editor().expect("matrix editor initialized");
                    editor.read(cx).connection_status().map(str::to_owned)
                })
                .expect("read rejected connection status");
            assert!(
                status.as_deref().is_some_and(|message| message.contains("types do not match")),
                "incompatible port drag must expose its typed rejection reason, got {status:?}"
            );
        }
        _ => {}
    }
    session.capture().expect("capture Node Editor interaction state")
}

#[cfg(target_os = "macos")]
fn dispatch_node_drag(
    session: &mut HeadlessSession<NodeEditorMatrixPreview>,
    start: gpui_pre::Point<gpui_pre::Pixels>,
    end: gpui_pre::Point<gpui_pre::Pixels>,
) {
    session
        .update(|_, window, cx| {
            window.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Left,
                    position: start,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            window.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position: end,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Modifiers::default(),
                }),
                cx,
            );
        })
        .expect("dispatch Node Editor pointer drag");
}

#[cfg(target_os = "macos")]
fn capture_property_inspector(
    state: &'static str,
    palette: mkit::core::theme::Theme,
    scale: f32,
) -> image::RgbaImage {
    let size = size(px(760.0), px(540.0));
    let view = PropertyInspectorMatrixPreview::for_state(state);
    let init = |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, palette);
        cx.bind_keys(mkit::property_inspector::default_key_bindings());
    };
    if state == "focused" {
        // Keyboard focus on the root, then ArrowDown, so the active row shows its focus outline.
        let mut session = HeadlessSession::new(view, size, scale, init)
            .expect("create property inspector focus capture");
        session
            .update(|root, window, cx| {
                let inspector = root.read(cx).inspector().expect("matrix inspector initialized");
                let focus = inspector.read(cx).focus_handle(cx);
                focus.focus(window, cx);
            })
            .expect("focus property inspector");
        session.simulate_keystrokes("down").expect("move the active property");
        return session.capture().expect("capture focused property inspector");
    }
    if state != "edited" {
        return screenshot(view, size, scale, init).expect("capture property inspector state");
    }

    let mut session = HeadlessSession::new(view, size, scale, init)
        .expect("create property inspector edit capture");
    // The opacity increment control is at this token-stable position in the 760px preview.
    let edit = point(px(577.0), px(170.0));
    session
        .update(|_, window, cx| {
            window.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Left,
                    position: edit,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
            window.dispatch_event(
                PlatformInput::MouseUp(MouseUpEvent {
                    position: edit,
                    button: MouseButton::Left,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                }),
                cx,
            );
        })
        .expect("apply opacity edit");
    session.capture().expect("capture edited property inspector")
}

#[cfg(target_os = "macos")]
fn capture_layer_panel(
    state: LayerPanelVisualState,
    palette: mkit::core::theme::Theme,
    scale: f32,
) -> image::RgbaImage {
    let view = LayerPanelMatrixPreview::for_state(state);
    if state == LayerPanelVisualState::Focused {
        // Keyboard focus on the panel, then ArrowDown, so the active row shows its focus outline.
        let mut session =
            HeadlessSession::new(view, size(px(660.0), px(260.0)), scale, |cx: &mut App| {
                gpui_kit::base::init(cx);
                theme::set_theme(cx, palette);
                cx.bind_keys(mkit::layer_panel::default_key_bindings());
            })
            .expect("create layer panel focus capture");
        session
            .update(|root, window, cx| {
                let panel = root.read(cx).panel().expect("matrix panel initialized");
                let focus = panel.read(cx).focus_handle(cx);
                focus.focus(window, cx);
            })
            .expect("focus layer panel");
        session.simulate_keystrokes("down").expect("move the active layer");
        return session.capture().expect("capture focused layer panel");
    }
    if state != LayerPanelVisualState::Dragging {
        return screenshot(view, size(px(660.0), px(260.0)), scale, |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, palette);
        })
        .expect("capture layer panel state");
    }

    let mut session =
        HeadlessSession::new(view, size(px(660.0), px(260.0)), scale, |cx: &mut App| {
            gpui_kit::base::init(cx);
            theme::set_theme(cx, palette);
        })
        .expect("create layer panel drag capture");
    let row_height = palette.controls.small;
    let pad = palette.spacing.medium;
    let x = px(180.0);
    let source = point(x, px(pad + row_height * 3.5));
    let target = point(x, px(pad + row_height * 2.5));
    session
        .update(|_, window, cx| {
            window.dispatch_event(
                PlatformInput::MouseDown(MouseDownEvent {
                    button: MouseButton::Left,
                    position: source,
                    modifiers: Modifiers::default(),
                    click_count: 1,
                    first_mouse: false,
                }),
                cx,
            );
        })
        .expect("start layer panel drag");
    session
        .update(|_, window, cx| {
            window.dispatch_event(
                PlatformInput::MouseMove(MouseMoveEvent {
                    position: target,
                    pressed_button: Some(MouseButton::Left),
                    modifiers: Modifiers::default(),
                }),
                cx,
            );
        })
        .expect("move layer panel drag over sibling");
    session.capture().expect("capture active layer drag")
}

#[cfg(target_os = "macos")]
fn capture_node_editor_case<V: Render + 'static>(
    view: V,
    palette: mkit::core::theme::Theme,
    scale: f32,
    relative: &str,
    slice: &str,
) {
    if !selected_case(relative) {
        return;
    }
    let actual = screenshot(view, size(px(960.0), px(540.0)), scale, |cx: &mut App| {
        gpui_kit::base::init(cx);
        theme::set_theme(cx, palette);
    })
    .unwrap_or_else(|error| panic!("capture node editor {slice} preview: {error}"));
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if save_candidate(&actual, &path) {
        return;
    }
    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        actual
            .save(&path)
            .unwrap_or_else(|error| panic!("save node editor {slice} baseline: {error}"));
        return;
    }
    let expected = image::open(&path)
        .unwrap_or_else(|error| {
            panic!("missing node editor {slice} baseline {}: {error}", path.display())
        })
        .to_rgba8();
    assert!(
        PixelTolerance { channel_delta: 2, max_different_pixels: 32 }.matches(&expected, &actual),
        "node editor {slice} preview differs: {}",
        path.display()
    );
}

#[cfg(target_os = "macos")]
fn save_candidate(actual: &image::RgbaImage, baseline: &std::path::Path) -> bool {
    let Some(dir) = std::env::var_os("E8_SNAPSHOT_CANDIDATE_DIR") else { return false };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("create E8 candidate directory");
    actual.save(dir.join(baseline.file_name().expect("baseline name"))).expect("save E8 candidate");
    true
}

#[cfg(target_os = "macos")]
fn selected_case(relative: &str) -> bool {
    std::env::var("E8_SNAPSHOT_ONLY").map_or(true, |names| {
        names.split(',').any(|name| !name.is_empty() && relative.contains(name))
    })
}
