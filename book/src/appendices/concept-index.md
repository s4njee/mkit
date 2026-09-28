# Concept index

This index covers all **634** public inventory rows at the pinned `gpui-pre` 0.3.5 snapshot. Each source link points to the pinned Zed commit. A chapter link means the named concept is introduced there; it does not promise complete coverage of every method. **Future coverage** means the current book has no verified chapter for that item. See the [API inventory](api-inventory.md) for target and feature-profile caveats.

The index is generated from `book/inventory.md` with a small manually reviewed chapter-route map. Review routes and counts after every inventory refresh.

## Getting started (5)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::App`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686) | struct | [Published chapter](../state/contexts.md) |
| [`gpui::Application`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L144) | struct | [Published chapter](../examples/hello.md) |
| [`gpui::ApplicationHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L151) | struct | Future coverage |
| [`gpui::Window`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144) | struct | [Published chapter](../getting-started/how-gpui-thinks.md) |
| [`gpui::WindowOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2044) | struct | [Published chapter](../examples/hello.md) |

## App, context, and entities (34)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::AnyEntity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L246) | struct | [Published chapter](../architecture/entity-map.md) |
| [`gpui::AnyTooltip`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3038) | struct | Future coverage |
| [`gpui::AnyView`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L19) | struct | Future coverage |
| [`gpui::AnyWeakEntity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L569) | struct | Future coverage |
| [`gpui::AnyWeakView`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L112) | struct | Future coverage |
| [`gpui::AppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::AppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172) | trait | [Published chapter](../state/contexts.md) |
| [`gpui::BorrowAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L300) | trait | Future coverage |
| [`gpui::Context`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21) | struct | [Published chapter](../state/contexts.md) |
| [`gpui::CursorHideMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L343) | enum | Future coverage |
| [`gpui::EmptyView`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L476) | struct | Future coverage |
| [`gpui::Entity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414) | struct | [Published chapter](../state/entities.md) |
| [`gpui::EntityId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L27) | struct | [Published chapter](../architecture/entity-map.md) |
| [`gpui::EventEmitter`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L296) | trait | [Published chapter](../state/reactivity.md) |
| [`gpui::Global`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L22) | trait | [Published chapter](../state/globals.md) |
| [`gpui::GpuiBorrow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3090) | struct | Future coverage |
| [`gpui::LeakDetectorSnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L944) | struct | Future coverage |
| [`gpui::Observation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L193) | struct | Future coverage |
| [`gpui::prelude::Context`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21) | struct | Future coverage |
| [`gpui::prelude::Render`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) | macro | Future coverage |
| [`gpui::prelude::Render`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163) | trait | Future coverage |
| [`gpui::prelude::RenderOnce`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L179) | trait | Future coverage |
| [`gpui::ReadGlobal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L30) | trait | Future coverage |
| [`gpui::Render`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | [Published chapter](../state/views-and-components.md) |
| [`gpui::Render`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163) | trait | [Published chapter](../state/views-and-components.md) |
| [`gpui::RenderOnce`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L179) | trait | [Published chapter](../state/views-and-components.md) |
| [`gpui::Reservation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L249) | struct | Future coverage |
| [`gpui::Subscription`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L150) | struct | [Published chapter](../state/reactivity.md) |
| [`gpui::test::Observation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L193) | struct | Future coverage |
| [`gpui::UpdateGlobal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L44) | trait | Future coverage |
| [`gpui::View`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L182) | trait | Future coverage |
| [`gpui::VisualContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::VisualContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L260) | trait | Future coverage |
| [`gpui::WeakEntity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L740) | struct | [Published chapter](../state/entities.md) |

## Elements, styling, and text (157)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::AbsoluteLength`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3298) | enum | Future coverage |
| [`gpui::AlignContent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1089) | enum | Future coverage |
| [`gpui::AlignItems`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1039) | enum | Future coverage |
| [`gpui::AlignSelf`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1073) | type | Future coverage |
| [`gpui::Along`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L43) | trait | Future coverage |
| [`gpui::Anchor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2165) | enum | Future coverage |
| [`gpui::AnyElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L588) | struct | [Published chapter](../state/views-and-components.md) |
| [`gpui::AnyImageCache`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L24) | struct | Future coverage |
| [`gpui::Asset`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L99) | trait | Future coverage |
| [`gpui::AssetLogger`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L114) | enum | Future coverage |
| [`gpui::AssetSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L13) | trait | Future coverage |
| [`gpui::AvailableSpace`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/taffy.rs#L678) | enum | Future coverage |
| [`gpui::Axis`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L25) | enum | Future coverage |
| [`gpui::Background`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L779) | struct | Future coverage |
| [`gpui::border_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) | macro | Future coverage |
| [`gpui::Boundary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L674) | struct | Future coverage |
| [`gpui::Bounds`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L723) | struct | Future coverage |
| [`gpui::BoundsRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L720) | struct | Future coverage |
| [`gpui::box_shadow_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) | macro | Future coverage |
| [`gpui::BoxShadow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L349) | struct | [Published chapter](../custom-rendering/canvas-primitives.md) |
| [`gpui::colors`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L1) | module | Future coverage |
| [`gpui::colors::Colors`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L9) | struct | Future coverage |
| [`gpui::colors::DefaultAppearance`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L107) | enum | Future coverage |
| [`gpui::colors::DefaultColors`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L92) | trait | Future coverage |
| [`gpui::colors::GlobalColors`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L79) | struct | Future coverage |
| [`gpui::ColorSpace`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L759) | enum | Future coverage |
| [`gpui::ContainerQuery`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/container_query.rs#L52) | struct | Future coverage |
| [`gpui::Corners`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2258) | struct | Future coverage |
| [`gpui::CornersRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2255) | struct | Future coverage |
| [`gpui::cursor_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) | macro | Future coverage |
| [`gpui::DebugBelow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L23) | struct | Future coverage |
| [`gpui::DecorationRun`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L24) | struct | Future coverage |
| [`gpui::DefiniteLength`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3460) | enum | Future coverage |
| [`gpui::DevicePixels`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2982) | struct | Future coverage |
| [`gpui::Display`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1131) | enum | [Published chapter](../elements/div-and-layout.md) |
| [`gpui::Div`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1800) | struct | [Published chapter](../elements/div-and-layout.md) |
| [`gpui::DivFrameState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1848) | struct | Future coverage |
| [`gpui::DivInspectorState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1854) | struct | Future coverage |
| [`gpui::Edges`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L1750) | struct | Future coverage |
| [`gpui::EdgesRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L1747) | struct | Future coverage |
| [`gpui::Empty`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L728) | struct | Future coverage |
| [`gpui::Fill`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L853) | enum | Future coverage |
| [`gpui::FlexDirection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1173) | enum | Future coverage |
| [`gpui::FlexWrap`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1150) | enum | Future coverage |
| [`gpui::Font`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1063) | struct | Future coverage |
| [`gpui::FontFallbacks`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/font_fallbacks.rs#L9) | struct | Future coverage |
| [`gpui::FontFamilyId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L42) | struct | Future coverage |
| [`gpui::FontFeatures`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/font_features.rs#L8) | struct | Future coverage |
| [`gpui::FontId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L38) | struct | Future coverage |
| [`gpui::FontMetrics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1116) | struct | Future coverage |
| [`gpui::FontRun`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L877) | struct | Future coverage |
| [`gpui::FontStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L981) | enum | Future coverage |
| [`gpui::FontWeight`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L899) | struct | Future coverage |
| [`gpui::GlobalElementId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L213) | struct | [Published chapter](../elements/conditional-lists.md) |
| [`gpui::GlyphId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1026) | struct | Future coverage |
| [`gpui::GlyphRasterData`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L15) | struct | Future coverage |
| [`gpui::GridLocation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3793) | struct | Future coverage |
| [`gpui::GridPlacement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3802) | enum | Future coverage |
| [`gpui::GridTemplate`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L170) | struct | Future coverage |
| [`gpui::GridTemplateMinSize`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L145) | enum | Future coverage |
| [`gpui::GridTemplateRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L159) | struct | Future coverage |
| [`gpui::GroupStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L57) | struct | Future coverage |
| [`gpui::Half`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3827) | trait | Future coverage |
| [`gpui::HighlightStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L580) | struct | Future coverage |
| [`gpui::Hsla`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L334) | struct | Future coverage |
| [`gpui::Image`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2774) | struct | [Published chapter](../elements/images-and-svg.md) |
| [`gpui::ImageAssetLoader`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L617) | enum | Future coverage |
| [`gpui::ImageCache`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L197) | trait | Future coverage |
| [`gpui::ImageCacheElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L71) | struct | Future coverage |
| [`gpui::ImageCacheError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L750) | enum | Future coverage |
| [`gpui::ImageCacheItem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L167) | struct | Future coverage |
| [`gpui::ImageCacheProvider`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L210) | trait | Future coverage |
| [`gpui::ImageFormat`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2697) | enum | Future coverage |
| [`gpui::ImageFormatIter`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2696) | struct | Future coverage |
| [`gpui::ImageId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L33) | struct | Future coverage |
| [`gpui::ImageSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L42) | enum | [Published chapter](../elements/images-and-svg.md) |
| [`gpui::ImageStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L129) | struct | Future coverage |
| [`gpui::Img`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L192) | struct | Future coverage |
| [`gpui::ImgLayoutState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L260) | struct | Future coverage |
| [`gpui::ImgResourceLoader`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L38) | type | Future coverage |
| [`gpui::InteractiveText`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L981) | struct | Future coverage |
| [`gpui::IntoElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | [Published chapter](../state/views-and-components.md) |
| [`gpui::IntoElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L145) | trait | [Published chapter](../state/views-and-components.md) |
| [`gpui::IsZero`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3878) | trait | Future coverage |
| [`gpui::JustifyContent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1124) | type | Future coverage |
| [`gpui::JustifyItems`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1066) | type | Future coverage |
| [`gpui::JustifySelf`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1080) | type | Future coverage |
| [`gpui::LayoutId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/taffy.rs#L385) | struct | Future coverage |
| [`gpui::Length`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3611) | enum | Future coverage |
| [`gpui::LinearColorStop`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L883) | struct | Future coverage |
| [`gpui::LineFragment`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L614) | enum | Future coverage |
| [`gpui::LineLayout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L16) | struct | Future coverage |
| [`gpui::LineWrapper`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L17) | struct | Future coverage |
| [`gpui::LineWrapperHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L862) | struct | Future coverage |
| [`gpui::margin_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) | macro | Future coverage |
| [`gpui::ObjectFit`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L29) | enum | [Published chapter](../elements/images-and-svg.md) |
| [`gpui::Overflow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1208) | enum | [Published chapter](../elements/size-and-scroll.md) |
| [`gpui::overflow_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) | macro | Future coverage |
| [`gpui::padding_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) | macro | Future coverage |
| [`gpui::ParentElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L188) | trait | [Published chapter](../elements/div-and-layout.md) |
| [`gpui::Percentage`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2621) | struct | Future coverage |
| [`gpui::Pixels`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2677) | struct | Future coverage |
| [`gpui::Point`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L85) | struct | Future coverage |
| [`gpui::PointRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L67) | struct | Future coverage |
| [`gpui::Position`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1236) | enum | Future coverage |
| [`gpui::position_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) | macro | Future coverage |
| [`gpui::prelude::Element`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51) | trait | Future coverage |
| [`gpui::prelude::IntoElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L145) | trait | Future coverage |
| [`gpui::prelude::ParentElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L188) | trait | Future coverage |
| [`gpui::prelude::Styled`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L22) | trait | Future coverage |
| [`gpui::prelude::StyledImage`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L148) | trait | Future coverage |
| [`gpui::Radians`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2596) | struct | Future coverage |
| [`gpui::Rems`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3238) | struct | Future coverage |
| [`gpui::RenderGlyphParams`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1035) | struct | Future coverage |
| [`gpui::RenderImage`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L43) | struct | [Published chapter](../elements/images-and-svg.md) |
| [`gpui::RenderImageParams`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L37) | struct | Future coverage |
| [`gpui::Resource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L71) | enum | Future coverage |
| [`gpui::RetainAllImageCache`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L222) | struct | Future coverage |
| [`gpui::RetainAllImageCacheProvider`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L312) | struct | Future coverage |
| [`gpui::Rgba`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L39) | struct | Future coverage |
| [`gpui::ScaledPixels`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3075) | struct | Future coverage |
| [`gpui::ScrollAnchor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4191) | struct | Future coverage |
| [`gpui::ScrollHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4245) | struct | Future coverage |
| [`gpui::ShapedGlyph`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L42) | struct | Future coverage |
| [`gpui::ShapedLine`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L43) | struct | Future coverage |
| [`gpui::ShapedRun`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L33) | struct | Future coverage |
| [`gpui::Size`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L396) | struct | Future coverage |
| [`gpui::SizeRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L392) | struct | Future coverage |
| [`gpui::Stateful`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4063) | struct | Future coverage |
| [`gpui::StrikethroughStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L843) | struct | Future coverage |
| [`gpui::StrikethroughStyleRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L841) | struct | Future coverage |
| [`gpui::Style`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L180) | struct | [Published chapter](../elements/size-and-scroll.md) |
| [`gpui::Styled`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L22) | trait | [Published chapter](../elements/div-and-layout.md) |
| [`gpui::StyledImage`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L148) | trait | Future coverage |
| [`gpui::StyledText`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L391) | struct | [Published chapter](../elements/text.md) |
| [`gpui::StyleRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L178) | struct | Future coverage |
| [`gpui::Svg`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/svg.rs#L16) | struct | Future coverage |
| [`gpui::text`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L160) | macro | Future coverage |
| [`gpui::Text`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L67) | struct | Future coverage |
| [`gpui::TextAlign`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L423) | enum | Future coverage |
| [`gpui::TextLayout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L614) | struct | Future coverage |
| [`gpui::TextOverflow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L407) | enum | Future coverage |
| [`gpui::TextRun`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L999) | struct | [Published chapter](../elements/text.md) |
| [`gpui::TextStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L438) | struct | [Published chapter](../elements/text.md) |
| [`gpui::TextStyleRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L436) | struct | Future coverage |
| [`gpui::TextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L51) | struct | [Published chapter](../architecture/text-systems.md) |
| [`gpui::Transformation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/svg.rs#L214) | struct | Future coverage |
| [`gpui::TruncateFrom`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L7) | enum | Future coverage |
| [`gpui::UnderlineStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L828) | struct | Future coverage |
| [`gpui::UnderlineStyleRefinement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L826) | struct | Future coverage |
| [`gpui::Visibility`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L339) | enum | Future coverage |
| [`gpui::visibility_style_methods`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L11) | macro | Future coverage |
| [`gpui::WhiteSpace`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L397) | enum | Future coverage |
| [`gpui::WindowTextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L376) | struct | Future coverage |
| [`gpui::WrapBoundary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L287) | struct | Future coverage |
| [`gpui::WrappedLine`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L267) | struct | Future coverage |
| [`gpui::WrappedLineLayout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L274) | struct | Future coverage |

## Input, actions, and key bindings (93)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::Action`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L4) | macro | [Published chapter](../interaction/actions.md) |
| [`gpui::Action`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L118) | trait | [Published chapter](../interaction/actions.md) |
| [`gpui::ActionBuildError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L194) | enum | Future coverage |
| [`gpui::actions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L25) | macro | [Published chapter](../interaction/actions.md) |
| [`gpui::AnyDrag`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3011) | struct | Future coverage |
| [`gpui::AsKeystroke`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L11) | trait | Future coverage |
| [`gpui::BindingIndex`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L27) | struct | Future coverage |
| [`gpui::ClickEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L290) | enum | Future coverage |
| [`gpui::ClipboardEntry`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2563) | enum | Future coverage |
| [`gpui::ClipboardItem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2522) | struct | [Published chapter](../interaction/clipboard.md) |
| [`gpui::ClipboardReadError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2532) | enum | Future coverage |
| [`gpui::ClipboardString`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2928) | struct | Future coverage |
| [`gpui::ContextEntry`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L14) | struct | Future coverage |
| [`gpui::DispatchEventResult`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L2109) | struct | Future coverage |
| [`gpui::DispatchPhase`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L91) | enum | [Published chapter](../interaction/actions.md) |
| [`gpui::DragMoveEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L67) | struct | Future coverage |
| [`gpui::ElementClickedState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3627) | struct | Future coverage |
| [`gpui::ElementHoverState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3643) | struct | Future coverage |
| [`gpui::ElementId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7218) | enum | [Published chapter](../elements/conditional-lists.md) |
| [`gpui::ExternalDragPayload`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L706) | enum | Future coverage |
| [`gpui::ExternalDragPayloadSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3032) | type | Future coverage |
| [`gpui::ExternalPaths`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L694) | struct | [Published chapter](../interaction/drag-and-drop.md) |
| [`gpui::FileDragPaths`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L714) | struct | Future coverage |
| [`gpui::FileDropEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L737) | enum | [Published chapter](../interaction/drag-and-drop.md) |
| [`gpui::Focusable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L701) | trait | Future coverage |
| [`gpui::FocusHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L527) | struct | [Published chapter](../interaction/focus.md) |
| [`gpui::FocusId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L325) | struct | Future coverage |
| [`gpui::FocusOutEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L320) | struct | Future coverage |
| [`gpui::GestureEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L21) | trait | Future coverage |
| [`gpui::GestureKinds`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L353) | struct | Future coverage |
| [`gpui::GestureTuning`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L114) | struct | Future coverage |
| [`gpui::Hitbox`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L822) | struct | Future coverage |
| [`gpui::HitboxBehavior`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L879) | enum | Future coverage |
| [`gpui::HitboxId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L753) | struct | Future coverage |
| [`gpui::HoverListenerMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1737) | enum | Future coverage |
| [`gpui::InputEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L9) | trait | Future coverage |
| [`gpui::InteractiveElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L774) | trait | [Published chapter](../elements/conditional-lists.md) |
| [`gpui::InteractiveElementState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3605) | struct | Future coverage |
| [`gpui::Interactivity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L2128) | struct | Future coverage |
| [`gpui::InvalidKeystrokeError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L51) | struct | Future coverage |
| [`gpui::KeyBinding`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L10) | struct | [Published chapter](../interaction/actions.md) |
| [`gpui::KeyBindingContextPredicate`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L172) | enum | Future coverage |
| [`gpui::KeybindingKeystroke`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L37) | struct | Future coverage |
| [`gpui::KeyBindingMetaIndex`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L143) | struct | Future coverage |
| [`gpui::KeyboardButton`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L443) | enum | Future coverage |
| [`gpui::KeyboardClickEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L264) | struct | Future coverage |
| [`gpui::KeyContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L10) | struct | [Published chapter](../interaction/actions.md) |
| [`gpui::KeyDownEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L25) | struct | [Published chapter](../getting-started/how-gpui-thinks.md) |
| [`gpui::KeyEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L15) | trait | Future coverage |
| [`gpui::Keymap`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L18) | struct | Future coverage |
| [`gpui::KeymapVersion`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L14) | struct | Future coverage |
| [`gpui::Keystroke`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L18) | struct | [Published chapter](../cookbook/input/rebind-navigation.md) |
| [`gpui::KeystrokeEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3053) | struct | Future coverage |
| [`gpui::KeyUpEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L47) | struct | Future coverage |
| [`gpui::LongPressEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L406) | struct | Future coverage |
| [`gpui::Menu`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L4) | struct | [Published chapter](../interaction/menus.md) |
| [`gpui::MenuItem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L76) | enum | [Published chapter](../interaction/menus.md) |
| [`gpui::Modifiers`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L448) | struct | Future coverage |
| [`gpui::ModifiersChangedEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L62) | struct | Future coverage |
| [`gpui::MouseButton`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L453) | enum | Future coverage |
| [`gpui::MouseClickEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L220) | struct | Future coverage |
| [`gpui::MouseDownEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L148) | struct | [Published chapter](../interaction/mouse.md) |
| [`gpui::MouseEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L18) | trait | Future coverage |
| [`gpui::MouseExitEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L666) | struct | Future coverage |
| [`gpui::MouseMoveEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L494) | struct | Future coverage |
| [`gpui::MousePressureEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L243) | struct | Future coverage |
| [`gpui::MouseUpEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L185) | struct | Future coverage |
| [`gpui::NavigationDirection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L483) | enum | Future coverage |
| [`gpui::NoAction`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L431) | struct | Future coverage |
| [`gpui::NullPlatformGestures`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L452) | struct | Future coverage |
| [`gpui::OngoingScroll`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L49) | struct | Future coverage |
| [`gpui::OsAction`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L311) | enum | Future coverage |
| [`gpui::OsMenu`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L50) | struct | Future coverage |
| [`gpui::OwnedMenu`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L238) | struct | Future coverage |
| [`gpui::OwnedMenuItem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L250) | enum | Future coverage |
| [`gpui::PinchEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L571) | struct | Future coverage |
| [`gpui::PlatformGestures`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L438) | trait | Future coverage |
| [`gpui::PlatformInput`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L771) | enum | Future coverage |
| [`gpui::prelude::InteractiveElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L774) | trait | Future coverage |
| [`gpui::prelude::StatefulInteractiveElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1303) | trait | Future coverage |
| [`gpui::PressureStage`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L230) | enum | Future coverage |
| [`gpui::register_action`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::ScrollDelta`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L554) | enum | Future coverage |
| [`gpui::ScrollPhysics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L151) | enum | Future coverage |
| [`gpui::ScrollWheelEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L522) | struct | [Published chapter](../interaction/mouse.md) |
| [`gpui::StatefulInteractiveElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1303) | trait | [Published chapter](../windows-platform-shipping/accessibility.md) |
| [`gpui::TouchClickEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L275) | struct | Future coverage |
| [`gpui::TouchDragEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L386) | struct | Future coverage |
| [`gpui::TouchEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L119) | struct | Future coverage |
| [`gpui::TouchId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L109) | struct | Future coverage |
| [`gpui::TouchPhase`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L88) | enum | Future coverage |
| [`gpui::Unbind`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L448) | struct | Future coverage |
| [`gpui::WeakFocusHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L666) | struct | Future coverage |

## Text input and IME (10)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::ElementInputHandler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L128) | struct | [Published chapter](../text-input/handler-contract.md) |
| [`gpui::EntityInputHandler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L13) | trait | [Published chapter](../text-input/handler-contract.md) |
| [`gpui::InputHandler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1793) | trait | [Published chapter](../text-input/handler-contract.md) |
| [`gpui::PendingInputStatus`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1297) | struct | Future coverage |
| [`gpui::PendingInputTimeoutStatus`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1316) | struct | Future coverage |
| [`gpui::PlatformInputHandler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1536) | struct | Future coverage |
| [`gpui::TextInputAction`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2022) | enum | Future coverage |
| [`gpui::TextInputConfiguration`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1987) | struct | Future coverage |
| [`gpui::TextInputStateChange`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L864) | enum | Future coverage |
| [`gpui::UTF16Selection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1780) | struct | [Published chapter](../text-input/selection-and-undo.md) |

## Composition and advanced elements (70)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::Anchored`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L16) | struct | [Published chapter](../custom-rendering/overlays.md) |
| [`gpui::AnchoredFitMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L243) | enum | Future coverage |
| [`gpui::AnchoredPositionMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L254) | enum | Future coverage |
| [`gpui::AnchoredState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L10) | struct | Future coverage |
| [`gpui::Animation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L15) | struct | [Published chapter](../custom-rendering/animation.md) |
| [`gpui::AnimationElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L161) | struct | Future coverage |
| [`gpui::AnimationExt`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L82) | trait | [Published chapter](../custom-rendering/animation.md) |
| [`gpui::AnimationPhase`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L328) | struct | Future coverage |
| [`gpui::BorderStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L597) | enum | Future coverage |
| [`gpui::Canvas`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/canvas.rs#L23) | struct | Future coverage |
| [`gpui::ContentMask`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L2119) | struct | Future coverage |
| [`gpui::Deferred`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/deferred.rs#L16) | struct | [Published chapter](../custom-rendering/overlays.md) |
| [`gpui::DeferredScrollToItem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L103) | struct | Future coverage |
| [`gpui::Drawable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L254) | struct | Future coverage |
| [`gpui::DrawOrder`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L23) | type | Future coverage |
| [`gpui::Element`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51) | trait | [Published chapter](../getting-started/how-gpui-thinks.md) |
| [`gpui::FillOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) | struct | Future coverage |
| [`gpui::FillRule`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) | enum | Future coverage |
| [`gpui::FollowMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L113) | enum | Future coverage |
| [`gpui::Interpolate`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L408) | trait | Future coverage |
| [`gpui::ItemSize`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L126) | struct | Future coverage |
| [`gpui::List`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L37) | struct | Future coverage |
| [`gpui::ListAlignment`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L164) | enum | Future coverage |
| [`gpui::ListHorizontalSizingBehavior`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L218) | enum | Future coverage |
| [`gpui::ListMeasuringBehavior`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L198) | enum | Future coverage |
| [`gpui::ListOffset`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L1433) | struct | Future coverage |
| [`gpui::ListPrepaintState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L239) | struct | Future coverage |
| [`gpui::ListScrollEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L172) | struct | Future coverage |
| [`gpui::ListSizingBehavior`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L188) | enum | Future coverage |
| [`gpui::ListState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L54) | struct | [Published chapter](../elements/conditional-lists.md) |
| [`gpui::MonochromeSprite`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L711) | struct | Future coverage |
| [`gpui::PaddedBool32`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L31) | struct | Future coverage |
| [`gpui::PaintQuad`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7384) | struct | Future coverage |
| [`gpui::PaintSurface`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L768) | struct | Future coverage |
| [`gpui::ParsedSvg`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L103) | struct | Future coverage |
| [`gpui::Path`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L789) | struct | Future coverage |
| [`gpui::PathBuilder`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L25) | struct | [Published chapter](../custom-rendering/canvas-primitives.md) |
| [`gpui::PathId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L784) | struct | Future coverage |
| [`gpui::PathStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L17) | enum | Future coverage |
| [`gpui::PathVertex`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L934) | struct | Future coverage |
| [`gpui::PathVertex_ScaledPixels`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L20) | type | Future coverage |
| [`gpui::PolychromeSprite`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L749) | struct | Future coverage |
| [`gpui::Primitive`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L222) | enum | Future coverage |
| [`gpui::PrimitiveBatch`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L477) | enum | Future coverage |
| [`gpui::Quad`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L535) | struct | Future coverage |
| [`gpui::RenderSvgParams`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L85) | struct | Future coverage |
| [`gpui::Scene`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L41) | struct | [Published chapter](../architecture/frame-pipeline.md) |
| [`gpui::ScrollStrategy`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L84) | enum | Future coverage |
| [`gpui::Shadow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L574) | struct | Future coverage |
| [`gpui::SpringAnimation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L472) | struct | Future coverage |
| [`gpui::SpringAnimationElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L169) | struct | Future coverage |
| [`gpui::SpringConfig`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L13) | struct | Future coverage |
| [`gpui::SpringPlayback`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L456) | enum | Future coverage |
| [`gpui::SpringState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L252) | struct | Future coverage |
| [`gpui::SpringTarget`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L264) | trait | Future coverage |
| [`gpui::StrokeOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) | struct | Future coverage |
| [`gpui::SubpixelSprite`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L730) | struct | Future coverage |
| [`gpui::Surface`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/surface.rs#L25) | struct | [Published chapter](../custom-rendering/gpu-surfaces.md) |
| [`gpui::SurfaceSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/surface.rs#L11) | enum | [Published chapter](../custom-rendering/gpu-surfaces.md) |
| [`gpui::SvgRenderer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L92) | struct | Future coverage |
| [`gpui::SvgSize`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L107) | enum | Future coverage |
| [`gpui::TileId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1521) | struct | Future coverage |
| [`gpui::Transform`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L11) | type | Future coverage |
| [`gpui::TransformationMatrix`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L608) | struct | Future coverage |
| [`gpui::Underline`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L555) | struct | Future coverage |
| [`gpui::UniformList`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L58) | struct | [Published chapter](../custom-rendering/virtualization.md) |
| [`gpui::UniformListDecoration`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L580) | trait | Future coverage |
| [`gpui::UniformListFrameState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L72) | struct | Future coverage |
| [`gpui::UniformListScrollHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L80) | struct | Future coverage |
| [`gpui::UniformListScrollState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L115) | struct | Future coverage |

## Async work and queues (14)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::AsyncApp`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22) | struct | [Published chapter](../state/contexts.md) |
| [`gpui::AsyncWindowContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L283) | struct | Future coverage |
| [`gpui::BackgroundExecutor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L19) | struct | [Published chapter](../async/executors-and-tasks.md) |
| [`gpui::DedicatedExecutor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) | struct | Future coverage |
| [`gpui::FallibleTask`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) | struct | Future coverage |
| [`gpui::ForegroundExecutor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L27) | struct | [Published chapter](../async/executors-and-tasks.md) |
| [`gpui::FutureExt`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L64) | trait | Future coverage |
| [`gpui::Priority`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) | enum | Future coverage |
| [`gpui::queue`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L1) | module | Future coverage |
| [`gpui::SchedulerLocalExecutor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) | struct | Future coverage |
| [`gpui::Scope`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L458) | struct | Future coverage |
| [`gpui::Task`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) | struct | [Published chapter](../async/executors-and-tasks.md) |
| [`gpui::TaskExt`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L38) | trait | Future coverage |
| [`gpui::Timeout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L95) | struct | Future coverage |

## Windows, platforms, and accessibility (145)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::A11yCallbacks`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L788) | struct | Future coverage |
| [`gpui::A11ySubtreeBuilder`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L300) | struct | Future coverage |
| [`gpui::AccessibleAction`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L90) | enum | Future coverage |
| [`gpui::ActivityGuard`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L131) | struct | Future coverage |
| [`gpui::AnyWindowHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7139) | struct | Future coverage |
| [`gpui::AppLifecyclePhase`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L819) | enum | Future coverage |
| [`gpui::Autocapitalize`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2000) | enum | Future coverage |
| [`gpui::Capslock`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L665) | struct | Future coverage |
| [`gpui::CursorStyle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2434) | enum | Future coverage |
| [`gpui::DebugFrameOverlayMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/debug_overlay.rs#L13) | enum | Future coverage |
| [`gpui::Decorations`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L582) | enum | Future coverage |
| [`gpui::DismissEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L719) | struct | Future coverage |
| [`gpui::DisplayId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L522) | struct | Future coverage |
| [`gpui::DummyKeyboardMapper`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L27) | struct | Future coverage |
| [`gpui::FallbackPromptRenderer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L94) | struct | Future coverage |
| [`gpui::FrameDurationSnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L854) | struct | Future coverage |
| [`gpui::FrameEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L843) | enum | Future coverage |
| [`gpui::FrameTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L788) | struct | Future coverage |
| [`gpui::FrameTimingCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1200) | struct | Future coverage |
| [`gpui::GpuSpecs`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L343) | struct | Future coverage |
| [`gpui::hang`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L1) | module | Future coverage |
| [`gpui::hang::HangDetector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L29) | struct | Future coverage |
| [`gpui::hang::HangIncident`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L39) | struct | Future coverage |
| [`gpui::hang::HangTrigger`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L59) | enum | Future coverage |
| [`gpui::hang::SerializedHangContributor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L171) | enum | Future coverage |
| [`gpui::hang::SerializedHangIncident`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L113) | struct | Future coverage |
| [`gpui::InputLatencySnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L868) | struct | Future coverage |
| [`gpui::Inspector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L60) | struct | Future coverage |
| [`gpui::inspector_reflection::FunctionReflection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L233) | struct | Future coverage |
| [`gpui::InspectorElementId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L3) | struct | Future coverage |
| [`gpui::InspectorElementPath`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L30) | struct | Future coverage |
| [`gpui::InspectorRenderer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L55) | type | Future coverage |
| [`gpui::journal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L1) | module | Future coverage |
| [`gpui::journal::DrainedEntries`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L917) | struct | Future coverage |
| [`gpui::journal::ForegroundEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L65) | enum | Future coverage |
| [`gpui::journal::ForegroundJournal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L890) | struct | Future coverage |
| [`gpui::journal::ForegroundJournalCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L931) | struct | Future coverage |
| [`gpui::journal::ForegroundJournalEntry`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L227) | enum | Future coverage |
| [`gpui::journal::FrameSnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L245) | struct | Future coverage |
| [`gpui::journal::FrameStateChange`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L207) | enum | Future coverage |
| [`gpui::journal::InputTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L117) | struct | Future coverage |
| [`gpui::journal::IntervalBoundary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L176) | enum | Future coverage |
| [`gpui::journal::IntervalSealer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L978) | struct | Future coverage |
| [`gpui::journal::PollSummary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L131) | struct | Future coverage |
| [`gpui::journal::PresentedFrame`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L158) | struct | Future coverage |
| [`gpui::journal::SmallPollFlush`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L147) | struct | Future coverage |
| [`gpui::layer_shell`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L1) | module | Future coverage |
| [`gpui::layer_shell::Anchor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L24) | struct | Future coverage |
| [`gpui::layer_shell::KeyboardInteractivity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L43) | enum | Future coverage |
| [`gpui::layer_shell::Layer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L9) | enum | Future coverage |
| [`gpui::layer_shell::LayerShellNotSupportedError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L83) | struct | Future coverage |
| [`gpui::layer_shell::LayerShellOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L59) | struct | Future coverage |
| [`gpui::ManagedView`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L714) | trait | Future coverage |
| [`gpui::Orientation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) | enum | Future coverage |
| [`gpui::OwnedOsMenu`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L228) | struct | Future coverage |
| [`gpui::PathPromptOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2355) | struct | [Published chapter](../cookbook/input/file-picker.md) |
| [`gpui::Platform`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L182) | trait | Future coverage |
| [`gpui::PlatformDisplay`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L404) | trait | Future coverage |
| [`gpui::PlatformHeadlessRenderer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1090) | trait | Future coverage |
| [`gpui::PlatformKeyboardLayout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L6) | trait | Future coverage |
| [`gpui::PlatformKeyboardMapper`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L14) | trait | Future coverage |
| [`gpui::PlatformTextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1173) | trait | [Published chapter](../architecture/text-systems.md) |
| [`gpui::PlatformWindow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L876) | trait | Future coverage |
| [`gpui::popup`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L1) | module | Future coverage |
| [`gpui::popup::PopupAnchor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L57) | enum | Future coverage |
| [`gpui::popup::PopupConstraintAdjustment`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L106) | struct | Future coverage |
| [`gpui::popup::PopupGravity`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L84) | enum | Future coverage |
| [`gpui::popup::PopupNotSupportedError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L134) | struct | Future coverage |
| [`gpui::popup::PopupOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L16) | struct | Future coverage |
| [`gpui::PresentTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L820) | struct | Future coverage |
| [`gpui::profiler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1) | module | Future coverage |
| [`gpui::profiler::FrameDurationSnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L854) | struct | Future coverage |
| [`gpui::profiler::FrameEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L843) | enum | Future coverage |
| [`gpui::profiler::FrameTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L788) | struct | Future coverage |
| [`gpui::profiler::FrameTimingCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1200) | struct | Future coverage |
| [`gpui::profiler::hang`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L1) | module | Future coverage |
| [`gpui::profiler::hang::HangDetector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L29) | struct | Future coverage |
| [`gpui::profiler::hang::HangIncident`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L39) | struct | Future coverage |
| [`gpui::profiler::hang::HangTrigger`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L59) | enum | Future coverage |
| [`gpui::profiler::hang::SerializedHangContributor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L171) | enum | Future coverage |
| [`gpui::profiler::hang::SerializedHangIncident`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L113) | struct | Future coverage |
| [`gpui::profiler::InputLatencySnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L868) | struct | Future coverage |
| [`gpui::profiler::journal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L1) | module | Future coverage |
| [`gpui::profiler::journal::DrainedEntries`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L917) | struct | Future coverage |
| [`gpui::profiler::journal::ForegroundEvent`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L65) | enum | Future coverage |
| [`gpui::profiler::journal::ForegroundJournal`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L890) | struct | Future coverage |
| [`gpui::profiler::journal::ForegroundJournalCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L931) | struct | Future coverage |
| [`gpui::profiler::journal::ForegroundJournalEntry`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L227) | enum | Future coverage |
| [`gpui::profiler::journal::FrameSnapshot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L245) | struct | Future coverage |
| [`gpui::profiler::journal::FrameStateChange`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L207) | enum | Future coverage |
| [`gpui::profiler::journal::InputTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L117) | struct | Future coverage |
| [`gpui::profiler::journal::IntervalBoundary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L176) | enum | Future coverage |
| [`gpui::profiler::journal::IntervalSealer`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L978) | struct | Future coverage |
| [`gpui::profiler::journal::PollSummary`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L131) | struct | Future coverage |
| [`gpui::profiler::journal::PresentedFrame`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L158) | struct | Future coverage |
| [`gpui::profiler::journal::SmallPollFlush`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L147) | struct | Future coverage |
| [`gpui::profiler::PresentTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L820) | struct | Future coverage |
| [`gpui::profiler::SerializedLocation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L228) | struct | Future coverage |
| [`gpui::profiler::SerializedTaskTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L249) | struct | Future coverage |
| [`gpui::profiler::SerializedThreadTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L295) | struct | Future coverage |
| [`gpui::profiler::WindowProfiler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L895) | struct | Future coverage |
| [`gpui::Prompt`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L19) | trait | Future coverage |
| [`gpui::PromptButton`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2381) | enum | Future coverage |
| [`gpui::PromptHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L24) | struct | Future coverage |
| [`gpui::PromptLevel`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2368) | enum | Future coverage |
| [`gpui::PromptResponse`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L16) | struct | Future coverage |
| [`gpui::QuitMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L328) | enum | Future coverage |
| [`gpui::RenderablePromptHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L67) | struct | Future coverage |
| [`gpui::RequestFrameOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L799) | struct | Future coverage |
| [`gpui::ResizeEdge`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L551) | enum | Future coverage |
| [`gpui::Role`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) | enum | [Published chapter](../windows-platform-shipping/accessibility.md) |
| [`gpui::RunnableMeta`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L56) | struct | Future coverage |
| [`gpui::scap_screen_capture`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/scap_screen_capture.rs#L1) | module | Future coverage |
| [`gpui::ScreenCaptureFrame`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L518) | struct | Future coverage |
| [`gpui::ScreenCaptureSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L498) | trait | Future coverage |
| [`gpui::ScreenCaptureStream`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L512) | trait | Future coverage |
| [`gpui::SerializedLocation`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L228) | struct | Future coverage |
| [`gpui::SerializedTaskTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L249) | struct | Future coverage |
| [`gpui::SerializedThreadTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L295) | struct | Future coverage |
| [`gpui::SourceMetadata`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L486) | struct | Future coverage |
| [`gpui::SystemMenuType`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L70) | enum | Future coverage |
| [`gpui::SystemNotification`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L437) | struct | [Published chapter](../cookbook/windows/system-notification.md) |
| [`gpui::SystemNotificationAction`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L453) | struct | Future coverage |
| [`gpui::SystemNotificationResponse`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L463) | struct | Future coverage |
| [`gpui::SystemWindowTabController`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L377) | struct | Future coverage |
| [`gpui::TextRenderingMode`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2343) | enum | Future coverage |
| [`gpui::ThermalState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L473) | enum | Future coverage |
| [`gpui::TitlebarOptions`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2245) | struct | [Published chapter](../windows-platform-shipping/windows-and-appearance.md) |
| [`gpui::Toggled`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) | enum | Future coverage |
| [`gpui::TooltipId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L938) | struct | Future coverage |
| [`gpui::WindowAppearance`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2293) | enum | Future coverage |
| [`gpui::WindowBackgroundAppearance`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2319) | enum | Future coverage |
| [`gpui::WindowBounds`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2182) | enum | [Published chapter](../windows-platform-shipping/windows-and-appearance.md) |
| [`gpui::WindowButton`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L620) | enum | Future coverage |
| [`gpui::WindowButtonLayout`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L657) | struct | Future coverage |
| [`gpui::WindowControlArea`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L740) | enum | Future coverage |
| [`gpui::WindowControls`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L595) | struct | Future coverage |
| [`gpui::WindowDecorations`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L572) | enum | Future coverage |
| [`gpui::WindowHandle`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L6996) | struct | Future coverage |
| [`gpui::WindowId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L6975) | struct | Future coverage |
| [`gpui::WindowInsets`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L838) | struct | Future coverage |
| [`gpui::WindowKind`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2259) | enum | Future coverage |
| [`gpui::WindowParams`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2125) | struct | Future coverage |
| [`gpui::WindowProfiler`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L895) | struct | Future coverage |
| [`gpui::WindowVisibility`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L102) | enum | Future coverage |

## Testing and profiling (18)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::bench`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::bench_group`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L120) | macro | Future coverage |
| [`gpui::bench_main`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L131) | macro | Future coverage |
| [`gpui::BenchAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L484) | struct | Future coverage |
| [`gpui::BenchReport`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L100) | struct | Future coverage |
| [`gpui::BenchWindowContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L1151) | struct | Future coverage |
| [`gpui::HeadlessAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38) | struct | [Published chapter](../testing/harness-screenshots.md) |
| [`gpui::property_test`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::test`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) | macro | Future coverage |
| [`gpui::test`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L1) | module | Future coverage |
| [`gpui::TestApp`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_app.rs#L40) | struct | Future coverage |
| [`gpui::TestAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L21) | struct | [Published chapter](../testing/test-contexts.md) |
| [`gpui::TestAppWindow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_app.rs#L320) | struct | Future coverage |
| [`gpui::TestScreenCaptureSource`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/platform.rs#L60) | struct | Future coverage |
| [`gpui::TestScreenCaptureStream`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/platform.rs#L63) | struct | Future coverage |
| [`gpui::VisualTestAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/visual_test_context.rs#L21) | struct | Future coverage |
| [`gpui::VisualTestContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L783) | struct | [Published chapter](../testing/test-contexts.md) |
| [`gpui::VisualTestPlatform`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/visual_test.rs#L31) | struct | Future coverage |

## Types, preludes, and interop (17)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::ArcCow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L137) | enum | Future coverage |
| [`gpui::Cascade`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L80) | struct | Future coverage |
| [`gpui::CascadeSlot`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L93) | struct | Future coverage |
| [`gpui::IsEmpty`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L66) | trait | Future coverage |
| [`gpui::prelude`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L1) | module | Future coverage |
| [`gpui::prelude::BorrowAppContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L300) | trait | Future coverage |
| [`gpui::prelude::FluentBuilder`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L11) | trait | Future coverage |
| [`gpui::prelude::IntoElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L6) | macro | Future coverage |
| [`gpui::prelude::Refineable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) | macro | Future coverage |
| [`gpui::prelude::Refineable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) | trait | Future coverage |
| [`gpui::prelude::VisualContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L8) | macro | Future coverage |
| [`gpui::prelude::VisualContext`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L260) | trait | Future coverage |
| [`gpui::Refineable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L1) | macro | Future coverage |
| [`gpui::Refineable`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L29) | trait | Future coverage |
| [`gpui::Result`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L93) | type | Future coverage |
| [`gpui::SharedString`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_shared_string/gpui_shared_string.rs#L15) | struct | Future coverage |
| [`gpui::SharedUri`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/shared_uri.rs#L7) | struct | Future coverage |

## Internals (11)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::ArenaClearNeeded`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L457) | struct | Future coverage |
| [`gpui::AtlasKey`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1376) | enum | Future coverage |
| [`gpui::AtlasTextureId`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1494) | struct | Future coverage |
| [`gpui::AtlasTextureKind`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1512) | enum | Future coverage |
| [`gpui::AtlasTile`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1480) | struct | Future coverage |
| [`gpui::inspector_reflection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L227) | module | Future coverage |
| [`gpui::NoopTextSystem`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1209) | struct | [Published chapter](../architecture/text-systems.md) |
| [`gpui::PlatformAtlas`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1427) | trait | Future coverage |
| [`gpui::styled_reflection`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L20) | module | Future coverage |
| [`gpui::ThreadedDispatcher`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/threaded_dispatcher.rs#L28) | struct | Future coverage |
| [`gpui::Tiling`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L759) | struct | Future coverage |

## Excluded: documentation-only conceptual module (2)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::_accessibility`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/_accessibility.rs#L1) | module | Future coverage |
| [`gpui::_ownership_and_data_flow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/_ownership_and_data_flow.rs#L1) | module | Future coverage |

## Excluded: foreign crate module reexport (3)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::accesskit`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L89) | module | Future coverage |
| [`gpui::http_client`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L138) | module | Future coverage |
| [`gpui::proptest`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L66) | module | Future coverage |

## Excluded: foreign macro reexport (1)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::ctor`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L99) | macro | Future coverage |

## Excluded: hidden implementation support (54)

| Public item | Kind | Teaching route |
| --- | --- | --- |
| [`gpui::ActionStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L11) | struct | Future coverage |
| [`gpui::ActionTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L150) | struct | Future coverage |
| [`gpui::ActiveTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L98) | struct | Future coverage |
| [`gpui::AppCell`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L80) | struct | Future coverage |
| [`gpui::AppRef`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L118) | struct | Future coverage |
| [`gpui::AppRefMut`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L131) | struct | Future coverage |
| [`gpui::AtlasTextureList`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1442) | struct | Future coverage |
| [`gpui::GlobalThreadTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L417) | struct | Future coverage |
| [`gpui::GuardedTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L414) | type | Future coverage |
| [`gpui::InteractiveTextState`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L999) | struct | Future coverage |
| [`gpui::MacroActionBuilder`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L268) | struct | Future coverage |
| [`gpui::MacroActionData`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L273) | struct | Future coverage |
| [`gpui::PlatformDispatcher`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1126) | trait | Future coverage |
| [`gpui::PriorityQueueReceiver`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L165) | struct | Future coverage |
| [`gpui::PriorityQueueSender`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L136) | struct | Future coverage |
| [`gpui::private`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L75) | module | Future coverage |
| [`gpui::private::anyhow`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L76) | module | Future coverage |
| [`gpui::private::inventory`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L77) | module | Future coverage |
| [`gpui::private::schemars`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L78) | module | Future coverage |
| [`gpui::private::serde`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L79) | module | Future coverage |
| [`gpui::private::serde_json`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L80) | module | Future coverage |
| [`gpui::profiler::ActionStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L11) | struct | Future coverage |
| [`gpui::profiler::ActionTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L150) | struct | Future coverage |
| [`gpui::profiler::ActiveTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L98) | struct | Future coverage |
| [`gpui::profiler::GlobalThreadTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L417) | struct | Future coverage |
| [`gpui::profiler::GuardedTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L414) | type | Future coverage |
| [`gpui::profiler::ProfilingCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L339) | struct | Future coverage |
| [`gpui::profiler::TaskStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L424) | struct | Future coverage |
| [`gpui::profiler::TaskTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L78) | struct | Future coverage |
| [`gpui::profiler::ThreadTaskStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L180) | struct | Future coverage |
| [`gpui::profiler::ThreadTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L129) | struct | Future coverage |
| [`gpui::profiler::ThreadTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L556) | struct | Future coverage |
| [`gpui::profiler::ThreadTimingsDelta`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L327) | struct | Future coverage |
| [`gpui::profiler::YieldTime`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L74) | struct | Future coverage |
| [`gpui::ProfilingCollector`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L339) | struct | Future coverage |
| [`gpui::queue::Iter`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L369) | struct | Future coverage |
| [`gpui::queue::PriorityQueueReceiver`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L165) | struct | Future coverage |
| [`gpui::queue::PriorityQueueSender`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L136) | struct | Future coverage |
| [`gpui::queue::RecvError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L195) | struct | Future coverage |
| [`gpui::queue::SendError`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L185) | struct | Future coverage |
| [`gpui::queue::TryIter`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L380) | struct | Future coverage |
| [`gpui::RunnableVariant`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1112) | type | Future coverage |
| [`gpui::SystemWindowTab`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L356) | struct | Future coverage |
| [`gpui::TasksIncluded`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1118) | enum | Future coverage |
| [`gpui::TaskStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L424) | struct | Future coverage |
| [`gpui::TaskTiming`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L78) | struct | Future coverage |
| [`gpui::TestDispatcher`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/dispatcher.rs#L17) | struct | Future coverage |
| [`gpui::ThreadTaskStatistics`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L180) | struct | Future coverage |
| [`gpui::ThreadTaskTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L129) | struct | Future coverage |
| [`gpui::ThreadTimings`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L556) | struct | Future coverage |
| [`gpui::ThreadTimingsDelta`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L327) | struct | Future coverage |
| [`gpui::TimerResolutionGuard`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1115) | type | Future coverage |
| [`gpui::ViewElement`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L240) | struct | Future coverage |
| [`gpui::YieldTime`](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L74) | struct | Future coverage |

