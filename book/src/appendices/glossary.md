# Glossary

**Action.** A typed app command that can be dispatched by a key binding or another interaction. See [actions](../interaction/actions.md).

**AccessKit tree.** The accessibility nodes GPUI sends to platform adapters. Roles, names, and stable IDs make controls discoverable. See the [pinned accessibility source](https://github.com/zed-industries/zed/blob/d89e9c2124b2786a390c7a451c7488601b4da2e1/crates/gpui/src/window/a11y.rs#L35-L95).

**Anchor.** A named pair of `ANCHOR` comments around compiling example code. mdBook includes that source in a chapter.

**App context.** The access object used to create entities, read globals, open windows, and schedule work. See [contexts](../state/contexts.md).

**Caret.** The insertion position in editable text. GPUI's platform input ranges use UTF-16 code units. See [selection](../text-input/selection-and-undo.md).

**Entity.** A retained handle to app-owned state. A view is an entity that renders. See [entities](../state/entities.md).

**Focus handle.** A stable handle GPUI uses to route keyboard focus. See [focus](../interaction/focus.md).

**Global.** App-wide state stored by type in GPUI's context. See [globals](../state/globals.md).

**Headless render.** Rendering without a visible desktop window. The current screenshot harness uses the macOS offscreen renderer. See [harness screenshots](../testing/harness-screenshots.md).

**IME.** An input method editor that composes text from key sequences. Its temporary text is a marked range. See [composition](../text-input/ime-composition.md).

**Immediate-style rendering.** A view rebuilds element descriptions on render, while its entity state persists across frames. See [How GPUI thinks](../getting-started/how-gpui-thinks.md).

**Key context.** A scope that selects which key bindings apply while a view has focus. See [actions](../interaction/actions.md).

**Marked text.** The temporary composition range supplied by an IME before commit. See [composition](../text-input/ime-composition.md).

**Notify.** `cx.notify()` marks an entity changed so its observers and view can update. See [reactivity](../state/reactivity.md).

**Prepaint.** GPUI's frame phase after layout and before paint, used for final bounds and hit-test preparation. See [How GPUI thinks](../getting-started/how-gpui-thinks.md).

**RenderOnce.** A stateless component rendered from a builder value rather than a retained view entity. See [views and components](../state/views-and-components.md).

**Subscription.** A retained connection to change notifications or typed events. Dropping it stops delivery. See [reactivity](../state/reactivity.md).

**UTF-16 range.** Offsets counted in 16-bit code units by platform input methods. They differ from UTF-8 byte offsets and can differ from visible character boundaries. See [selection](../text-input/selection-and-undo.md).

**Visual test context.** A GPUI test window that can draw and simulate input, then let a test inspect state. See [testing](../testing/test-contexts.md).
