import struct
import tempfile
import unittest
from pathlib import Path

from check_book import check_book, check_built_banner, check_built_links, check_links, rust_includes


class BookCheckTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        (self.root / "book/src/examples").mkdir(parents=True)
        (self.root / "examples/demo/src").mkdir(parents=True)
        (self.root / "examples/demo/tests").mkdir(parents=True)
        (self.root / "Cargo.toml").write_text('[workspace]\nmembers = ["examples/demo"]\n[workspace.dependencies]\ngpui_pre = { package = "gpui-pre", version = "=0.3.5" }\n')
        (self.root / "examples/demo/Cargo.toml").write_text('[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2024"\n')
        (self.root / "examples/demo/src/main.rs").write_text('// ANCHOR: main\nfn main() {}\n// ANCHOR_END: main\n')
        self.page = self.root / "book/src/examples/demo.md"

    def test_anchored_compiling_crate_source_is_accepted(self):
        self.page.write_text('# Demo\n\n```rust\n{{#include ../../../examples/demo/src/main.rs:main}}\n```\n')
        self.assertEqual(rust_includes(self.page, self.root), {self.root / "examples/demo"})
        self.assertEqual(check_book(self.root, compile_examples=False), (1, 1))

    def test_inline_rust_or_missing_anchor_is_rejected(self):
        self.page.write_text('```rust\nfn main() {}\n```\n')
        with self.assertRaisesRegex(ValueError, "only anchored"):
            rust_includes(self.page, self.root)
        self.page.write_text('```rust\n{{#include ../../../examples/demo/src/main.rs:wrong}}\n```\n')
        with self.assertRaisesRegex(ValueError, "missing or empty anchor"):
            rust_includes(self.page, self.root)
        self.page.write_text('```\nfn main() {}\n```\n')
        with self.assertRaisesRegex(ValueError, "explicit supported language"):
            rust_includes(self.page, self.root)

    def test_unlinked_or_nonexample_source_is_rejected(self):
        (self.root / "outside.rs").write_text('// ANCHOR: main\nfn main() {}\n// ANCHOR_END: main\n')
        self.page.write_text('```rust\n{{#include ../../../outside.rs:main}}\n```\n')
        with self.assertRaisesRegex(ValueError, "not Rust source in examples"):
            rust_includes(self.page, self.root)

    def test_local_link_and_heading_fragment(self):
        other = self.root / "book/src/other.md"
        other.write_text('# A Heading\n')
        self.page.write_text('[good](../other.md#a-heading)\n')
        check_links(self.page, self.root)
        self.page.write_text('[bad](../other.md#missing)\n')
        with self.assertRaisesRegex(ValueError, "missing heading"):
            check_links(self.page, self.root)

    def test_screenshot_requires_real_png_and_harness_test(self):
        image = self.root / "book/src/images/demo.png"
        image.parent.mkdir()
        image.write_bytes(b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\x0dIHDR" + struct.pack(">II", 10, 10))
        self.page.write_text('![demo](../images/demo.png)\n')
        with self.assertRaisesRegex(ValueError, "no examples/ harness screenshot test"):
            check_links(self.page, self.root)
        (self.root / "examples/demo/tests/screenshot.rs").write_text(
            'use mkit_harness::screenshot;\n'
            'let image = screenshot(view, size, 1.0, init);\n'
            'if std::env::var_os("UPDATE_SNAPSHOTS").is_some() { image.save(&baseline); }\n'
            'let baseline = "../../book/src/images/demo.png";\n'
        )
        check_links(self.page, self.root)

    def test_e8_registry_baseline_is_accepted_only_when_bytes_match(self):
        image = self.root / "book/src/images/e8-histogram-state.png"
        image.parent.mkdir(parents=True)
        png = b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\x0dIHDR" + struct.pack(">II", 10, 10)
        image.write_bytes(png)
        component = self.root / "registry/histogram/tests"
        baseline = component / "baselines/histogram/light-1x.png"
        baseline.parent.mkdir(parents=True)
        baseline.write_bytes(png)
        (component / "conformance.json").write_text(
            '{"screenshot_cases":[{"platform":"macos",'
            '"baseline":"histogram/light-1x.png"}]}'
        )
        self.page.write_text('![histogram](../images/e8-histogram-state.png)\n')
        check_links(self.page, self.root)
        image.write_bytes(png + b"changed")
        with self.assertRaisesRegex(ValueError, "no examples/ harness screenshot test"):
            check_links(self.page, self.root)

    def test_e8_registry_baseline_with_component_prefix_is_accepted(self):
        image = self.root / "book/src/images/e8-command-palette-filtered.png"
        image.parent.mkdir(parents=True)
        png = b"\x89PNG\r\n\x1a\n" + b"\x00\x00\x00\x0dIHDR" + struct.pack(">II", 10, 10)
        image.write_bytes(png)
        component = self.root / "registry/command-palette/tests"
        baseline = component / "baselines/open_filtered/dark-2x.png"
        baseline.parent.mkdir(parents=True)
        baseline.write_bytes(png)
        (component / "conformance.json").write_text(
            '{"screenshot_cases":[{"platform":"macos",'
            '"baseline":"command-palette/open_filtered/dark-2x.png"}]}'
        )
        self.page.write_text('![filtered](../images/e8-command-palette-filtered.png)\n')
        check_links(self.page, self.root)

    def test_rendered_banner_matches_workspace_pin(self):
        self.page.write_text('# Demo\n')
        (self.root / "book/book.toml").write_text('[book]\ntitle = "demo"\n[preprocessor.version-banner]\ncommand = "node theme/version_banner.mjs"\n')
        output = self.root / "book/book/examples/demo.html"
        output.parent.mkdir(parents=True)
        output.write_text('<div class="gpui-version-banner"><code>gpui-pre 0.3.5</code></div>')
        check_built_banner(self.root)
        output.write_text('<div class="gpui-version-banner"><code>gpui-pre 0.3.4</code></div>')
        with self.assertRaisesRegex(ValueError, "incorrect GPUI pin banner"):
            check_built_banner(self.root)

    def test_built_output_checks_local_links_images_and_fragments(self):
        output = self.root / "book/book"
        page = output / "examples/demo.html"
        target = output / "other.html"
        image = output / "images/demo.png"
        page.parent.mkdir(parents=True, exist_ok=True)
        target.parent.mkdir(parents=True, exist_ok=True)
        image.parent.mkdir(parents=True, exist_ok=True)
        page.write_text('<a href="../other.html#section">ok</a><img src="../images/demo.png">')
        target.write_text('<h1 id="section">Section</h1>')
        image.write_bytes(b"png")
        check_built_links(self.root)

        page.write_text('<a href="../../examples/demo/src/main.rs">source tree only</a>')
        (self.root / "examples/demo/src/main.rs").write_text("fn main() {}")
        with self.assertRaisesRegex(ValueError, "escapes mdBook output"):
            check_built_links(self.root)

        page.write_text('<a href="missing.html">missing</a>')
        with self.assertRaisesRegex(ValueError, "broken built local link"):
            check_built_links(self.root)

        page.write_text('<a href="../other.html#not-there">missing heading</a>')
        with self.assertRaisesRegex(ValueError, "missing built heading"):
            check_built_links(self.root)


if __name__ == "__main__":
    unittest.main()
