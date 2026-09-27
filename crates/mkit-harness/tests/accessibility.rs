use mkit_harness::accessibility::serialize_accessibility_json;

// GPUI's test macro currently hard-codes the canonical `gpui` crate name in
// generated test scaffolding. The workspace depends on the pinned GPUI
// snapshot as `gpui_pre`, so make that same crate available under the expected
// macro expansion name for this integration-test target.
extern crate gpui_pre as gpui;

#[gpui::test]
fn gpui_test_window_does_not_expose_a_captured_accessibility_tree(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;
    use mkit_harness::AccessibilityError;

    let window = cx
        .update(|cx| cx.open_window(Default::default(), |_, cx| cx.new(|_| gpui::Empty)).unwrap());
    cx.run_until_parked();

    window
        .update(cx, |_, window, _| {
            assert!(!window.is_a11y_active());
            assert!(window.debug_a11y_tree_json().is_none());
            assert_eq!(
                mkit_harness::AccessibilitySnapshot::capture(window),
                Err(AccessibilityError::Inactive)
            );
        })
        .unwrap();
}

#[test]
fn snapshot_keeps_semantic_state_and_tree_order_but_drops_gpui_metadata() {
    // Deliberately scramble object key order and use different volatile IDs in
    // the payload. Child array order is the semantic tree order.
    let json = r#"{
        "frame":{"frame_number":41,"rendered_at":"2026-09-24T12:00:00Z"},
        "gpui_focus":"n2",
        "active_descendant_focus":"n3",
        "root":"n1",
        "nodes":{
            "n1":{"accesskit_id":"111","aria":{"role":"Window"},"children":["n2","n3"]},
            "n2":{"element_id":"volatile-element","aria":{"role":"TextField","label":"Search","value":"cats","placeholder":"Find items","on_action":["Focus"],"numeric_value":2,"min_numeric_value":0,"max_numeric_value":5,"numeric_value_step":1}},
            "n3":{"aria":{"role":"Option","label":"Cats","selected":true,"expanded":false,"level":2,"position_in_set":1,"size_of_set":3,"row_index":0,"column_index":1,"row_count":2,"column_count":4,"keyboard_shortcut":"Ctrl+K","access_key":"k","tooltip":"Select cats","role_description":"suggestion"}}
        }
    }"#;

    let snapshot = serialize_accessibility_json(json).unwrap();
    assert_eq!(
        snapshot,
        concat!(
            "Window\n",
            "  TextField [focused] name=\"Search\" value=\"cats\" numeric_value=2 min_numeric_value=0 max_numeric_value=5 numeric_value_step=1 on_action=[\"Focus\"] placeholder=\"Find items\"\n",
            "  Option [active-descendant] name=\"Cats\" selected=true expanded=false level=2 position_in_set=1 size_of_set=3 row_index=0 column_index=1 row_count=2 column_count=4 keyboard_shortcut=\"Ctrl+K\" access_key=\"k\" tooltip=\"Select cats\" role_description=\"suggestion\"",
        )
    );
    assert!(!snapshot.contains("rendered_at"));
    assert!(!snapshot.contains("111"));
    assert!(!snapshot.contains("volatile-element"));
}

#[test]
fn reports_malformed_references_and_cycles_instead_of_silently_truncating() {
    let missing = r#"{"root":"a","nodes":{"a":{"aria":{"role":"Window"},"children":["missing"]}}}"#;
    assert!(
        serialize_accessibility_json(missing)
            .unwrap_err()
            .contains("referenced node \"missing\" is absent")
    );

    let cycle = r#"{"root":"a","nodes":{"a":{"aria":{"role":"Window"},"children":["b"]},"b":{"aria":{"role":"Group"},"children":["a"]}}}"#;
    assert!(serialize_accessibility_json(cycle).unwrap_err().contains("cycle at node \"a\""));
}

#[test]
fn rejects_reused_nodes_and_invalid_child_shapes() {
    let reused = r#"{"root":"a","nodes":{"a":{"aria":{"role":"Window"},"children":["b","b"]},"b":{"aria":{"role":"Button"}}}}"#;
    assert!(
        serialize_accessibility_json(reused).unwrap_err().contains("referenced more than once")
    );

    let invalid_child = r#"{"root":"a","nodes":{"a":{"aria":{"role":"Window"},"children":[4]}}}"#;
    assert!(serialize_accessibility_json(invalid_child).unwrap_err().contains("child reference"));
}
