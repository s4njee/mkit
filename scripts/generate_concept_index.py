#!/usr/bin/env python3
"""Generate the concept index from the pinned inventory and reviewed routes.

Run `python3 scripts/generate_concept_index.py` to update the index or pass
`--check` to verify it is current. Routes are deliberately curated here; a
missing route is labeled Future coverage rather than inferred from a section.
"""

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "book/inventory.md"
JSON_INVENTORY = ROOT / "book/gpui-api-inventory.json"
OUTPUT = ROOT / "book/src/appendices/concept-index.md"
EXPECTED_COUNT = 634
REVISION = "d89e9c2124b2786a390c7a451c7488601b4da2e1"
SOURCE_PREFIX = f"https://github.com/zed-industries/zed/blob/{REVISION}/"

# (public path, kind, path and line within the pinned source) -> published page.
# This is intentionally a small manual map: entries not listed stay Future coverage.
ROUTES = {
    ("gpui::App", "struct", "crates/gpui/src/app.rs#L686"): "../state/contexts.md",
    ("gpui::Application", "struct", "crates/gpui/src/app.rs#L144"): "../examples/hello.md",
    ("gpui::Window", "struct", "crates/gpui/src/window.rs#L1144"): "../getting-started/how-gpui-thinks.md",
    ("gpui::WindowOptions", "struct", "crates/gpui/src/platform.rs#L2044"): "../examples/hello.md",
    ("gpui::Context", "struct", "crates/gpui/src/app/context.rs#L21"): "../state/contexts.md",
    ("gpui::Entity", "struct", "crates/gpui/src/app/entity_map.rs#L414"): "../state/entities.md",
    ("gpui::EventEmitter", "trait", "crates/gpui/src/gpui.rs#L296"): "../state/reactivity.md",
    ("gpui::Global", "trait", "crates/gpui/src/global.rs#L22"): "../state/globals.md",
    ("gpui::Render", "macro", "crates/gpui/src/gpui.rs#L109"): "../state/views-and-components.md",
    ("gpui::Render", "trait", "crates/gpui/src/element.rs#L163"): "../state/views-and-components.md",
    ("gpui::RenderOnce", "trait", "crates/gpui/src/element.rs#L179"): "../state/views-and-components.md",
    ("gpui::Subscription", "struct", "crates/gpui/src/subscription.rs#L150"): "../state/reactivity.md",
    ("gpui::WeakEntity", "struct", "crates/gpui/src/app/entity_map.rs#L740"): "../state/entities.md",
    ("gpui::Div", "struct", "crates/gpui/src/elements/div.rs#L1800"): "../elements/div-and-layout.md",
    ("gpui::Image", "struct", "crates/gpui/src/platform.rs#L2774"): "../elements/images-and-svg.md",
    ("gpui::IntoElement", "macro", "crates/gpui/src/gpui.rs#L109"): "../state/views-and-components.md",
    ("gpui::IntoElement", "trait", "crates/gpui/src/element.rs#L145"): "../state/views-and-components.md",
    ("gpui::ObjectFit", "enum", "crates/gpui/src/style.rs#L29"): "../elements/images-and-svg.md",
    ("gpui::Styled", "trait", "crates/gpui/src/styled.rs#L22"): "../elements/div-and-layout.md",
    ("gpui::Action", "macro", "crates/gpui/src/action.rs#L4"): "../interaction/actions.md",
    ("gpui::Action", "trait", "crates/gpui/src/action.rs#L118"): "../interaction/actions.md",
    ("gpui::ClipboardItem", "struct", "crates/gpui/src/platform.rs#L2522"): "../interaction/clipboard.md",
    ("gpui::ExternalPaths", "struct", "crates/gpui/src/interactive.rs#L694"): "../interaction/drag-and-drop.md",
    ("gpui::FocusHandle", "struct", "crates/gpui/src/window.rs#L527"): "../interaction/focus.md",
    ("gpui::KeyBinding", "struct", "crates/gpui/src/keymap/binding.rs#L10"): "../interaction/actions.md",
    ("gpui::KeyContext", "struct", "crates/gpui/src/keymap/context.rs#L10"): "../interaction/actions.md",
    ("gpui::MouseDownEvent", "struct", "crates/gpui/src/interactive.rs#L148"): "../interaction/mouse.md",
    ("gpui::ScrollWheelEvent", "struct", "crates/gpui/src/interactive.rs#L522"): "../interaction/mouse.md",
    ("gpui::ElementInputHandler", "struct", "crates/gpui/src/input.rs#L128"): "../text-input/handler-contract.md",
    ("gpui::EntityInputHandler", "trait", "crates/gpui/src/input.rs#L13"): "../text-input/handler-contract.md",
    ("gpui::UTF16Selection", "struct", "crates/gpui/src/platform.rs#L1780"): "../text-input/selection-and-undo.md",
    ("gpui::Element", "trait", "crates/gpui/src/element.rs#L51"): "../getting-started/how-gpui-thinks.md",
    ("gpui::ListState", "struct", "crates/gpui/src/elements/list.rs#L54"): "../elements/conditional-lists.md",
    ("gpui::BackgroundExecutor", "struct", "crates/gpui/src/executor.rs#L19"): "../async/executors-and-tasks.md",
    ("gpui::ForegroundExecutor", "struct", "crates/gpui/src/executor.rs#L27"): "../async/executors-and-tasks.md",
    ("gpui::Task", "struct", "crates/gpui/src/executor.rs#L13"): "../async/executors-and-tasks.md",
    ("gpui::HeadlessAppContext", "struct", "crates/gpui/src/app/headless_app_context.rs#L38"): "../testing/harness-screenshots.md",
    ("gpui::TestAppContext", "struct", "crates/gpui/src/app/test_context.rs#L21"): "../testing/test-contexts.md",
    ("gpui::VisualTestContext", "struct", "crates/gpui/src/app/test_context.rs#L783"): "../testing/test-contexts.md",
    ("gpui::AnyEntity", "struct", "crates/gpui/src/app/entity_map.rs#L246"): "../architecture/entity-map.md",
    ("gpui::AppContext", "trait", "crates/gpui/src/gpui.rs#L172"): "../state/contexts.md",
    ("gpui::EntityId", "struct", "crates/gpui/src/app/entity_map.rs#L27"): "../architecture/entity-map.md",
    ("gpui::AnyElement", "struct", "crates/gpui/src/element.rs#L588"): "../state/views-and-components.md",
    ("gpui::BoxShadow", "struct", "crates/gpui/src/style.rs#L349"): "../custom-rendering/canvas-primitives.md",
    ("gpui::Display", "enum", "crates/gpui/src/style.rs#L1131"): "../elements/div-and-layout.md",
    ("gpui::GlobalElementId", "struct", "crates/gpui/src/element.rs#L213"): "../elements/conditional-lists.md",
    ("gpui::ImageSource", "enum", "crates/gpui/src/elements/img.rs#L42"): "../elements/images-and-svg.md",
    ("gpui::Overflow", "enum", "crates/gpui/src/style.rs#L1208"): "../elements/size-and-scroll.md",
    ("gpui::ParentElement", "trait", "crates/gpui/src/element.rs#L188"): "../elements/div-and-layout.md",
    ("gpui::RenderImage", "struct", "crates/gpui/src/assets.rs#L43"): "../elements/images-and-svg.md",
    ("gpui::StyledText", "struct", "crates/gpui/src/elements/text.rs#L391"): "../elements/text.md",
    ("gpui::TextRun", "struct", "crates/gpui/src/text_system.rs#L999"): "../elements/text.md",
    ("gpui::TextStyle", "struct", "crates/gpui/src/style.rs#L438"): "../elements/text.md",
    ("gpui::TextSystem", "struct", "crates/gpui/src/text_system.rs#L51"): "../architecture/text-systems.md",
    ("gpui::ElementId", "enum", "crates/gpui/src/window.rs#L7218"): "../elements/conditional-lists.md",
    ("gpui::FileDropEvent", "enum", "crates/gpui/src/interactive.rs#L737"): "../interaction/drag-and-drop.md",
    ("gpui::InteractiveElement", "trait", "crates/gpui/src/elements/div.rs#L774"): "../elements/conditional-lists.md",
    ("gpui::Menu", "struct", "crates/gpui/src/platform/app_menu.rs#L4"): "../interaction/menus.md",
    ("gpui::MenuItem", "enum", "crates/gpui/src/platform/app_menu.rs#L76"): "../interaction/menus.md",
    ("gpui::InputHandler", "trait", "crates/gpui/src/platform.rs#L1793"): "../text-input/handler-contract.md",
    ("gpui::Animation", "struct", "crates/gpui/src/elements/animation.rs#L15"): "../custom-rendering/animation.md",
    ("gpui::AnimationExt", "trait", "crates/gpui/src/elements/animation.rs#L82"): "../custom-rendering/animation.md",
    ("gpui::Deferred", "struct", "crates/gpui/src/elements/deferred.rs#L16"): "../custom-rendering/overlays.md",
    ("gpui::PathBuilder", "struct", "crates/gpui/src/path_builder.rs#L25"): "../custom-rendering/canvas-primitives.md",
    ("gpui::SurfaceSource", "enum", "crates/gpui/src/elements/surface.rs#L11"): "../custom-rendering/gpu-surfaces.md",
    ("gpui::AsyncApp", "struct", "crates/gpui/src/app/async_context.rs#L22"): "../state/contexts.md",
    ("gpui::PathPromptOptions", "struct", "crates/gpui/src/platform.rs#L2355"): "../cookbook/input/file-picker.md",
    ("gpui::PlatformTextSystem", "trait", "crates/gpui/src/platform.rs#L1173"): "../architecture/text-systems.md",
    ("gpui::Role", "enum", "crates/gpui/src/gpui.rs#L91"): "../windows-platform-shipping/accessibility.md",
    ("gpui::SystemNotification", "struct", "crates/gpui/src/platform.rs#L437"): "../cookbook/windows/system-notification.md",
    ("gpui::Anchored", "struct", "crates/gpui/src/elements/anchored.rs#L16"): "../custom-rendering/overlays.md",
    ("gpui::Surface", "struct", "crates/gpui/src/elements/surface.rs#L25"): "../custom-rendering/gpu-surfaces.md",
    ("gpui::UniformList", "struct", "crates/gpui/src/elements/uniform_list.rs#L58"): "../custom-rendering/virtualization.md",
    ("gpui::TitlebarOptions", "struct", "crates/gpui/src/platform.rs#L2245"): "../windows-platform-shipping/windows-and-appearance.md",
    ("gpui::WindowBounds", "enum", "crates/gpui/src/platform.rs#L2182"): "../windows-platform-shipping/windows-and-appearance.md",
    ("gpui::NoopTextSystem", "struct", "crates/gpui/src/platform.rs#L1209"): "../architecture/text-systems.md",
    ("gpui::Style", "struct", "crates/gpui/src/style.rs#L180"): "../elements/size-and-scroll.md",
    ("gpui::actions", "macro", "crates/gpui/src/action.rs#L25"): "../interaction/actions.md",
    ("gpui::DispatchPhase", "enum", "crates/gpui/src/window.rs#L91"): "../interaction/actions.md",
    ("gpui::KeyDownEvent", "struct", "crates/gpui/src/interactive.rs#L25"): "../getting-started/how-gpui-thinks.md",
    ("gpui::Keystroke", "struct", "crates/gpui/src/platform/keystroke.rs#L18"): "../cookbook/input/rebind-navigation.md",
    ("gpui::StatefulInteractiveElement", "trait", "crates/gpui/src/elements/div.rs#L1303"): "../windows-platform-shipping/accessibility.md",
    ("gpui::Scene", "struct", "crates/gpui/src/scene.rs#L41"): "../architecture/frame-pipeline.md",
}

INTRO = """# Concept index

This index covers all **{count}** public inventory rows at the pinned `gpui-pre` 0.3.5 snapshot. Each source link points to the pinned Zed commit. A chapter link means the named concept is introduced there; it does not promise complete coverage of every method. **Future coverage** means the current book has no verified chapter for that item. See the [API inventory](api-inventory.md) for target and feature-profile caveats.

The index is generated from `book/inventory.md` with a small manually reviewed chapter-route map. Review routes and counts after every inventory refresh.
"""


@dataclass(frozen=True)
class Item:
    path: str
    kind: str
    source_url: str

    @property
    def source_suffix(self):
        if not self.source_url.startswith(SOURCE_PREFIX):
            raise ValueError(f"source URL is not from pinned revision: {self.source_url}")
        return self.source_url.removeprefix(SOURCE_PREFIX)

    @property
    def route_key(self):
        return (self.path, self.kind, self.source_suffix)


@dataclass
class Group:
    heading: str
    items: list[Item]


def parse_inventory(markdown: str) -> list[Group]:
    groups: list[Group] = []
    current = None
    for line_number, line in enumerate(markdown.splitlines(), 1):
        if line.startswith("## "):
            current = Group(line[3:].strip(), [])
            groups.append(current)
            continue
        if not line.startswith("| ") or line.startswith("| API |") or line.startswith("| ---"):
            continue
        if current is None:
            raise ValueError(f"inventory row outside a section at line {line_number}")
        columns = [column.strip() for column in line.strip("|").split("|")]
        if len(columns) != 4:
            raise ValueError(f"expected four inventory columns at line {line_number}")
        api = columns[0]
        if not (api.startswith("`") and api.endswith("`")):
            raise ValueError(f"expected code-formatted API path at line {line_number}: {api}")
        source = re.fullmatch(r"\[source\]\(([^)]+)\)", columns[3])
        if not source:
            raise ValueError(f"expected source link at line {line_number}")
        current.items.append(Item(api[1:-1], columns[1], source.group(1)))

    identities = {}
    duplicates = []
    for group in groups:
        for item in group.items:
            if item.route_key in identities:
                duplicates.append((item.route_key, identities[item.route_key], group.heading))
            else:
                identities[item.route_key] = group.heading
    if duplicates:
        details = "; ".join(f"{key!r} appears in {first!r} and {second!r}" for key, first, second in duplicates)
        raise ValueError(f"duplicate API path/kind/source rows: {details}")
    return groups


def render(groups: list[Group]) -> str:
    total = sum(len(group.items) for group in groups)
    if total != EXPECTED_COUNT:
        raise ValueError(f"inventory has {total} rows; expected {EXPECTED_COUNT}; review and update EXPECTED_COUNT")
    output = [INTRO.format(count=total).rstrip(), ""]
    used_routes = set()
    for group in groups:
        heading = re.sub(r" \(\d+\)$", "", group.heading)
        heading = re.sub(r"^E\d+(?:\.\d+)*: ", "", heading)
        output.append(f"## {heading} ({len(group.items)})")
        output.extend(("", "| Public item | Kind | Teaching route |", "| --- | --- | --- |"))
        for item in group.items:
            route = ROUTES.get(item.route_key)
            route_label = f"[Published chapter]({route})" if route else "Future coverage"
            output.append(f"| [`{item.path}`]({item.source_url}) | {item.kind} | {route_label} |")
            if route:
                used_routes.add(item.route_key)
        output.append("")
    unused = set(ROUTES) - used_routes
    if unused:
        raise ValueError("curated routes do not match current inventory rows: " + ", ".join(map(repr, sorted(unused))))
    # Keep the existing Markdown file's final blank line for a no-op regeneration.
    return "\n".join(output).rstrip() + "\n\n"


def generate() -> str:
    groups = parse_inventory(INVENTORY.read_text())
    data = json.loads(JSON_INVENTORY.read_text())
    json_count = data.get("count")
    actual_count = sum(len(group.items) for group in groups)
    if json_count != actual_count:
        raise ValueError(f"inventory markdown has {actual_count} rows but JSON inventory records {json_count}")
    if len(data.get("items", [])) != json_count:
        raise ValueError(f"JSON inventory count says {json_count}, but contains {len(data.get('items', []))} item records")
    markdown_keys = {
        (item.path, item.kind, item.source_url)
        for group in groups
        for item in group.items
    }
    json_keys = {
        (item["path"], item["kind"], item["source_url"])
        for item in data["items"]
    }
    if len(json_keys) != len(data["items"]):
        raise ValueError("JSON inventory contains duplicate API path/kind/source rows")
    if markdown_keys != json_keys:
        missing = sorted(json_keys - markdown_keys)
        unexpected = sorted(markdown_keys - json_keys)
        raise ValueError(f"inventory Markdown/JSON rows differ; missing={missing[:3]!r}, unexpected={unexpected[:3]!r}")
    return render(groups)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if the checked-in index differs")
    args = parser.parse_args(argv)
    try:
        generated = generate()
    except (OSError, ValueError, KeyError) as error:
        print(f"concept index generation failed: {error}", file=sys.stderr)
        return 1
    if args.check:
        try:
            current = OUTPUT.read_text()
        except FileNotFoundError:
            print(f"{OUTPUT.relative_to(ROOT)} is missing; run python3 scripts/generate_concept_index.py", file=sys.stderr)
            return 1
        if current != generated:
            print(f"{OUTPUT.relative_to(ROOT)} is stale; run python3 scripts/generate_concept_index.py", file=sys.stderr)
            return 1
        print(f"concept index is current: {EXPECTED_COUNT} rows")
        return 0
    OUTPUT.write_text(generated)
    print(f"wrote {OUTPUT.relative_to(ROOT)} with {EXPECTED_COUNT} rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
