//! Small, explicit input scripts for exercising GPUI views in tests.
//!
//! The parser deliberately supports only the documented command forms. Target
//! references are resolved by the caller from its rendered element bounds.

use std::collections::HashMap;

use gpui_pre::{MouseButton, Pixels, Point, Render, TestAppWindow};

/// A command in an input script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptAction {
    /// Dispatch GPUI's space-separated keystroke syntax.
    Press(String),
    /// Type text through GPUI's test input facility.
    Type(String),
    /// Click the center of a named target.
    Click(String),
    /// Drag from the center of one named target to another.
    Drag { from: String, to: String },
}

/// Parsed input script.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputScript {
    actions: Vec<ScriptAction>,
}

/// An input-script parse or execution error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptError(pub String);

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ScriptError {}

impl InputScript {
    /// Parse one command per line, for example `press "down down enter"`.
    pub fn parse(source: &str) -> Result<Self, ScriptError> {
        let mut actions = Vec::new();
        for (line_index, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parsed = parse_line(line)
                .map_err(|e| ScriptError(format!("line {}: {e}", line_index + 1)))?;
            actions.push(parsed);
        }
        Ok(Self { actions })
    }

    pub fn actions(&self) -> &[ScriptAction] {
        &self.actions
    }

    /// Run actions against GPUI's real test input dispatch. Callers provide
    /// element centers because GPUI 0.3.5 does not expose a stable test query
    /// API for mapping semantic refs to layout bounds.
    pub fn run<V>(
        &self,
        app: &mut TestAppWindow<V>,
        targets: &HashMap<String, Point<Pixels>>,
    ) -> Result<(), ScriptError>
    where
        V: 'static + Render,
    {
        self.run_observed(app, targets, |_, _| {})
    }

    /// Run a script and invoke `after_action` after each dispatched command.
    /// The hook can read application entities or capture a screenshot through
    /// the caller's test context. It runs only after a successful dispatch.
    pub fn run_observed<V>(
        &self,
        app: &mut TestAppWindow<V>,
        targets: &HashMap<String, Point<Pixels>>,
        mut after_action: impl FnMut(&ScriptAction, &mut TestAppWindow<V>),
    ) -> Result<(), ScriptError>
    where
        V: 'static + Render,
    {
        for action in &self.actions {
            match action {
                ScriptAction::Press(keys) => app.simulate_keystrokes(keys),
                ScriptAction::Type(text) => app.simulate_input(text),
                ScriptAction::Click(target) => {
                    app.simulate_click(resolve(target, targets)?, MouseButton::Left)
                }
                ScriptAction::Drag { from, to } => {
                    let start = resolve(from, targets)?;
                    let end = resolve(to, targets)?;
                    app.simulate_mouse_move(start);
                    app.simulate_mouse_down(start, MouseButton::Left);
                    app.simulate_mouse_move(end);
                    app.simulate_mouse_up(end, MouseButton::Left);
                }
            }
            after_action(action, app);
        }
        Ok(())
    }
}

/// Parse scripts using the same format as [`InputScript::parse`].
pub fn parse_script(source: &str) -> Result<InputScript, ScriptError> {
    InputScript::parse(source)
}

fn resolve(
    name: &str,
    targets: &HashMap<String, Point<Pixels>>,
) -> Result<Point<Pixels>, ScriptError> {
    targets.get(name).copied().ok_or_else(|| ScriptError(format!("unknown target @{name}")))
}

fn parse_line(line: &str) -> Result<ScriptAction, String> {
    let (command, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    let rest = rest.trim();
    match command {
        "press" => Ok(ScriptAction::Press(parse_quoted(rest)?)),
        "type" => Ok(ScriptAction::Type(parse_quoted(rest)?)),
        "click" => Ok(ScriptAction::Click(parse_ref(rest)?)),
        "drag" => {
            let (from, to) = rest.split_once("->").ok_or("expected `drag @from -> @to`")?;
            Ok(ScriptAction::Drag { from: parse_ref(from.trim())?, to: parse_ref(to.trim())? })
        }
        _ => Err(format!("unknown command `{command}`")),
    }
}

fn parse_ref(value: &str) -> Result<String, String> {
    let name = value.strip_prefix('@').ok_or("expected target reference like `@name`")?;
    if name.is_empty() || name.chars().any(char::is_whitespace) {
        return Err("invalid target reference".into());
    }
    Ok(name.to_owned())
}

fn parse_quoted(value: &str) -> Result<String, String> {
    if !value.starts_with('"') {
        return Err("expected a double-quoted string".into());
    }
    let mut chars = value[1..].char_indices();
    let mut out = String::new();
    while let Some((i, ch)) = chars.next() {
        match ch {
            '"' => {
                if !value[1 + i + 1..].trim().is_empty() {
                    return Err("unexpected text after quoted string".into());
                }
                return Ok(out);
            }
            '\\' => match chars.next() {
                Some((_, '"')) => out.push('"'),
                Some((_, '\\')) => out.push('\\'),
                Some((_, 'n')) => out.push('\n'),
                Some((_, 't')) => out.push('\t'),
                _ => return Err("supported escapes are \", \\\\, \\n, and \\t".into()),
            },
            _ => out.push(ch),
        }
    }
    Err("unterminated quoted string".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::prelude::{
        InteractiveElement, ParentElement, StatefulInteractiveElement, Styled,
    };
    use gpui_pre::{
        Bounds, Context, ElementInputHandler, EntityInputHandler, FocusHandle, IntoElement,
        KeyBinding, MouseButton, Pixels, TextInputConfiguration, UTF16Selection, Window, canvas,
        div, point, px,
    };
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    gpui_pre::actions!(mkit_harness_input_test, [ScriptActionEvent]);

    #[test]
    fn parses_documented_commands_and_escapes() {
        let script = parse_script(
            "press \"down down enter\"\ntype \"hello\\nworld\"\nclick @ok\ndrag @a -> @b",
        )
        .unwrap();
        assert_eq!(
            script.actions(),
            &[
                ScriptAction::Press("down down enter".into()),
                ScriptAction::Type("hello\nworld".into()),
                ScriptAction::Click("ok".into()),
                ScriptAction::Drag { from: "a".into(), to: "b".into() },
            ]
        );
    }

    #[test]
    fn rejects_malformed_commands_and_missing_target() {
        assert!(parse_script("click button").unwrap_err().0.contains("target reference"));
        assert!(parse_script("wait 1").unwrap_err().0.contains("unknown command"));
        assert!(resolve("missing", &HashMap::new()).unwrap_err().0.contains("unknown target"));
    }

    struct Fixture {
        focus: FocusHandle,
        events: Rc<RefCell<Vec<&'static str>>>,
        action_count: Rc<Cell<usize>>,
    }

    impl Render for Fixture {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let clicked = self.events.clone();
            let keyed = self.events.clone();
            let down = self.events.clone();
            let moved = self.events.clone();
            let up = self.events.clone();
            let action_count = self.action_count.clone();
            div()
                .id("target")
                .size_full()
                .key_context("ScriptFixture")
                .track_focus(&self.focus)
                .on_click(move |_, _, _| clicked.borrow_mut().push("click"))
                .on_key_down(move |_, _, _| keyed.borrow_mut().push("key"))
                .on_mouse_down(MouseButton::Left, move |_, _, _| down.borrow_mut().push("down"))
                .on_mouse_move(move |_, _, _| moved.borrow_mut().push("move"))
                .on_mouse_up(MouseButton::Left, move |_, _, _| up.borrow_mut().push("up"))
                .on_action(move |_: &ScriptActionEvent, _, _| {
                    action_count.set(action_count.get() + 1)
                })
        }
    }

    #[test]
    fn dispatches_scripted_pointer_and_keyboard_events_to_focused_gpui_view() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let action_count = Rc::new(Cell::new(0));
        let mut app = gpui_pre::TestApp::new();
        app.update(|cx| {
            cx.bind_keys([KeyBinding::new("ctrl-g", ScriptActionEvent, Some("ScriptFixture"))]);
        });
        let events_for_view = events.clone();
        let action_count_for_view = action_count.clone();
        let mut window = app.open_window(move |window, cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            Fixture { focus, events: events_for_view, action_count: action_count_for_view }
        });
        window.update(|fixture, gpui_window, _| assert!(fixture.focus.is_focused(gpui_window)));
        let script =
            parse_script("click @target\npress \"enter ctrl-g\"\ntype \"hi\"\ndrag @a -> @b")
                .unwrap();
        let targets = HashMap::from([
            ("target".to_owned(), point(px(10.), px(10.))),
            ("a".to_owned(), point(px(20.), px(20.))),
            ("b".to_owned(), point(px(50.), px(50.))),
        ]);
        script.run(&mut window, &targets).unwrap();
        let events = events.borrow();
        assert!(events.contains(&"click"));
        assert_eq!(events.iter().filter(|event| **event == "key").count(), 3);
        assert_eq!(action_count.get(), 1, "ctrl-g should dispatch the registered GPUI action");
        assert_eq!(events.iter().filter(|event| **event == "down").count(), 2);
        assert_eq!(events.iter().filter(|event| **event == "up").count(), 2);
        assert!(events.iter().filter(|event| **event == "move").count() >= 2);
        let drag_down = events.iter().rposition(|event| *event == "down").unwrap();
        let drag_move = events
            .iter()
            .enumerate()
            .skip(drag_down + 1)
            .find(|(_, event)| **event == "move")
            .map(|(i, _)| i)
            .unwrap();
        let drag_up = events
            .iter()
            .enumerate()
            .skip(drag_move + 1)
            .find(|(_, event)| **event == "up")
            .map(|(i, _)| i)
            .unwrap();
        assert!(drag_down < drag_move && drag_move < drag_up);
        drop(events);
        window.update(|_, gpui_window, _| {
            assert!(!gpui_window.is_a11y_active());
            assert!(matches!(
                crate::accessibility::AccessibilitySnapshot::capture(gpui_window),
                Err(crate::accessibility::AccessibilityError::Inactive)
            ));
        });
        drop(window);
        app.update(|cx| cx.shutdown());
    }

    struct TextInputFixture {
        focus: FocusHandle,
        text: String,
        key_events: Rc<Cell<usize>>,
        action_count: Rc<Cell<usize>>,
    }

    impl Render for TextInputFixture {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            let focus = self.focus.clone();
            let key_events = self.key_events.clone();
            let action_count = self.action_count.clone();
            div()
                .size_full()
                .key_context("TextInputFixture")
                .track_focus(&self.focus)
                .on_key_down(move |_, _, _| key_events.set(key_events.get() + 1))
                .on_action(move |_: &ScriptActionEvent, _, _| {
                    action_count.set(action_count.get() + 1)
                })
                .child(
                    canvas(
                        |_, _, _| {},
                        move |bounds, _, window, cx| {
                            window.handle_input(
                                &focus,
                                ElementInputHandler::new(bounds, entity),
                                cx,
                            );
                        },
                    )
                    .size_full(),
                )
        }
    }

    impl EntityInputHandler for TextInputFixture {
        fn text_for_range(
            &mut self,
            range: std::ops::Range<usize>,
            _adjusted_range: &mut Option<std::ops::Range<usize>>,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> Option<String> {
            Some(self.text.get(range).unwrap_or_default().to_owned())
        }

        fn selected_text_range(
            &mut self,
            _ignore_disabled_input: bool,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> Option<UTF16Selection> {
            let offset = self.text.encode_utf16().count();
            Some(UTF16Selection { range: offset..offset, reversed: false })
        }

        fn marked_text_range(
            &self,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> Option<std::ops::Range<usize>> {
            None
        }

        fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

        fn replace_text_in_range(
            &mut self,
            range: Option<std::ops::Range<usize>>,
            text: &str,
            _window: &mut Window,
            cx: &mut Context<Self>,
        ) {
            let range = range.unwrap_or(self.text.len()..self.text.len());
            self.text.replace_range(range, text);
            cx.notify();
        }

        fn replace_and_mark_text_in_range(
            &mut self,
            range: Option<std::ops::Range<usize>>,
            text: &str,
            _new_selected_range: Option<std::ops::Range<usize>>,
            window: &mut Window,
            cx: &mut Context<Self>,
        ) {
            self.replace_text_in_range(range, text, window, cx);
        }

        fn bounds_for_range(
            &mut self,
            _range_utf16: std::ops::Range<usize>,
            _element_bounds: Bounds<Pixels>,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> Option<Bounds<Pixels>> {
            None
        }

        fn character_index_for_point(
            &mut self,
            _point: Point<Pixels>,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> Option<usize> {
            None
        }

        fn text_input_configuration(
            &mut self,
            _window: &mut Window,
            _cx: &mut Context<Self>,
        ) -> TextInputConfiguration {
            TextInputConfiguration::default()
        }
    }

    #[test]
    fn simulate_input_inserts_text_and_keeps_keyboard_actions_on_the_focused_view() {
        let key_events = Rc::new(Cell::new(0));
        let action_count = Rc::new(Cell::new(0));
        let mut app = gpui_pre::TestApp::new();
        app.update(|cx| {
            cx.bind_keys([KeyBinding::new("ctrl-g", ScriptActionEvent, Some("TextInputFixture"))]);
        });
        let key_events_for_view = key_events.clone();
        let action_count_for_view = action_count.clone();
        let mut window = app.open_window(move |window, cx| {
            let focus = cx.focus_handle();
            window.focus(&focus, cx);
            TextInputFixture {
                focus,
                text: String::new(),
                key_events: key_events_for_view,
                action_count: action_count_for_view,
            }
        });

        window.update(|fixture, gpui_window, _| assert!(fixture.focus.is_focused(gpui_window)));
        let script = parse_script("type \"hello GPUI\"\npress \"ctrl-g\"").unwrap();
        script.run(&mut window, &HashMap::new()).unwrap();
        window.update(|fixture, gpui_window, _| {
            assert!(fixture.focus.is_focused(gpui_window));
            assert_eq!(fixture.text, "hello GPUI");
        });
        assert!(key_events.get() > 0, "typing and pressing ctrl-g should dispatch key events");
        assert_eq!(action_count.get(), 1, "ctrl-g should dispatch the registered GPUI action");
        app.update(|cx| cx.shutdown());
    }
}
