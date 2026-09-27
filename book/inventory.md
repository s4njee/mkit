# GPUI public API inventory

Pinned package: `gpui-pre` 0.3.5 at Zed revision [`d89e9c2124b2786a390c7a451c7488601b4da2e1`](https://github.com/zed-industries/zed/tree/d89e9c2124b2786a390c7a451c7488601b4da2e1).

This snapshot records **634** public type, trait, macro, and module names from rustdoc JSON extracts for `aarch64-apple-darwin, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc, wasm32-unknown-unknown`. It includes a Linux all-features extract (632 names) and a WebAssembly test-support extract (566 names) produced in disposable copies with compile-only dependency shims. Those shims permit rustdoc metadata generation only; they do not prove that either runtime profile executes. Rustdoc also enables `cfg(doc)`. Chapter destinations are planned coverage, not a claim that chapters already exist.

Observed target labels: **macOS** = `aarch64-apple-darwin`; **Linux** = `aarch64-unknown-linux-gnu`; **Windows** = `x86_64-pc-windows-msvc`; **WebAssembly** = `wasm32-unknown-unknown`. **macOS (all features)** means the name appeared only with every `gpui-pre` feature enabled. An absent label means that name was not observed in that extract; it does not establish unavailability on the platform.

The inventory excludes functions, constants, associated items, fields, and variants. Foreign crate modules are listed at the reexport boundary; their dependencies' own APIs are outside this inventory. The Windows extract uses `font-kit,test-support,wayland,x11` with default `windows-manifest` disabled because the `llvm-rc` tool is unavailable. Linux all-features uses a temporary x11 build-script pkg-config bypass and an equivalent wait-timeout cast edit for current-nightly lint compatibility; rustdoc does not link or run the backend. WebAssembly test-support uses a temporary panic stub for the missing `wait-timeout` implementation and a wasm `ExitStatus` shim in `rusty-fork`; neither shim is executed. The Linux broad optional-feature audit remains a separate profile and omits `x11`, `screen-capture`, `test-support`, and `windows-manifest`.

This is a cross-profile name union, not proof of exhaustive API coverage. FreeBSD and other target triples are not extracted. Windows `windows-manifest` is unobserved, and source cfg branches that are enabled in all-features profiles or share a name with an observed declaration can still expose distinct paths. The file-level cfg scan is only a review aid; it cannot establish all target/feature reachability. The first E2.1 coverage checklist item therefore remains open pending a complete source-reachability audit or additional target/profile evidence.

Refresh the macOS extract and union with `python3 scripts/generate_gpui_inventory.py --regenerate`; refresh standard target extracts with `--update-linux`, `--update-windows`, `--update-wasm`, `--update-all-features`, and `--update-linux-feature-audit`. Reproduce the two isolated metadata profiles with `--update-isolated-audits`. Generate this page with `python3 scripts/generate_gpui_inventory.py --markdown > book/inventory.md`; verify snapshots, union JSON, and page with `python3 scripts/generate_gpui_inventory.py --check`. The check also fails when the workspace GPUI pin changes.

## E2.4: Getting started (5)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::App` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L686) |
| `gpui::Application` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L144) |
| `gpui::ApplicationHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L151) |
| `gpui::Window` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1144) |
| `gpui::WindowOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2044) |

## E2.5: App, context, and entities (34)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::AnyEntity` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L246) |
| `gpui::AnyTooltip` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3038) |
| `gpui::AnyView` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L19) |
| `gpui::AnyWeakEntity` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L569) |
| `gpui::AnyWeakView` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L112) |
| `gpui::AppContext` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::AppContext` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L172) |
| `gpui::BorrowAppContext` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L300) |
| `gpui::Context` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21) |
| `gpui::CursorHideMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L343) |
| `gpui::EmptyView` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L476) |
| `gpui::Entity` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L414) |
| `gpui::EntityId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L27) |
| `gpui::EventEmitter` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L296) |
| `gpui::Global` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L22) |
| `gpui::GpuiBorrow` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3090) |
| `gpui::LeakDetectorSnapshot` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L944) |
| `gpui::Observation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L193) |
| `gpui::prelude::Context` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/context.rs#L21) |
| `gpui::prelude::Render` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) |
| `gpui::prelude::Render` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163) |
| `gpui::prelude::RenderOnce` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L179) |
| `gpui::ReadGlobal` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L30) |
| `gpui::Render` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::Render` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L163) |
| `gpui::RenderOnce` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L179) |
| `gpui::Reservation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L249) |
| `gpui::Subscription` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/subscription.rs#L150) |
| `gpui::test::Observation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L193) |
| `gpui::UpdateGlobal` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/global.rs#L44) |
| `gpui::View` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L182) |
| `gpui::VisualContext` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::VisualContext` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L260) |
| `gpui::WeakEntity` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/entity_map.rs#L740) |

## E2.6: Elements, styling, and text (157)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::AbsoluteLength` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3298) |
| `gpui::AlignContent` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1089) |
| `gpui::AlignItems` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1039) |
| `gpui::AlignSelf` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1073) |
| `gpui::Along` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L43) |
| `gpui::Anchor` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2165) |
| `gpui::AnyElement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L588) |
| `gpui::AnyImageCache` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L24) |
| `gpui::Asset` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L99) |
| `gpui::AssetLogger` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L114) |
| `gpui::AssetSource` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L13) |
| `gpui::AvailableSpace` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/taffy.rs#L678) |
| `gpui::Axis` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L25) |
| `gpui::Background` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L779) |
| `gpui::border_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) |
| `gpui::Boundary` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L674) |
| `gpui::Bounds` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L723) |
| `gpui::BoundsRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L720) |
| `gpui::box_shadow_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) |
| `gpui::BoxShadow` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L349) |
| `gpui::colors` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L1) |
| `gpui::colors::Colors` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L9) |
| `gpui::colors::DefaultAppearance` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L107) |
| `gpui::colors::DefaultColors` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L92) |
| `gpui::colors::GlobalColors` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/colors.rs#L79) |
| `gpui::ColorSpace` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L759) |
| `gpui::ContainerQuery` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/container_query.rs#L52) |
| `gpui::Corners` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2258) |
| `gpui::CornersRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2255) |
| `gpui::cursor_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) |
| `gpui::DebugBelow` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L23) |
| `gpui::DecorationRun` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L24) |
| `gpui::DefiniteLength` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3460) |
| `gpui::DevicePixels` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2982) |
| `gpui::Display` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1131) |
| `gpui::Div` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1800) |
| `gpui::DivFrameState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1848) |
| `gpui::DivInspectorState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1854) |
| `gpui::Edges` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L1750) |
| `gpui::EdgesRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L1747) |
| `gpui::Empty` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L728) |
| `gpui::Fill` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L853) |
| `gpui::FlexDirection` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1173) |
| `gpui::FlexWrap` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1150) |
| `gpui::Font` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1063) |
| `gpui::FontFallbacks` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/font_fallbacks.rs#L9) |
| `gpui::FontFamilyId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L42) |
| `gpui::FontFeatures` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/font_features.rs#L8) |
| `gpui::FontId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L38) |
| `gpui::FontMetrics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1116) |
| `gpui::FontRun` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L877) |
| `gpui::FontStyle` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L981) |
| `gpui::FontWeight` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L899) |
| `gpui::GlobalElementId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L213) |
| `gpui::GlyphId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1026) |
| `gpui::GlyphRasterData` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L15) |
| `gpui::GridLocation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3793) |
| `gpui::GridPlacement` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3802) |
| `gpui::GridTemplate` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L170) |
| `gpui::GridTemplateMinSize` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L145) |
| `gpui::GridTemplateRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L159) |
| `gpui::GroupStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L57) |
| `gpui::Half` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3827) |
| `gpui::HighlightStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L580) |
| `gpui::Hsla` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L334) |
| `gpui::Image` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2774) |
| `gpui::ImageAssetLoader` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L617) |
| `gpui::ImageCache` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L197) |
| `gpui::ImageCacheElement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L71) |
| `gpui::ImageCacheError` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L750) |
| `gpui::ImageCacheItem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L167) |
| `gpui::ImageCacheProvider` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L210) |
| `gpui::ImageFormat` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2697) |
| `gpui::ImageFormatIter` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2696) |
| `gpui::ImageId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L33) |
| `gpui::ImageSource` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L42) |
| `gpui::ImageStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L129) |
| `gpui::Img` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L192) |
| `gpui::ImgLayoutState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L260) |
| `gpui::ImgResourceLoader` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L38) |
| `gpui::InteractiveText` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L981) |
| `gpui::IntoElement` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::IntoElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L145) |
| `gpui::IsZero` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3878) |
| `gpui::JustifyContent` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1124) |
| `gpui::JustifyItems` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1066) |
| `gpui::JustifySelf` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1080) |
| `gpui::LayoutId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/taffy.rs#L385) |
| `gpui::Length` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3611) |
| `gpui::LinearColorStop` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L883) |
| `gpui::LineFragment` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L614) |
| `gpui::LineLayout` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L16) |
| `gpui::LineWrapper` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L17) |
| `gpui::LineWrapperHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L862) |
| `gpui::margin_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L9) |
| `gpui::ObjectFit` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L29) |
| `gpui::Overflow` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1208) |
| `gpui::overflow_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) |
| `gpui::padding_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) |
| `gpui::ParentElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L188) |
| `gpui::Percentage` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2621) |
| `gpui::Pixels` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2677) |
| `gpui::Point` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L85) |
| `gpui::PointRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L67) |
| `gpui::Position` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L1236) |
| `gpui::position_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L10) |
| `gpui::prelude::Element` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51) |
| `gpui::prelude::IntoElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L145) |
| `gpui::prelude::ParentElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L188) |
| `gpui::prelude::Styled` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L22) |
| `gpui::prelude::StyledImage` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L148) |
| `gpui::Radians` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L2596) |
| `gpui::Rems` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3238) |
| `gpui::RenderGlyphParams` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L1035) |
| `gpui::RenderImage` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L43) |
| `gpui::RenderImageParams` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/assets.rs#L37) |
| `gpui::Resource` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/asset_cache.rs#L71) |
| `gpui::RetainAllImageCache` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L222) |
| `gpui::RetainAllImageCacheProvider` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/image_cache.rs#L312) |
| `gpui::Rgba` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/color.rs#L39) |
| `gpui::ScaledPixels` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L3075) |
| `gpui::ScrollAnchor` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4191) |
| `gpui::ScrollHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4245) |
| `gpui::ShapedGlyph` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L42) |
| `gpui::ShapedLine` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L43) |
| `gpui::ShapedRun` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L33) |
| `gpui::Size` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L396) |
| `gpui::SizeRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/geometry.rs#L392) |
| `gpui::Stateful` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L4063) |
| `gpui::StrikethroughStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L843) |
| `gpui::StrikethroughStyleRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L841) |
| `gpui::Style` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L180) |
| `gpui::Styled` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L22) |
| `gpui::StyledImage` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/img.rs#L148) |
| `gpui::StyledText` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L391) |
| `gpui::StyleRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L178) |
| `gpui::Svg` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/svg.rs#L16) |
| `gpui::text` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L160) |
| `gpui::Text` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L67) |
| `gpui::TextAlign` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L423) |
| `gpui::TextLayout` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L614) |
| `gpui::TextOverflow` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L407) |
| `gpui::TextRun` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L999) |
| `gpui::TextStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L438) |
| `gpui::TextStyleRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L436) |
| `gpui::TextSystem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L51) |
| `gpui::Transformation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/svg.rs#L214) |
| `gpui::TruncateFrom` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_wrapper.rs#L7) |
| `gpui::UnderlineStyle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L828) |
| `gpui::UnderlineStyleRefinement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L826) |
| `gpui::Visibility` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L339) |
| `gpui::visibility_style_methods` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L11) |
| `gpui::WhiteSpace` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/style.rs#L397) |
| `gpui::WindowTextSystem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system.rs#L376) |
| `gpui::WrapBoundary` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L287) |
| `gpui::WrappedLine` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line.rs#L267) |
| `gpui::WrappedLineLayout` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/text_system/line_layout.rs#L274) |

## E2.7: Input, actions, and key bindings (93)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::Action` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L4) |
| `gpui::Action` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L118) |
| `gpui::ActionBuildError` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L194) |
| `gpui::actions` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L25) |
| `gpui::AnyDrag` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3011) |
| `gpui::AsKeystroke` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L11) |
| `gpui::BindingIndex` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L27) |
| `gpui::ClickEvent` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L290) |
| `gpui::ClipboardEntry` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2563) |
| `gpui::ClipboardItem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2522) |
| `gpui::ClipboardReadError` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2532) |
| `gpui::ClipboardString` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2928) |
| `gpui::ContextEntry` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L14) |
| `gpui::DispatchEventResult` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L2109) |
| `gpui::DispatchPhase` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L91) |
| `gpui::DragMoveEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L67) |
| `gpui::ElementClickedState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3627) |
| `gpui::ElementHoverState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3643) |
| `gpui::ElementId` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7218) |
| `gpui::ExternalDragPayload` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L706) |
| `gpui::ExternalDragPayloadSource` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3032) |
| `gpui::ExternalPaths` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L694) |
| `gpui::FileDragPaths` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L714) |
| `gpui::FileDropEvent` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L737) |
| `gpui::Focusable` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L701) |
| `gpui::FocusHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L527) |
| `gpui::FocusId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L325) |
| `gpui::FocusOutEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L320) |
| `gpui::GestureEvent` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L21) |
| `gpui::GestureKinds` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L353) |
| `gpui::GestureTuning` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L114) |
| `gpui::Hitbox` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L822) |
| `gpui::HitboxBehavior` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L879) |
| `gpui::HitboxId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L753) |
| `gpui::HoverListenerMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1737) |
| `gpui::InputEvent` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L9) |
| `gpui::InteractiveElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L774) |
| `gpui::InteractiveElementState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L3605) |
| `gpui::Interactivity` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L2128) |
| `gpui::InvalidKeystrokeError` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L51) |
| `gpui::KeyBinding` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L10) |
| `gpui::KeyBindingContextPredicate` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L172) |
| `gpui::KeybindingKeystroke` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L37) |
| `gpui::KeyBindingMetaIndex` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/binding.rs#L143) |
| `gpui::KeyboardButton` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L443) |
| `gpui::KeyboardClickEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L264) |
| `gpui::KeyContext` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap/context.rs#L10) |
| `gpui::KeyDownEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L25) |
| `gpui::KeyEvent` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L15) |
| `gpui::Keymap` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L18) |
| `gpui::KeymapVersion` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/keymap.rs#L14) |
| `gpui::Keystroke` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L18) |
| `gpui::KeystrokeEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L3053) |
| `gpui::KeyUpEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L47) |
| `gpui::LongPressEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L406) |
| `gpui::Menu` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L4) |
| `gpui::MenuItem` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L76) |
| `gpui::Modifiers` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L448) |
| `gpui::ModifiersChangedEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L62) |
| `gpui::MouseButton` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L453) |
| `gpui::MouseClickEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L220) |
| `gpui::MouseDownEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L148) |
| `gpui::MouseEvent` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L18) |
| `gpui::MouseExitEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L666) |
| `gpui::MouseMoveEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L494) |
| `gpui::MousePressureEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L243) |
| `gpui::MouseUpEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L185) |
| `gpui::NavigationDirection` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L483) |
| `gpui::NoAction` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L431) |
| `gpui::NullPlatformGestures` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L452) |
| `gpui::OngoingScroll` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L49) |
| `gpui::OsAction` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L311) |
| `gpui::OsMenu` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L50) |
| `gpui::OwnedMenu` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L238) |
| `gpui::OwnedMenuItem` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L250) |
| `gpui::PinchEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L571) |
| `gpui::PlatformGestures` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L438) |
| `gpui::PlatformInput` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L771) |
| `gpui::prelude::InteractiveElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L774) |
| `gpui::prelude::StatefulInteractiveElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1303) |
| `gpui::PressureStage` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L230) |
| `gpui::register_action` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::ScrollDelta` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L554) |
| `gpui::ScrollPhysics` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L151) |
| `gpui::ScrollWheelEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L522) |
| `gpui::StatefulInteractiveElement` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/div.rs#L1303) |
| `gpui::TouchClickEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L275) |
| `gpui::TouchDragEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gestures.rs#L386) |
| `gpui::TouchEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L119) |
| `gpui::TouchId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L109) |
| `gpui::TouchPhase` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/interactive.rs#L88) |
| `gpui::Unbind` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L448) |
| `gpui::WeakFocusHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L666) |

## E2.8: Text input and IME (10)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::ElementInputHandler` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L128) |
| `gpui::EntityInputHandler` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/input.rs#L13) |
| `gpui::InputHandler` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1793) |
| `gpui::PendingInputStatus` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1297) |
| `gpui::PendingInputTimeoutStatus` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L1316) |
| `gpui::PlatformInputHandler` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1536) |
| `gpui::TextInputAction` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2022) |
| `gpui::TextInputConfiguration` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1987) |
| `gpui::TextInputStateChange` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L864) |
| `gpui::UTF16Selection` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1780) |

## E2.9: Composition and advanced elements (70)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::Anchored` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L16) |
| `gpui::AnchoredFitMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L243) |
| `gpui::AnchoredPositionMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L254) |
| `gpui::AnchoredState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/anchored.rs#L10) |
| `gpui::Animation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L15) |
| `gpui::AnimationElement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L161) |
| `gpui::AnimationExt` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L82) |
| `gpui::AnimationPhase` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L328) |
| `gpui::BorderStyle` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L597) |
| `gpui::Canvas` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/canvas.rs#L23) |
| `gpui::ContentMask` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L2119) |
| `gpui::Deferred` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/deferred.rs#L16) |
| `gpui::DeferredScrollToItem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L103) |
| `gpui::Drawable` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L254) |
| `gpui::DrawOrder` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L23) |
| `gpui::Element` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/element.rs#L51) |
| `gpui::FillOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) |
| `gpui::FillRule` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) |
| `gpui::FollowMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L113) |
| `gpui::Interpolate` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L408) |
| `gpui::ItemSize` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L126) |
| `gpui::List` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L37) |
| `gpui::ListAlignment` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L164) |
| `gpui::ListHorizontalSizingBehavior` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L218) |
| `gpui::ListMeasuringBehavior` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L198) |
| `gpui::ListOffset` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L1433) |
| `gpui::ListPrepaintState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L239) |
| `gpui::ListScrollEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L172) |
| `gpui::ListSizingBehavior` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L188) |
| `gpui::ListState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/list.rs#L54) |
| `gpui::MonochromeSprite` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L711) |
| `gpui::PaddedBool32` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L31) |
| `gpui::PaintQuad` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7384) |
| `gpui::PaintSurface` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L768) |
| `gpui::ParsedSvg` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L103) |
| `gpui::Path` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L789) |
| `gpui::PathBuilder` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L25) |
| `gpui::PathId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L784) |
| `gpui::PathStyle` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L17) |
| `gpui::PathVertex` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L934) |
| `gpui::PathVertex_ScaledPixels` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L20) |
| `gpui::PolychromeSprite` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L749) |
| `gpui::Primitive` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L222) |
| `gpui::PrimitiveBatch` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L477) |
| `gpui::Quad` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L535) |
| `gpui::RenderSvgParams` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L85) |
| `gpui::Scene` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L41) |
| `gpui::ScrollStrategy` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L84) |
| `gpui::Shadow` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L574) |
| `gpui::SpringAnimation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L472) |
| `gpui::SpringAnimationElement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/animation.rs#L169) |
| `gpui::SpringConfig` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L13) |
| `gpui::SpringPlayback` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L456) |
| `gpui::SpringState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L252) |
| `gpui::SpringTarget` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/spring.rs#L264) |
| `gpui::StrokeOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L12) |
| `gpui::SubpixelSprite` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L730) |
| `gpui::Surface` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/surface.rs#L25) |
| `gpui::SurfaceSource` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/surface.rs#L11) |
| `gpui::SvgRenderer` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L92) |
| `gpui::SvgSize` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/svg_renderer.rs#L107) |
| `gpui::TileId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1521) |
| `gpui::Transform` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/path_builder.rs#L11) |
| `gpui::TransformationMatrix` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L608) |
| `gpui::Underline` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/scene.rs#L555) |
| `gpui::UniformList` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L58) |
| `gpui::UniformListDecoration` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L580) |
| `gpui::UniformListFrameState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L72) |
| `gpui::UniformListScrollHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L80) |
| `gpui::UniformListScrollState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/uniform_list.rs#L115) |

## E2.10: Async work and queues (14)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::AsyncApp` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L22) |
| `gpui::AsyncWindowContext` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/async_context.rs#L283) |
| `gpui::BackgroundExecutor` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L19) |
| `gpui::DedicatedExecutor` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) |
| `gpui::FallibleTask` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) |
| `gpui::ForegroundExecutor` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L27) |
| `gpui::FutureExt` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L64) |
| `gpui::Priority` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) |
| `gpui::queue` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L1) |
| `gpui::SchedulerLocalExecutor` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) |
| `gpui::Scope` | struct | macOS, Linux, Windows | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L458) |
| `gpui::Task` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L13) |
| `gpui::TaskExt` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/executor.rs#L38) |
| `gpui::Timeout` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L95) |

## E2.11: Windows, platforms, and accessibility (145)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::A11yCallbacks` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L788) |
| `gpui::A11ySubtreeBuilder` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L300) |
| `gpui::AccessibleAction` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L90) |
| `gpui::ActivityGuard` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L131) |
| `gpui::AnyWindowHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L7139) |
| `gpui::AppLifecyclePhase` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L819) |
| `gpui::Autocapitalize` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2000) |
| `gpui::Capslock` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keystroke.rs#L665) |
| `gpui::CursorStyle` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2434) |
| `gpui::DebugFrameOverlayMode` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/debug_overlay.rs#L13) |
| `gpui::Decorations` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L582) |
| `gpui::DismissEvent` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L719) |
| `gpui::DisplayId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L522) |
| `gpui::DummyKeyboardMapper` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L27) |
| `gpui::FallbackPromptRenderer` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L94) |
| `gpui::FrameDurationSnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L854) |
| `gpui::FrameEvent` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L843) |
| `gpui::FrameTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L788) |
| `gpui::FrameTimingCollector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1200) |
| `gpui::GpuSpecs` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L343) |
| `gpui::hang` | module | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L1) |
| `gpui::hang::HangDetector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L29) |
| `gpui::hang::HangIncident` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L39) |
| `gpui::hang::HangTrigger` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L59) |
| `gpui::hang::SerializedHangContributor` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L171) |
| `gpui::hang::SerializedHangIncident` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L113) |
| `gpui::InputLatencySnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L868) |
| `gpui::Inspector` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L60) |
| `gpui::inspector_reflection::FunctionReflection` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L233) |
| `gpui::InspectorElementId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L3) |
| `gpui::InspectorElementPath` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L30) |
| `gpui::InspectorRenderer` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L55) |
| `gpui::journal` | module | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L1) |
| `gpui::journal::DrainedEntries` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L917) |
| `gpui::journal::ForegroundEvent` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L65) |
| `gpui::journal::ForegroundJournal` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L890) |
| `gpui::journal::ForegroundJournalCollector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L931) |
| `gpui::journal::ForegroundJournalEntry` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L227) |
| `gpui::journal::FrameSnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L245) |
| `gpui::journal::FrameStateChange` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L207) |
| `gpui::journal::InputTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L117) |
| `gpui::journal::IntervalBoundary` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L176) |
| `gpui::journal::IntervalSealer` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L978) |
| `gpui::journal::PollSummary` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L131) |
| `gpui::journal::PresentedFrame` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L158) |
| `gpui::journal::SmallPollFlush` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L147) |
| `gpui::layer_shell` | module | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L1) |
| `gpui::layer_shell::Anchor` | struct | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L24) |
| `gpui::layer_shell::KeyboardInteractivity` | enum | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L43) |
| `gpui::layer_shell::Layer` | enum | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L9) |
| `gpui::layer_shell::LayerShellNotSupportedError` | struct | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L83) |
| `gpui::layer_shell::LayerShellOptions` | struct | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/layer_shell.rs#L59) |
| `gpui::ManagedView` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L714) |
| `gpui::Orientation` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) |
| `gpui::OwnedOsMenu` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L228) |
| `gpui::PathPromptOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2355) |
| `gpui::Platform` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L182) |
| `gpui::PlatformDisplay` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L404) |
| `gpui::PlatformHeadlessRenderer` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1090) |
| `gpui::PlatformKeyboardLayout` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L6) |
| `gpui::PlatformKeyboardMapper` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/keyboard.rs#L14) |
| `gpui::PlatformTextSystem` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1173) |
| `gpui::PlatformWindow` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L876) |
| `gpui::popup` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L1) |
| `gpui::popup::PopupAnchor` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L57) |
| `gpui::popup::PopupConstraintAdjustment` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L106) |
| `gpui::popup::PopupGravity` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L84) |
| `gpui::popup::PopupNotSupportedError` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L134) |
| `gpui::popup::PopupOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/popup.rs#L16) |
| `gpui::PresentTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L820) |
| `gpui::profiler` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1) |
| `gpui::profiler::FrameDurationSnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L854) |
| `gpui::profiler::FrameEvent` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L843) |
| `gpui::profiler::FrameTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L788) |
| `gpui::profiler::FrameTimingCollector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L1200) |
| `gpui::profiler::hang` | module | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L1) |
| `gpui::profiler::hang::HangDetector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L29) |
| `gpui::profiler::hang::HangIncident` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L39) |
| `gpui::profiler::hang::HangTrigger` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L59) |
| `gpui::profiler::hang::SerializedHangContributor` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L171) |
| `gpui::profiler::hang::SerializedHangIncident` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/hang.rs#L113) |
| `gpui::profiler::InputLatencySnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L868) |
| `gpui::profiler::journal` | module | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L1) |
| `gpui::profiler::journal::DrainedEntries` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L917) |
| `gpui::profiler::journal::ForegroundEvent` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L65) |
| `gpui::profiler::journal::ForegroundJournal` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L890) |
| `gpui::profiler::journal::ForegroundJournalCollector` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L931) |
| `gpui::profiler::journal::ForegroundJournalEntry` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L227) |
| `gpui::profiler::journal::FrameSnapshot` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L245) |
| `gpui::profiler::journal::FrameStateChange` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L207) |
| `gpui::profiler::journal::InputTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L117) |
| `gpui::profiler::journal::IntervalBoundary` | enum | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L176) |
| `gpui::profiler::journal::IntervalSealer` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L978) |
| `gpui::profiler::journal::PollSummary` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L131) |
| `gpui::profiler::journal::PresentedFrame` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L158) |
| `gpui::profiler::journal::SmallPollFlush` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/journal.rs#L147) |
| `gpui::profiler::PresentTiming` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L820) |
| `gpui::profiler::SerializedLocation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L228) |
| `gpui::profiler::SerializedTaskTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L249) |
| `gpui::profiler::SerializedThreadTaskTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L295) |
| `gpui::profiler::WindowProfiler` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L895) |
| `gpui::Prompt` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L19) |
| `gpui::PromptButton` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2381) |
| `gpui::PromptHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L24) |
| `gpui::PromptLevel` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2368) |
| `gpui::PromptResponse` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L16) |
| `gpui::QuitMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L328) |
| `gpui::RenderablePromptHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/prompts.rs#L67) |
| `gpui::RequestFrameOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L799) |
| `gpui::ResizeEdge` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L551) |
| `gpui::Role` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) |
| `gpui::RunnableMeta` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L56) |
| `gpui::scap_screen_capture` | module | Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/scap_screen_capture.rs#L1) |
| `gpui::ScreenCaptureFrame` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L518) |
| `gpui::ScreenCaptureSource` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L498) |
| `gpui::ScreenCaptureStream` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L512) |
| `gpui::SerializedLocation` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L228) |
| `gpui::SerializedTaskTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L249) |
| `gpui::SerializedThreadTaskTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L295) |
| `gpui::SourceMetadata` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L486) |
| `gpui::SystemMenuType` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/app_menu.rs#L70) |
| `gpui::SystemNotification` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L437) |
| `gpui::SystemNotificationAction` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L453) |
| `gpui::SystemNotificationResponse` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L463) |
| `gpui::SystemWindowTabController` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L377) |
| `gpui::TextRenderingMode` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2343) |
| `gpui::ThermalState` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L473) |
| `gpui::TitlebarOptions` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2245) |
| `gpui::Toggled` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L91) |
| `gpui::TooltipId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L938) |
| `gpui::WindowAppearance` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2293) |
| `gpui::WindowBackgroundAppearance` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2319) |
| `gpui::WindowBounds` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2182) |
| `gpui::WindowButton` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L620) |
| `gpui::WindowButtonLayout` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L657) |
| `gpui::WindowControlArea` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L740) |
| `gpui::WindowControls` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L595) |
| `gpui::WindowDecorations` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L572) |
| `gpui::WindowHandle` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L6996) |
| `gpui::WindowId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L6975) |
| `gpui::WindowInsets` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L838) |
| `gpui::WindowKind` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2259) |
| `gpui::WindowParams` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L2125) |
| `gpui::WindowProfiler` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L895) |
| `gpui::WindowVisibility` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L102) |

## E2.12: Testing and profiling (18)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::bench` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::bench_group` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L120) |
| `gpui::bench_main` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L131) |
| `gpui::BenchAppContext` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L484) |
| `gpui::BenchReport` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L100) |
| `gpui::BenchWindowContext` | struct | macOS (all features), Linux | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/bench_context.rs#L1151) |
| `gpui::HeadlessAppContext` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/headless_app_context.rs#L38) |
| `gpui::property_test` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::test` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L109) |
| `gpui::test` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/test.rs#L1) |
| `gpui::TestApp` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_app.rs#L40) |
| `gpui::TestAppContext` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L21) |
| `gpui::TestAppWindow` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_app.rs#L320) |
| `gpui::TestScreenCaptureSource` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/platform.rs#L60) |
| `gpui::TestScreenCaptureStream` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/platform.rs#L63) |
| `gpui::VisualTestAppContext` | struct | macOS | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/visual_test_context.rs#L21) |
| `gpui::VisualTestContext` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app/test_context.rs#L783) |
| `gpui::VisualTestPlatform` | struct | macOS | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/visual_test.rs#L31) |

## E2.15: Types, preludes, and interop (17)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::ArcCow` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L137) |
| `gpui::Cascade` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L80) |
| `gpui::CascadeSlot` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L93) |
| `gpui::IsEmpty` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L66) |
| `gpui::prelude` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L1) |
| `gpui::prelude::BorrowAppContext` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L300) |
| `gpui::prelude::FluentBuilder` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/util.rs#L11) |
| `gpui::prelude::IntoElement` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L6) |
| `gpui::prelude::Refineable` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) |
| `gpui::prelude::Refineable` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L7) |
| `gpui::prelude::VisualContext` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/prelude.rs#L8) |
| `gpui::prelude::VisualContext` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L260) |
| `gpui::Refineable` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L1) |
| `gpui::Refineable` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/refineable/src/refineable.rs#L29) |
| `gpui::Result` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L93) |
| `gpui::SharedString` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui_shared_string/gpui_shared_string.rs#L15) |
| `gpui::SharedUri` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/shared_uri.rs#L7) |

## E2.16: Internals (11)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::ArenaClearNeeded` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window.rs#L457) |
| `gpui::AtlasKey` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1376) |
| `gpui::AtlasTextureId` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1494) |
| `gpui::AtlasTextureKind` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1512) |
| `gpui::AtlasTile` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1480) |
| `gpui::inspector_reflection` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/inspector.rs#L227) |
| `gpui::NoopTextSystem` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1209) |
| `gpui::PlatformAtlas` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1427) |
| `gpui::styled_reflection` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/styled.rs#L20) |
| `gpui::ThreadedDispatcher` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/threaded_dispatcher.rs#L28) |
| `gpui::Tiling` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L759) |

## Excluded: documentation-only conceptual module (2)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::_accessibility` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/_accessibility.rs#L1) |
| `gpui::_ownership_and_data_flow` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/_ownership_and_data_flow.rs#L1) |

## Excluded: foreign crate module reexport (3)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::accesskit` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L89) |
| `gpui::http_client` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L138) |
| `gpui::proptest` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L66) |

## Excluded: foreign macro reexport (1)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::ctor` | macro | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L99) |

## Excluded: hidden implementation support (54)

| API | Kind | Observed target | Source |
| --- | --- | --- | --- |
| `gpui::ActionStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L11) |
| `gpui::ActionTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L150) |
| `gpui::ActiveTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L98) |
| `gpui::AppCell` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L80) |
| `gpui::AppRef` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L118) |
| `gpui::AppRefMut` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L131) |
| `gpui::AtlasTextureList` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1442) |
| `gpui::GlobalThreadTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L417) |
| `gpui::GuardedTaskTimings` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L414) |
| `gpui::InteractiveTextState` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/elements/text.rs#L999) |
| `gpui::MacroActionBuilder` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L268) |
| `gpui::MacroActionData` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/action.rs#L273) |
| `gpui::PlatformDispatcher` | trait | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1126) |
| `gpui::PriorityQueueReceiver` | struct | Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L165) |
| `gpui::PriorityQueueSender` | struct | Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L136) |
| `gpui::private` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L75) |
| `gpui::private::anyhow` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L76) |
| `gpui::private::inventory` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L77) |
| `gpui::private::schemars` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L78) |
| `gpui::private::serde` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L79) |
| `gpui::private::serde_json` | module | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/gpui.rs#L80) |
| `gpui::profiler::ActionStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L11) |
| `gpui::profiler::ActionTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler/actions.rs#L150) |
| `gpui::profiler::ActiveTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L98) |
| `gpui::profiler::GlobalThreadTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L417) |
| `gpui::profiler::GuardedTaskTimings` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L414) |
| `gpui::profiler::ProfilingCollector` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L339) |
| `gpui::profiler::TaskStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L424) |
| `gpui::profiler::TaskTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L78) |
| `gpui::profiler::ThreadTaskStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L180) |
| `gpui::profiler::ThreadTaskTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L129) |
| `gpui::profiler::ThreadTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L556) |
| `gpui::profiler::ThreadTimingsDelta` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L327) |
| `gpui::profiler::YieldTime` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L74) |
| `gpui::ProfilingCollector` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L339) |
| `gpui::queue::Iter` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L369) |
| `gpui::queue::PriorityQueueReceiver` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L165) |
| `gpui::queue::PriorityQueueSender` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L136) |
| `gpui::queue::RecvError` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L195) |
| `gpui::queue::SendError` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L185) |
| `gpui::queue::TryIter` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/queue.rs#L380) |
| `gpui::RunnableVariant` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1112) |
| `gpui::SystemWindowTab` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/app.rs#L356) |
| `gpui::TasksIncluded` | enum | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1118) |
| `gpui::TaskStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L424) |
| `gpui::TaskTiming` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L78) |
| `gpui::TestDispatcher` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform/test/dispatcher.rs#L17) |
| `gpui::ThreadTaskStatistics` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L180) |
| `gpui::ThreadTaskTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L129) |
| `gpui::ThreadTimings` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L556) |
| `gpui::ThreadTimingsDelta` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L327) |
| `gpui::TimerResolutionGuard` | type | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/platform.rs#L1115) |
| `gpui::ViewElement` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/view.rs#L240) |
| `gpui::YieldTime` | struct | macOS, Linux, Windows, WebAssembly | [source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/profiler.rs#L74) |
