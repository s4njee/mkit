# Settings screen media assets

Both assets in this directory were authored locally for this example; neither is copied from an external source.

- `workspace-landscape.png` is a 160 × 100 RGBA geometric landscape. Run `python3 generate-landscape.py` in this directory to reproduce it. The script builds the PNG directly from simple gradients, circles, and sine-wave ridge shapes using Python's standard library; it downloads nothing.
- `sync-mark.svg` is a hand-authored, original 24 × 24 outline icon containing a single path. It is embedded with `include_bytes!` by the example.

The assets are distributed under the repository's license in the root `LICENSE` file.
