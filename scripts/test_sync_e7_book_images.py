import json
import tempfile
import unittest
from pathlib import Path

from sync_e7_book_images import sync


class SyncE7BookImagesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.book_e7 = self.root / "book/src/images/e7"
        self.book_e7.mkdir(parents=True)
        self.registry = self.root / "registry"
        self.component = self.registry / "demo"
        (self.component / "tests/baselines/demo/idle").mkdir(parents=True)

    def write_baseline(self, theme, scale, payload):
        path = self.component / "tests/baselines/demo/idle" / f"{theme}-{scale}x.png"
        path.write_bytes(payload)
        return {
            "state": "idle",
            "theme": theme,
            "scale": scale,
            "baseline": f"demo/idle/{theme}-{scale}x.png",
        }

    def write_manifest(self, cases):
        (self.component / "tests/conformance.json").write_text(
            json.dumps({"component": "demo", "screenshot_cases": cases})
        )

    def test_dark_1x_image_is_replaced_by_matching_dark_2x_baseline(self):
        dark_1x = self.write_baseline("dark", 1, b"dark-1x-bytes")
        dark_2x = self.write_baseline("dark", 2, b"dark-2x-bytes")
        self.write_manifest([dark_1x, dark_2x])
        image = self.book_e7 / "demo.png"
        image.write_bytes(b"dark-1x-bytes")

        changed, current, skipped = sync(self.book_e7, self.registry)
        self.assertEqual(changed, ["demo"])
        self.assertEqual(current, [])
        self.assertEqual(skipped, [])
        self.assertEqual(image.read_bytes(), b"dark-2x-bytes")

        changed, current, _ = sync(self.book_e7, self.registry)
        self.assertEqual(changed, [])
        self.assertEqual(current, ["demo"])

    def test_light_1x_image_is_replaced_by_matching_dark_2x_baseline(self):
        light_1x = self.write_baseline("light", 1, b"light-1x-bytes")
        dark_2x = self.write_baseline("dark", 2, b"dark-2x-bytes")
        self.write_manifest([light_1x, dark_2x])
        image = self.book_e7 / "demo.png"
        image.write_bytes(b"light-1x-bytes")

        changed, current, skipped = sync(self.book_e7, self.registry)
        self.assertEqual(changed, ["demo"])
        self.assertEqual(image.read_bytes(), b"dark-2x-bytes")

    def test_image_without_a_matching_baseline_is_skipped(self):
        dark_1x = self.write_baseline("dark", 1, b"dark-1x-bytes")
        dark_2x = self.write_baseline("dark", 2, b"dark-2x-bytes")
        self.write_manifest([dark_1x, dark_2x])
        image = self.book_e7 / "demo.png"
        image.write_bytes(b"something-else")

        changed, current, skipped = sync(self.book_e7, self.registry)
        self.assertEqual(changed, [])
        self.assertEqual(current, [])
        self.assertEqual(len(skipped), 1)
        self.assertEqual(image.read_bytes(), b"something-else")


if __name__ == "__main__":
    unittest.main()
