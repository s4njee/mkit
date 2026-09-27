# Package an app for users

Distribution guidance checked 2026-09-24; recheck official requirements before shipping. This workspace builds an explicitly ad-hoc-signed macOS development bundle. It has no Developer ID signature or notarization.

## What you'll build

A runnable GPUI app and a local macOS `.app` bundle, plus the release checks needed before users install it. Run `cargo run -p mkit-example-platform-shipping --locked` to inspect the app. Build the local bundle with `python3 scripts/build_macos_app.py` from the workspace root. Its output is `target/macos/MkitPlatformShipping.app`.

## Concept

`cargo build --release` produces a program, not an installer. The workspace script assembles that program into an `.app`, writes `Info.plist`, and signs the assembled bundle ad hoc for local development. Ad hoc signing is not a Developer ID distribution signature. Each OS has a separate distribution route. Apple describes a signed, hardened, notarized Developer ID app for direct Mac distribution. Microsoft distinguishes Store MSIX signing from non-Store signing. Linux offers formats such as Flatpak and AppImage with different runtime and integration rules. These are current platform requirements, not GPUI APIs: [Apple](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), [Microsoft](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), [Flatpak](https://docs.flatpak.org/en/latest/first-build.html), [AppImage](https://docs.appimage.org/packaging-guide/distribution.html).

## Minimal compiling example

```rust
{{#include ../../../examples/platform_shipping/src/main.rs:platform_shipping_main}}
```

This compiles and runs the app the local bundle contains. Run `cargo test -p mkit-example-platform-shipping --locked` for its app tests and macOS screenshot. On macOS, build and check the development bundle from the workspace root:

```sh
python3 scripts/build_macos_app.py
python3 scripts/build_macos_app.py --validate-only
plutil -lint target/macos/MkitPlatformShipping.app/Contents/Info.plist
codesign --verify --deep --strict --verbose=2 target/macos/MkitPlatformShipping.app
codesign -dv --verbose=2 target/macos/MkitPlatformShipping.app
```

The local check passed on 2026-09-24. `file` identifies the bundled executable as Mach-O arm64. `codesign -dv` reports identifier `dev.mkit.example.platform-shipping`, `Signature=adhoc`, `TeamIdentifier=not set`, ten `Info.plist` entries, and sealed resources v2. Strict verification accepts this local signature. There is no Developer ID signature, notarization ticket, installer, Windows or Linux package, or verified clean-machine launch. Do not put signing credentials in the book.

## How it works

The entry point initializes GPUI Kit and local diagnostics, then opens two windows. `scripts/build_macos_app.py` builds its release binary, copies it under `Contents/MacOS`, writes bundle metadata to `Contents/Info.plist`, signs the assembled `.app` with `codesign --sign -`, and validates its structure and strict signature. The script does not add a Developer ID identity, hardened-runtime release policy, a notarization ticket, or an installer. For direct Mac distribution, use the proper Developer ID identity and hardened runtime, submit with `notarytool`, staple the ticket where applicable, and verify the artifact users will download. For Windows, choose a Store MSIX path or an MSI/EXE installer path and apply the signing rules for that path. Microsoft says Store MSIX submissions are re-signed by the Store, while Store MSI/EXE submissions and non-Store distribution have separate signing requirements. For Linux, choose a Flatpak manifest with an appropriate runtime and permissions, or an AppImage with its bundled dependencies and desktop metadata. These requirements come from [Apple's bundle layout](https://developer.apple.com/documentation/bundleresources/placing-content-in-a-bundle), [Apple's notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow), [Microsoft's signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), and the [Flatpak](https://docs.flatpak.org/en/latest/first-build.html) and [AppImage](https://docs.appimage.org/packaging-guide/distribution.html) manuals. None of those distribution steps has been run for this example.

Record the release path as evidence, not only a successful build: the produced artifact and target architecture; the tool versions and signing identity used; signature and notarization or package verification results where applicable; and clean-machine install, launch, update, and uninstall results. Include the resource and dynamic-library inventory. A local `cargo build --release` does not produce any of those checks by itself.

## Common mistakes

Treating a passing local `codesign --verify` as release readiness is a mistake here: the inspected bundle explicitly reports `Signature=adhoc` and no Team ID. Submitting a Mac app signed with a development or self-signed certificate can fail notarization; Apple requires Developer ID for that route. Check the identity and hardened runtime before submission. See [Apple's notarization troubleshooting](https://developer.apple.com/documentation/security/resolving-common-notarization-issues). A self-signed MSIX certificate is suitable for controlled testing after trust setup, not general public distribution; see [Microsoft's options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options). These are OS distribution issues, not GPUI bugs; no matching official Zed report is being claimed.

## Exercises

Create a packaging pipeline for one target OS. Install its release artifact on a clean machine of the target architecture and record signature validation, launch, update, and uninstall results. Recheck the linked platform rules on the release date.

## API reference links

No GPUI public API is used by the distribution process. Consult [Apple notarization](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution), [Microsoft signing](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options), [Flatpak](https://docs.flatpak.org/en/latest/first-build.html), and [AppImage](https://docs.appimage.org/packaging-guide/distribution.html).
