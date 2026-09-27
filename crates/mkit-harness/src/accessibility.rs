//! Capture GPUI's accessibility debug tree without inventing a fallback tree.

use gpui_pre::Window;

/// A serialized accessibility snapshot captured from a rendered GPUI window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibilitySnapshot {
    text: String,
}

/// Why an accessibility tree could not be captured.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessibilityError {
    /// The platform/test window has not activated accessibility support.
    Inactive,
    /// Accessibility was active but GPUI has not produced a frame yet.
    NoFrame,
    /// GPUI returned a debug tree that the harness could not normalize.
    InvalidTree(String),
}

impl std::fmt::Display for AccessibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inactive => f.write_str("GPUI accessibility is inactive for this window"),
            Self::NoFrame => {
                f.write_str("GPUI has not produced an accessibility tree for this window")
            }
            Self::InvalidTree(error) => {
                write!(f, "GPUI returned an invalid accessibility tree: {error}")
            }
        }
    }
}

impl std::error::Error for AccessibilityError {}

impl AccessibilitySnapshot {
    /// Ask GPUI for the last debug serialization of its actual AccessKit tree.
    ///
    /// GPUI 0.3.5 only captures this tree when accessibility is activated by
    /// the platform adapter. Headless test windows do not activate it, so this
    /// returns [`AccessibilityError::Inactive`] rather than a fabricated tree.
    pub fn capture(window: &Window) -> Result<Self, AccessibilityError> {
        if !window.is_a11y_active() {
            return Err(AccessibilityError::Inactive);
        }
        let json = window.debug_a11y_tree_json().ok_or(AccessibilityError::NoFrame)?;
        let text = serialize_accessibility_json(&json).map_err(AccessibilityError::InvalidTree)?;
        Ok(Self { text })
    }

    /// Stable text with tree order and semantic fields. Volatile IDs and frame
    /// metadata from GPUI's debug JSON are omitted.
    pub fn as_text(&self) -> &str {
        &self.text
    }
}

/// Normalize a GPUI 0.3.5 accessibility debug dump into stable readable text.
pub fn serialize_accessibility_json(json: &str) -> Result<String, String> {
    use gpui_pre::private::serde_json::Value;
    let tree: Value = gpui_pre::private::serde_json::from_str(json).map_err(|e| e.to_string())?;
    let root = tree.get("root").and_then(Value::as_str).ok_or("missing root node")?;
    let nodes = tree.get("nodes").and_then(Value::as_object).ok_or("missing nodes object")?;
    let focus = tree.get("gpui_focus").and_then(Value::as_str);
    let active_descendant = tree.get("active_descendant_focus").and_then(Value::as_str);
    let mut serializer = TreeSerializer {
        focus,
        active_descendant,
        nodes,
        visiting: std::collections::HashSet::new(),
        visited: std::collections::HashSet::new(),
        lines: Vec::new(),
    };
    serializer.walk(root, 0)?;
    Ok(serializer.lines.join("\n"))
}

struct TreeSerializer<'a> {
    focus: Option<&'a str>,
    active_descendant: Option<&'a str>,
    nodes: &'a gpui_pre::private::serde_json::Map<String, gpui_pre::private::serde_json::Value>,
    visiting: std::collections::HashSet<String>,
    visited: std::collections::HashSet<String>,
    lines: Vec<String>,
}

impl TreeSerializer<'_> {
    fn walk(&mut self, id: &str, depth: usize) -> Result<(), String> {
        use gpui_pre::private::serde_json::Value;
        if self.visiting.contains(id) {
            return Err(format!("cycle at node {id:?}"));
        }
        if !self.visited.insert(id.to_owned()) {
            return Err(format!("node {id:?} is referenced more than once"));
        }
        let Some(node) = self.nodes.get(id) else {
            return Err(format!("referenced node {id:?} is absent"));
        };
        self.visiting.insert(id.to_owned());
        let aria = node.get("aria").unwrap_or(&Value::Null);
        let role = aria.get("role").and_then(Value::as_str).unwrap_or("Unknown");
        let name = ["label", "description", "placeholder"]
            .iter()
            .find_map(|key| aria.get(key).and_then(Value::as_str))
            .unwrap_or("");
        let mut line = format!("{}{}", "  ".repeat(depth), role);
        if self.focus == Some(id) {
            line.push_str(" [focused]");
        }
        if self.active_descendant == Some(id) {
            line.push_str(" [active-descendant]");
        }
        if !name.is_empty() {
            line.push_str(&format!(" name={name:?}"));
        }
        for key in [
            "value",
            "selected",
            "expanded",
            "toggled",
            "orientation",
            "numeric_value",
            "min_numeric_value",
            "max_numeric_value",
            "numeric_value_step",
            "level",
            "position_in_set",
            "size_of_set",
            "row_index",
            "column_index",
            "row_count",
            "column_count",
            "on_action",
        ] {
            if let Some(value) = aria.get(key) {
                line.push_str(&format!(" {key}={}", stable_value(value)));
            }
        }
        for key in ["placeholder", "keyboard_shortcut", "access_key", "tooltip", "role_description"]
        {
            if let Some(value) = aria.get(key) {
                line.push_str(&format!(" {key}={}", stable_value(value)));
            }
        }
        self.lines.push(line);
        if let Some(children) = node.get("children") {
            let children = children
                .as_array()
                .ok_or_else(|| format!("children for node {id:?} is not an array"))?;
            for child in children {
                let child = child
                    .as_str()
                    .ok_or_else(|| format!("child reference for node {id:?} is not a string"))?;
                self.walk(child, depth + 1)?;
            }
        }
        self.visiting.remove(id);
        Ok(())
    }
}

fn stable_value(value: &gpui_pre::private::serde_json::Value) -> String {
    match value {
        gpui_pre::private::serde_json::Value::String(value) => format!("{value:?}"),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_headless_tree_as_unsupported_when_accessibility_is_inactive() {
        assert_eq!(
            AccessibilityError::Inactive.to_string(),
            "GPUI accessibility is inactive for this window"
        );
    }

    #[test]
    fn serializes_semantics_in_tree_order_without_volatile_ids_or_frame_data() {
        let input = r#"{"root":"a","gpui_focus":"b","frame":{"frame_number":9,"rendered_at":"volatile"},"nodes":{"a":{"accesskit_id":"123","children":["b"],"aria":{"role":"Window"}},"b":{"accesskit_id":"456","element_id":"button-id","aria":{"role":"Button","label":"Save","selected":true,"on_action":["Click"]}}}}"#;
        let text = serialize_accessibility_json(input).unwrap();
        assert_eq!(
            text,
            "Window\n  Button [focused] name=\"Save\" selected=true on_action=[\"Click\"]"
        );
        assert!(!text.contains("volatile"));
        assert!(!text.contains("button-id"));
    }

    #[test]
    fn rejects_invalid_tree_data_instead_of_reporting_no_frame() {
        assert!(serialize_accessibility_json("not json").is_err());
    }
}
