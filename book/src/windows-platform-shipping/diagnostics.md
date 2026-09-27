# Logs and crash diagnostics

## What you'll build

A local diagnostic record for an app error and a caught panic summary. Run `cargo test -p mkit-example-platform-shipping --lib --locked`. This example does not upload reports or configure a crash service.

## Concept

A log records an event while the app is running. A crash report records where a process stopped. GPUI uses the Rust `log` facade for some internal warnings, so an application chooses and initializes its own log receiver. A report can only name useful functions if the matching symbols remain available. Apple [recommends retaining symbol information](https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs); Windows Error Reporting [can collect local dumps when configured](https://learn.microsoft.com/en-us/windows/win32/wer/collecting-user-mode-dumps).

## Minimal compiling example

```rust
{{#include ../../../examples/platform_shipping/src/lib.rs:diagnostics_setup}}
```

The runnable app installs this logger and panic hook in its [entry point](packaging.md). Run `cargo test -p mkit-example-platform-shipping --lib --locked`; `diagnostics_retain_tagged_errors_and_panic_summary_locally` checks a tagged log record and a caught panic message. It does not cause an uncaught process crash. The on-screen “Record local diagnostic” control currently increments its display counter only; it does not call `record_tagged_error` or test persistence.

## How it works

`install_diagnostics` registers a process-wide `log` receiver that keeps entries in memory. The binary records `APP_START` and installs a panic hook that preserves the prior hook. The unit test sends `E211` through the logger, catches a deliberate panic, stores its message, and finds both entries in the local sink. The test does not exercise the installed panic hook or OS crash reporting. The sink has no file writer or upload route, so entries disappear when the process exits. If release diagnostics are needed, choose a storage or collection path, retention limit, consent policy, and redaction rules before relying on reports. Preserve symbols for each shipped build and keep a way to match a report to its binary; on macOS, retrieve and symbolicate OS crash reports, and on Windows configure Windows Error Reporting if local dumps are needed. Keep private paths and user text out of routine logs. The pinned GPUI [a11y source emits `log` warnings](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L235-L262); it does not initialize the app's logger.

## Common mistakes

Dropping symbols after release makes crash addresses harder to interpret. Keep the matching release symbols and build ID; see [Apple's crash-report guide](https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs). Assuming Windows creates local dumps automatically also fails: the [WER LocalDumps feature](https://learn.microsoft.com/en-us/windows/win32/wer/collecting-user-mode-dumps) is disabled by default. No relevant official Zed issue has been verified for this app-owned diagnostic setup.

## Exercises

Emit a second tagged recoverable error and assert its local record. To test the panic hook, use an isolated child process, then locate the OS report and confirm its symbols match the binary. Inspect logs for sensitive content before shipping.

## API reference links

No GPUI diagnostics initializer is used. The relevant pinned behavior is [GPUI's `log` warning emission](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L235-L262). For OS reports, see [Apple crash reports](https://developer.apple.com/documentation/xcode/diagnosing-issues-using-crash-reports-and-device-logs) and [Windows Error Reporting](https://learn.microsoft.com/en-us/windows/win32/wer/collecting-user-mode-dumps).
