use base64::Engine as _;
use image::GenericImageView;
use mkit_inspector::{GpuiInspector, InspectorServer};
use serde_json::{Value, json};

#[cfg(target_os = "macos")]
fn exercise_fixture_on_main_thread() {
    let mut server = InspectorServer::new(GpuiInspector::default());
    let call = |server: &mut InspectorServer<GpuiInspector>, id, name, arguments| {
        server
            .handle(&json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":name,"arguments":arguments,"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{},"io.modelcontextprotocol/clientInfo":{"name":"mkit-inspector-test","version":"0.1.0"}}}}))
            .unwrap()["result"]
            .clone()
    };

    let before_launch = call(&mut server, 0, "entity_state", json!({}));
    assert_eq!(before_launch["isError"], true);
    assert!(
        before_launch["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("no GPUI example is running")
    );
    let unsupported = call(&mut server, 11, "launch", json!({"example":"unknown"}));
    assert_eq!(unsupported["isError"], true);
    let missing_argument = call(&mut server, 12, "launch", json!({}));
    assert_eq!(missing_argument["isError"], true);

    let launched = call(&mut server, 1, "launch", json!({"example":"hello"}));
    let launch_state: Value =
        serde_json::from_str(launched["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(launch_state["example"], "hello");
    assert_eq!(launch_state["status"], "running");
    let duplicate = call(&mut server, 13, "launch", json!({"example":"gallery"}));
    assert_eq!(duplicate["isError"], true);
    assert!(duplicate["content"][0]["text"].as_str().unwrap().contains("already running"));
    let initial_screenshot = call(&mut server, 10, "screenshot", json!({}));
    let initial_png = base64::engine::general_purpose::STANDARD
        .decode(initial_screenshot["content"][0]["data"].as_str().unwrap())
        .unwrap();
    let initial_image = image::load_from_memory(&initial_png).unwrap();
    let initial_background = initial_image.get_pixel(2, 2);
    call(&mut server, 2, "press", json!({"keys":"a"}));
    call(&mut server, 3, "type", json!({"text":"bc"}));
    call(&mut server, 4, "click", json!({"target":"@increment"}));
    let result = call(&mut server, 5, "entity_state", json!({}));
    let state: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert!(state["typed"].as_str().unwrap().contains("abc"));
    assert_eq!(state["last_key"], "c");
    assert_eq!(state["clicks"], 1);

    let screenshot = call(&mut server, 6, "screenshot", json!({}));
    assert_eq!(screenshot["content"][0]["type"], "image");
    let png = screenshot["content"][0]["data"].as_str().unwrap();
    let bytes = base64::engine::general_purpose::STANDARD.decode(png).unwrap();
    let image = image::load_from_memory(&bytes).unwrap();
    assert_eq!(image.dimensions(), (640, 400));
    assert_ne!(
        image.get_pixel(2, 2),
        initial_background,
        "fixture state must change rendered pixels"
    );

    let a11y = call(&mut server, 7, "a11y_tree", json!({}));
    assert_eq!(a11y["isError"], true);
    assert!(a11y["content"][0]["text"].as_str().unwrap().contains("accessibility is inactive"));

    let mut gallery = InspectorServer::new(GpuiInspector::default());
    let launched = call(&mut gallery, 8, "launch", json!({"example":"gallery"}));
    let launch_state: Value =
        serde_json::from_str(launched["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(launch_state["example"], "gallery");
    assert_eq!(launch_state["status"], "running");
    let screenshot = call(&mut gallery, 14, "screenshot", json!({}));
    assert_eq!(screenshot["content"][0]["type"], "image");
    let png = base64::engine::general_purpose::STANDARD
        .decode(screenshot["content"][0]["data"].as_str().unwrap())
        .unwrap();
    assert_eq!(image::load_from_memory(&png).unwrap().dimensions(), (640, 400));
    let clicked = call(&mut gallery, 15, "click", json!({"target":"@increment"}));
    assert_eq!(clicked["isError"], false);
    let result = call(&mut gallery, 9, "entity_state", json!({}));
    let state: Value =
        serde_json::from_str(result["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["heading"], "mkit component gallery");
    assert_eq!(state["clicks"], 1);

    let contexts = call(&mut gallery, 16, "launch", json!({"example":"contexts"}));
    assert_eq!(contexts["isError"], true);
    assert!(contexts["content"][0]["text"].as_str().unwrap().contains("nonvisual"));

    let mut counter = InspectorServer::new(GpuiInspector::default());
    assert_eq!(call(&mut counter, 17, "launch", json!({"example":"counter"}))["isError"], false);
    let initial = call(&mut counter, 18, "entity_state", json!({}));
    let initial: Value =
        serde_json::from_str(initial["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(initial["count"], 0);
    let before = call(&mut counter, 19, "screenshot", json!({}));
    let before = image::load_from_memory(
        &base64::engine::general_purpose::STANDARD
            .decode(before["content"][0]["data"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(call(&mut counter, 20, "click", json!({"target":"@increment"}))["isError"], false);
    let after = call(&mut counter, 21, "screenshot", json!({}));
    let after = image::load_from_memory(
        &base64::engine::general_purpose::STANDARD
            .decode(after["content"][0]["data"].as_str().unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_ne!(before, after, "counter click changes the rendered count");
    let state = call(&mut counter, 22, "entity_state", json!({}));
    let state: Value = serde_json::from_str(state["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["count"], 1);

    let mut entities = InspectorServer::new(GpuiInspector::default());
    assert_eq!(
        call(&mut entities, 23, "launch", json!({"example":"state_entities"}))["isError"],
        false
    );
    let state = call(&mut entities, 24, "entity_state", json!({}));
    let state: Value = serde_json::from_str(state["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["count"], 0);
    call(&mut entities, 25, "click", json!({"target":"@strong-increment"}));
    let state = call(&mut entities, 26, "entity_state", json!({}));
    let state: Value = serde_json::from_str(state["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["count"], 1);
    call(&mut entities, 27, "click", json!({"target":"@weak-increment"}));
    let state = call(&mut entities, 28, "entity_state", json!({}));
    let state: Value = serde_json::from_str(state["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["count"], 2);
    let screenshot = call(&mut entities, 29, "screenshot", json!({}));
    assert_eq!(
        image::load_from_memory(
            &base64::engine::general_purpose::STANDARD
                .decode(screenshot["content"][0]["data"].as_str().unwrap())
                .unwrap()
        )
        .unwrap()
        .dimensions(),
        (640, 400)
    );

    let mut reactivity = InspectorServer::new(GpuiInspector::default());
    assert_eq!(
        call(&mut reactivity, 30, "launch", json!({"example":"reactivity_views"}))["isError"],
        false
    );
    let initial = call(&mut reactivity, 31, "entity_state", json!({}));
    let initial: Value =
        serde_json::from_str(initial["content"][0]["text"].as_str().unwrap()).unwrap();
    let initial_events = initial["event_count"].as_u64().unwrap();
    let screenshot = call(&mut reactivity, 32, "screenshot", json!({}));
    assert_eq!(
        image::load_from_memory(
            &base64::engine::general_purpose::STANDARD
                .decode(screenshot["content"][0]["data"].as_str().unwrap())
                .unwrap()
        )
        .unwrap()
        .dimensions(),
        (640, 400)
    );
    call(&mut reactivity, 33, "click", json!({"target":"@increment"}));
    let state = call(&mut reactivity, 34, "entity_state", json!({}));
    let state: Value = serde_json::from_str(state["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(state["observed_count"], 1);
    assert_eq!(state["event_count"], initial_events + 1);
    assert_eq!(state["last_event_count"], 1);
}

#[cfg(not(target_os = "macos"))]
fn report_headless_platform_limitation() {
    let mut server = InspectorServer::new(GpuiInspector::default());
    let response = server
        .handle(&json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"tools/call",
            "params":{"name":"launch","arguments":{"example":"hello"}}
        }))
        .unwrap();
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("does not provide a headless screenshot renderer")
    );
}

fn main() {
    #[cfg(target_os = "macos")]
    exercise_fixture_on_main_thread();
    #[cfg(not(target_os = "macos"))]
    report_headless_platform_limitation();
}
