# Versions

The baseline is `gpui-pre` **0.3.5** and GPUI Kit (`gpui-kit`) **0.6.4**. The published `gpui-kit` 0.6.4 manifest depends on the `gpui-pre` package at version `0.3.5`, under the dependency name `gpui`. mkit uses the same `gpui-pre` release to allow both libraries in one app.

In a Cargo manifest, the intended direct dependency is:

```toml
[dependencies]
gpui_pre = { package = "gpui-pre", version = "=0.3.5" }
```

The leading `=` is an exact version requirement. The workspace uses the `gpui_pre` dependency key, which is also the Rust import name. Keep the pin aligned across mkit crates, examples, and coexistence checks. GPUI Kit itself declares a compatible `0.3.5` requirement; the workspace lockfile and the mkit exact pin determine the version resolved here. Changing the pin requires a build and compatibility check with GPUI Kit, plus updates to examples and this appendix.

The crates.io package named `gpui` is distinct from `gpui-pre`. GPUI Kit renames its `gpui-pre` dependency to `gpui` locally; that does not change the underlying package. Do not combine packages with different GPUI type identities in the same component boundary.

These versions describe the foundation baseline, not a promise that future GPUI or GPUI Kit releases remain compatible. See [Install](../install.md) for build prerequisites.

## Pinning and upgrade policy

Treat the exact `gpui-pre` version, the resolved `Cargo.lock`, and the inventory's Zed revision as one reviewed snapshot. The name `gpui` in newer upstream examples may mean a different package or revision; check the package and public signature before copying a call. Cargo's [dependency specification](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) explains both the `package` rename and exact `=` requirement.

To propose a new snapshot, first record the package version and upstream commit. Update the workspace pin and lockfile in one change, regenerate `book/inventory.md` with `python3 scripts/generate_gpui_inventory.py --regenerate` and its other target extracts where available, then run `python3 scripts/generate_gpui_inventory.py --check`. Build and test the workspace with `--locked`, run the coexistence check with GPUI Kit, and build the book so every anchored example is checked. Compare the old and new inventory and write a [migration note](migration.md) for changed names or behavior before claiming compatibility. A maintainer reviews the API and book accuracy; this is a policy for a future upgrade, not a claim that one has already run.
