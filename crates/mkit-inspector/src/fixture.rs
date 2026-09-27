use base64::Engine as _;
use gpui_pre::{App, InputEvent, MouseButton, MouseDownEvent, MouseUpEvent, point, px, size};
use image::ImageFormat;
use mkit_example_hello::InspectorFixture;
use mkit_harness::AccessibilitySnapshot;
use serde_json::{Value, json};
use std::io::Cursor;

type Session<V> = mkit_harness::HeadlessSession<V>;

enum RunningSession {
    Hello(Session<InspectorFixture>),
    Counter(Session<mkit_example_counter::Counter>),
    StateEntities(Session<mkit_example_state_entities::CounterWindow>),
    ReactivityViews(Session<mkit_example_reactivity_views::ReactivityDemo>),
}

#[derive(Default)]
pub struct GpuiInspector {
    session: Option<RunningSession>,
    example: Option<&'static str>,
}

impl GpuiInspector {
    fn capture(&mut self) -> Result<image::RgbaImage, String> {
        match self
            .session
            .as_mut()
            .ok_or("no GPUI example is running; call launch with a supported example first")?
        {
            RunningSession::Hello(session) => session.capture(),
            RunningSession::Counter(session) => session.capture(),
            RunningSession::StateEntities(session) => session.capture(),
            RunningSession::ReactivityViews(session) => session.capture(),
        }
        .map_err(|e| e.to_string())
    }

    fn update_window<R>(
        &mut self,
        update: impl FnOnce(&mut gpui_pre::Window, &mut App) -> R,
    ) -> Result<R, String> {
        match self
            .session
            .as_mut()
            .ok_or("no GPUI example is running; call launch with a supported example first")?
        {
            RunningSession::Hello(session) => session.update(|_, window, cx| update(window, cx)),
            RunningSession::Counter(session) => session.update(|_, window, cx| update(window, cx)),
            RunningSession::StateEntities(session) => {
                session.update(|_, window, cx| update(window, cx))
            }
            RunningSession::ReactivityViews(session) => {
                session.update(|_, window, cx| update(window, cx))
            }
        }
        .map_err(|e| e.to_string())
    }

    fn a11y_tree(&mut self) -> Result<String, String> {
        match self
            .session
            .as_mut()
            .ok_or("no GPUI example is running; call launch with a supported example first")?
        {
            RunningSession::Hello(session) => session.update(|_, window, _| {
                AccessibilitySnapshot::capture(window)
                    .map(|snapshot| snapshot.as_text().to_string())
                    .map_err(|e| e.to_string())
            }),
            RunningSession::Counter(session) => session.update(|_, window, _| {
                AccessibilitySnapshot::capture(window)
                    .map(|snapshot| snapshot.as_text().to_string())
                    .map_err(|e| e.to_string())
            }),
            RunningSession::StateEntities(session) => session.update(|_, window, _| {
                AccessibilitySnapshot::capture(window)
                    .map(|snapshot| snapshot.as_text().to_string())
                    .map_err(|e| e.to_string())
            }),
            RunningSession::ReactivityViews(session) => session.update(|_, window, _| {
                AccessibilitySnapshot::capture(window)
                    .map(|snapshot| snapshot.as_text().to_string())
                    .map_err(|e| e.to_string())
            }),
        }
        .map_err(|e| e.to_string())?
    }
}

fn make_session<V: gpui_pre::Render + 'static>(
    view: V,
    init: impl FnOnce(&mut App),
) -> Result<Session<V>, String> {
    Session::new(view, size(px(640.), px(400.)), 1.0, init).map_err(|e| e.to_string())
}

impl super::InspectorBackend for GpuiInspector {
    fn call(&mut self, tool: &str, args: &Value) -> Result<Value, String> {
        match tool {
            "launch" => {
                let example = args.get("example").and_then(Value::as_str)
                    .ok_or("required argument `example` must be a string")?;
                if example == "contexts" {
                    return Err("example `contexts` is nonvisual and cannot be launched by the GPUI inspector".into());
                }
                if self.session.is_some() {
                    return Err("an example is already running; this process currently supports one window".into());
                }
                let session = match example {
                    "hello" => RunningSession::Hello(make_session(InspectorFixture::hello(), |_| {})?),
                    "gallery" => RunningSession::Hello(make_session(mkit_gallery::inspector_fixture(), |_| {})?),
                    "counter" => RunningSession::Counter(make_session(mkit_example_counter::Counter::default(), gpui_kit::init)?),
                    "state_entities" => RunningSession::StateEntities(make_session(
                        mkit_example_state_entities::CounterWindow::preview(),
                        |cx| { gpui_kit::init(cx); cx.set_global(mkit_example_state_entities::CounterSettings::default()); },
                    )?),
                    "reactivity_views" => RunningSession::ReactivityViews(make_session(
                        mkit_example_reactivity_views::ReactivityDemo::default(), gpui_kit::init,
                    )?),
                    other => return Err(format!("example `{other}` is not supported by this inspector build; supported examples: `hello`, `gallery`, `counter`, `state_entities`, `reactivity_views` (the nonvisual `contexts` example cannot be inspected)")),
                };
                self.session = Some(session);
                self.example = Some(match example { "gallery" => "gallery", "hello" => "hello", "counter" => "counter", "state_entities" => "state_entities", _ => "reactivity_views" });
                Ok(json!({"example":example,"status":"running"}))
            }
            "screenshot" => {
                let image = self.capture()?;
                let mut png = Cursor::new(Vec::new());
                image.write_to(&mut png, ImageFormat::Png).map_err(|e| format!("PNG encoding failed: {e}"))?;
                Ok(json!({"_mcpImage":base64::engine::general_purpose::STANDARD.encode(png.into_inner())}))
            }
            "press" => {
                let keys = args.get("keys").and_then(Value::as_str).ok_or("required argument `keys` must be a string")?;
                for key in keys.split_whitespace() {
                    self.update_window(|window, cx| {
                        let keystroke = gpui_pre::Keystroke::parse(key).map_err(|e| e.to_string())?;
                        window.dispatch_keystroke(keystroke, cx);
                        Ok::<_, String>(())
                    })??;
                }
                Ok(json!({"dispatched":keys}))
            }
            "type" => {
                let text = args.get("text").and_then(Value::as_str).ok_or("required argument `text` must be a string")?;
                for ch in text.chars() {
                    let key = ch.to_string();
                    self.update_window(|window, cx| {
                        let keystroke = gpui_pre::Keystroke::parse(&key).map_err(|e| e.to_string())?;
                        window.dispatch_keystroke(keystroke, cx);
                        Ok::<_, String>(())
                    })??;
                }
                Ok(json!({"typed":text}))
            }
            "click" => {
                let (x, y) = if let Some(target) = args.get("target").and_then(Value::as_str) {
                    match (self.example, target.trim_start_matches('@')) {
                        (_, "increment") if self.example == Some("hello") || self.example == Some("gallery") => (30.0, 90.0),
                        (Some("counter"), "increment") => (320.0, 220.0),
                        (Some("state_entities"), "strong-increment") => (320.0, 210.0),
                        (Some("state_entities"), "weak-increment") => (320.0, 265.0),
                        (Some("reactivity_views"), "increment") => (110.0, 257.0),
                        (_, name) => return Err(format!("unknown target `@{name}` for the running example")),
                    }
                } else {
                    (
                        args.get("x").and_then(Value::as_f64).ok_or("provide `target` or numeric `x` and `y`")? as f32,
                        args.get("y").and_then(Value::as_f64).ok_or("provide `target` or numeric `x` and `y`")? as f32,
                    )
                };
                if !x.is_finite() || !y.is_finite() || x < 0. || y < 0. || x > 640. || y > 400. {
                    return Err("click coordinates must be finite and inside the 640 by 400 inspector window".into());
                }
                let position = point(px(x), px(y));
                self.update_window(|window, cx| {
                    window.dispatch_event(MouseDownEvent { position, button: MouseButton::Left, modifiers: Default::default(), click_count: 1, first_mouse: false }.to_platform_input(), cx);
                    window.dispatch_event(MouseUpEvent { position, button: MouseButton::Left, modifiers: Default::default(), click_count: 1 }.to_platform_input(), cx);
                })?;
                Ok(json!({"clicked":{"x":x,"y":y}}))
            }
            "a11y_tree" => self.a11y_tree().map(|tree| json!({"tree":tree})),
            "entity_state" => match self.session.as_mut().ok_or("no GPUI example is running; call launch with a supported example first")? {
                RunningSession::Hello(session) => session.update(|root, _, cx| root.update(cx, |fixture, _| json!({"clicks":fixture.clicks,"typed":fixture.typed,"last_key":fixture.last_key,"heading":fixture.heading}))).map_err(|e| e.to_string()),
                RunningSession::Counter(session) => session.update(|root, _, cx| root.update(cx, |view, _| json!({"example":"counter","count":view.inspector_count()}))).map_err(|e| e.to_string()),
                RunningSession::StateEntities(session) => session.update(|root, _, cx| root.update(cx, |view, cx| json!({"example":"state_entities","count":view.inspector_count(cx)}))).map_err(|e| e.to_string()),
                RunningSession::ReactivityViews(session) => session.update(|root, _, cx| root.update(cx, |view, _| {
                    let (observed_count, notification_count, event_count, last_event_count) = view.inspector_snapshot();
                    json!({"example":"reactivity_views","observed_count":observed_count,"notification_count":notification_count,"event_count":event_count,"last_event_count":last_event_count})
                })).map_err(|e| e.to_string()),
            },
            _ => Err(format!("unknown tool `{tool}`")),
        }
    }
}
