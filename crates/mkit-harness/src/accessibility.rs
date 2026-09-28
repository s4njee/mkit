//! Capture GPUI's accessibility tree without inventing a fallback tree.
//!
//! Two sources are supported. [`AccessibilitySnapshot::capture`] reads GPUI's
//! debug serialization from any window whose platform adapter is active.
//! [`AccessibilityTree::from_tree_update`] normalizes the AccessKit
//! `TreeUpdate` GPUI delivers to a platform adapter; on macOS,
//! `AccessibilitySession` produces those updates headlessly.

use gpui_pre::{Window, accesskit};
use std::collections::{BTreeMap, HashMap, HashSet};

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
    /// GPUI failed while opening, updating, or drawing the capture window.
    Gpui(String),
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
            Self::Gpui(error) => write!(f, "accessibility capture failed: {error}"),
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

/// A semantic value in a normalized accessibility node.
#[derive(Clone, Debug, PartialEq)]
pub enum AccessibilityValue {
    Bool(bool),
    Number(f64),
    Text(String),
    /// An AccessKit enum variant such as `Mixed` or `Horizontal`.
    Token(String),
    /// Relation targets as preorder node indices in the same tree.
    Nodes(Vec<usize>),
}

impl std::fmt::Display for AccessibilityValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::Text(value) => write!(f, "{value:?}"),
            Self::Token(value) => f.write_str(value),
            Self::Nodes(nodes) => {
                let nodes: Vec<String> = nodes.iter().map(|node| format!("#{node}")).collect();
                write!(f, "[{}]", nodes.join(", "))
            }
        }
    }
}

/// One node of an [`AccessibilityTree`], in AccessKit vocabulary.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityNode {
    /// Preorder position from the tree root (the window is `0`).
    pub index: usize,
    pub depth: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    /// AccessKit role name, e.g. `Button` or `CheckBox`.
    pub role: String,
    /// Accessible name: the label, else the text of `labelled_by` targets.
    pub name: Option<String>,
    pub focused: bool,
    /// Present semantic properties and states in a fixed order.
    pub properties: Vec<(&'static str, AccessibilityValue)>,
    /// Relations to other nodes, by preorder index, in a fixed order.
    pub relations: Vec<(&'static str, Vec<usize>)>,
    /// Actions the node supports, in AccessKit enum order.
    pub actions: Vec<String>,
}

impl AccessibilityNode {
    /// Look up an AccessKit-vocabulary property such as `disabled`.
    pub fn property(&self, name: &str) -> Option<&AccessibilityValue> {
        self.properties.iter().find(|(key, _)| *key == name).map(|(_, value)| value)
    }

    /// The WAI-ARIA role this AccessKit role is exposed as. Roles without a
    /// direct ARIA counterpart fall back to the lowercase AccessKit name.
    pub fn aria_role(&self) -> String {
        let role = match self.role.as_str() {
            "Button" | "DefaultButton" => "button",
            "CheckBox" => "checkbox",
            "Switch" => "switch",
            "RadioButton" => "radio",
            "RadioGroup" => "radiogroup",
            "Slider" => "slider",
            "SpinButton" => "spinbutton",
            "TextInput" | "MultilineTextInput" | "PasswordInput" | "EmailInput" | "NumberInput"
            | "PhoneNumberInput" | "UrlInput" | "DateInput" | "DateTimeInput" | "TimeInput" => {
                "textbox"
            }
            "SearchInput" => "searchbox",
            "ComboBox" | "EditableComboBox" => "combobox",
            "ListBox" => "listbox",
            "ListBoxOption" | "MenuListOption" => "option",
            "Menu" | "MenuListPopup" => "menu",
            "MenuBar" => "menubar",
            "MenuItem" => "menuitem",
            "MenuItemCheckBox" => "menuitemcheckbox",
            "MenuItemRadio" => "menuitemradio",
            "Tab" => "tab",
            "TabList" => "tablist",
            "TabPanel" => "tabpanel",
            "Tree" => "tree",
            "TreeItem" => "treeitem",
            "TreeGrid" => "treegrid",
            "Grid" => "grid",
            "GridCell" => "gridcell",
            "Table" => "table",
            "Row" => "row",
            "Cell" => "cell",
            "ColumnHeader" => "columnheader",
            "RowHeader" => "rowheader",
            "Dialog" => "dialog",
            "AlertDialog" => "alertdialog",
            "Alert" => "alert",
            "Status" => "status",
            "Tooltip" => "tooltip",
            "ProgressIndicator" => "progressbar",
            "Meter" => "meter",
            "ScrollBar" => "scrollbar",
            "Splitter" => "separator",
            "Toolbar" => "toolbar",
            "Link" => "link",
            "Group" => "group",
            "Navigation" => "navigation",
            "Image" => "img",
            "Heading" => "heading",
            "List" => "list",
            "ListItem" => "listitem",
            "Window" => "window",
            other => return other.to_lowercase(),
        };
        role.to_owned()
    }

    /// Project this node onto ARIA attribute names for conformance cases.
    ///
    /// Keys are `name`, `description`, `value`, and `aria-*` attributes. A
    /// disabled node also reports the `disabled` alias used by some specs.
    /// Only properties GPUI actually set are reported; absence means "not
    /// exposed", which ARIA treats as the default.
    pub fn aria_properties(&self) -> BTreeMap<String, AccessibilityValue> {
        let mut aria = BTreeMap::new();
        if let Some(name) = &self.name {
            aria.insert("name".to_owned(), AccessibilityValue::Text(name.clone()));
        }
        let aria_role = self.aria_role();
        for (key, value) in &self.properties {
            let mapped = match *key {
                "description" => "description",
                "value" => "value",
                "disabled" => {
                    aria.insert("disabled".to_owned(), value.clone());
                    "aria-disabled"
                }
                "hidden" => "aria-hidden",
                "required" => "aria-required",
                "read_only" => "aria-readonly",
                "busy" => "aria-busy",
                "modal" => "aria-modal",
                "multiselectable" => "aria-multiselectable",
                "selected" => "aria-selected",
                "expanded" => "aria-expanded",
                "toggled" => {
                    let attribute = match aria_role.as_str() {
                        "button" => "aria-pressed",
                        _ => "aria-checked",
                    };
                    let value = match value {
                        AccessibilityValue::Token(token) if token == "True" => {
                            AccessibilityValue::Bool(true)
                        }
                        AccessibilityValue::Token(token) if token == "False" => {
                            AccessibilityValue::Bool(false)
                        }
                        AccessibilityValue::Token(token) => {
                            AccessibilityValue::Text(token.to_lowercase())
                        }
                        other => other.clone(),
                    };
                    aria.insert(attribute.to_owned(), value);
                    continue;
                }
                "numeric_value" => "aria-valuenow",
                "min_numeric_value" => "aria-valuemin",
                "max_numeric_value" => "aria-valuemax",
                "placeholder" => "aria-placeholder",
                "role_description" => "aria-roledescription",
                "keyboard_shortcut" => "aria-keyshortcuts",
                "level" => "aria-level",
                "position_in_set" => "aria-posinset",
                "size_of_set" => "aria-setsize",
                "orientation" | "invalid" | "has_popup" | "current" | "live" => {
                    let attribute = match *key {
                        "orientation" => "aria-orientation",
                        "invalid" => "aria-invalid",
                        "has_popup" => "aria-haspopup",
                        "current" => "aria-current",
                        _ => "aria-live",
                    };
                    let value = match value {
                        AccessibilityValue::Token(token) => {
                            AccessibilityValue::Text(token.to_lowercase())
                        }
                        other => other.clone(),
                    };
                    aria.insert(attribute.to_owned(), value);
                    continue;
                }
                // Table indices, scroll offsets, and other AccessKit-only
                // fields remain in the text snapshot without an ARIA alias.
                _ => continue,
            };
            aria.insert(mapped.to_owned(), value.clone());
        }
        for (key, targets) in &self.relations {
            let attribute = match *key {
                "labelled_by" => "aria-labelledby",
                "described_by" => "aria-describedby",
                "controls" => "aria-controls",
                "details" => "aria-details",
                "owns" => "aria-owns",
                "flow_to" => "aria-flowto",
                "active_descendant" => "aria-activedescendant",
                "error_message" => "aria-errormessage",
                _ => continue,
            };
            aria.insert(attribute.to_owned(), AccessibilityValue::Nodes(targets.clone()));
        }
        aria
    }
}

/// A deterministic, normalized AccessKit tree for snapshot comparison.
///
/// Nodes are listed in preorder from the tree root. GPUI node IDs are replaced
/// by preorder indices, so snapshots do not depend on element-ID hashing.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessibilityTree {
    nodes: Vec<AccessibilityNode>,
}

impl AccessibilityTree {
    /// Normalize a full AccessKit `TreeUpdate` as GPUI sends it each frame.
    ///
    /// Duplicate, missing, unreachable, cyclic, or multiply-parented nodes and
    /// dangling relation targets are errors rather than silently dropped.
    pub fn from_tree_update(update: &accesskit::TreeUpdate) -> Result<Self, String> {
        let root = update.tree.as_ref().ok_or("tree update has no tree root")?.root;
        let mut by_id = HashMap::new();
        for (id, node) in &update.nodes {
            if by_id.insert(*id, node).is_some() {
                return Err(format!("node {id:?} appears more than once in the update"));
            }
        }

        // Preorder walk with explicit stack; any revisit is a cycle or a node
        // with two parents, both of which are malformed trees.
        let mut order: Vec<(accesskit::NodeId, usize, Option<usize>)> = Vec::new();
        let mut index_of: HashMap<accesskit::NodeId, usize> = HashMap::new();
        let mut seen = HashSet::new();
        let mut stack = vec![(root, 0usize, None::<usize>)];
        while let Some((id, depth, parent)) = stack.pop() {
            if !seen.insert(id) {
                return Err(format!("node {id:?} is referenced more than once"));
            }
            let node = by_id.get(&id).ok_or_else(|| format!("referenced node {id:?} is absent"))?;
            let index = order.len();
            index_of.insert(id, index);
            order.push((id, depth, parent));
            for child in node.children().iter().rev() {
                stack.push((*child, depth + 1, Some(index)));
            }
        }
        if order.len() != by_id.len() {
            return Err(format!(
                "{} node(s) are unreachable from the root",
                by_id.len() - order.len()
            ));
        }

        let resolve = |ids: &[accesskit::NodeId], relation: &str| -> Result<Vec<usize>, String> {
            ids.iter()
                .map(|id| {
                    index_of
                        .get(id)
                        .copied()
                        .ok_or_else(|| format!("{relation} target {id:?} is not in the tree"))
                })
                .collect()
        };

        let mut nodes = Vec::with_capacity(order.len());
        for (id, depth, parent) in &order {
            let node = by_id[id];
            let mut properties: Vec<(&'static str, AccessibilityValue)> = Vec::new();
            let mut text = |key: &'static str, value: Option<&str>| {
                if let Some(value) = value {
                    properties.push((key, AccessibilityValue::Text(value.to_owned())));
                }
            };
            text("description", node.description());
            text("value", node.value());
            text("placeholder", node.placeholder());
            text("role_description", node.role_description());
            text("state_description", node.state_description());
            text("tooltip", node.tooltip());
            text("keyboard_shortcut", node.keyboard_shortcut());
            text("access_key", node.access_key());
            for (key, set) in [
                ("disabled", node.is_disabled()),
                ("hidden", node.is_hidden()),
                ("required", node.is_required()),
                ("read_only", node.is_read_only()),
                ("busy", node.is_busy()),
                ("modal", node.is_modal()),
                ("multiselectable", node.is_multiselectable()),
            ] {
                if set {
                    properties.push((key, AccessibilityValue::Bool(true)));
                }
            }
            for (key, value) in [("selected", node.is_selected()), ("expanded", node.is_expanded())]
            {
                if let Some(value) = value {
                    properties.push((key, AccessibilityValue::Bool(value)));
                }
            }
            let token =
                |value: &dyn std::fmt::Debug| AccessibilityValue::Token(format!("{value:?}"));
            if let Some(value) = node.toggled() {
                properties.push(("toggled", token(&value)));
            }
            if let Some(value) = node.orientation() {
                properties.push(("orientation", token(&value)));
            }
            if let Some(value) = node.invalid() {
                properties.push(("invalid", token(&value)));
            }
            if let Some(value) = node.has_popup() {
                properties.push(("has_popup", token(&value)));
            }
            if let Some(value) = node.aria_current() {
                properties.push(("current", token(&value)));
            }
            if let Some(value) = node.live() {
                properties.push(("live", token(&value)));
            }
            for (key, value) in [
                ("numeric_value", node.numeric_value()),
                ("min_numeric_value", node.min_numeric_value()),
                ("max_numeric_value", node.max_numeric_value()),
                ("numeric_value_step", node.numeric_value_step()),
                ("numeric_value_jump", node.numeric_value_jump()),
            ] {
                if let Some(value) = value {
                    properties.push((key, AccessibilityValue::Number(value)));
                }
            }
            for (key, value) in [
                ("level", node.level()),
                ("position_in_set", node.position_in_set()),
                ("size_of_set", node.size_of_set()),
                ("row_index", node.row_index()),
                ("column_index", node.column_index()),
                ("row_count", node.row_count()),
                ("column_count", node.column_count()),
            ] {
                if let Some(value) = value {
                    properties.push((key, AccessibilityValue::Number(value as f64)));
                }
            }

            let mut relations = Vec::new();
            for (key, ids) in [
                ("labelled_by", node.labelled_by()),
                ("described_by", node.described_by()),
                ("controls", node.controls()),
                ("details", node.details()),
                ("owns", node.owns()),
                ("flow_to", node.flow_to()),
                ("radio_group", node.radio_group()),
            ] {
                if !ids.is_empty() {
                    relations.push((key, resolve(ids, key)?));
                }
            }
            for (key, target) in [
                ("active_descendant", node.active_descendant()),
                ("error_message", node.error_message()),
                ("member_of", node.member_of()),
                ("popup_for", node.popup_for()),
            ] {
                if let Some(target) = target {
                    relations.push((key, resolve(&[target], key)?));
                }
            }

            let name = node.label().map(str::to_owned).or_else(|| {
                let parts: Vec<&str> = node
                    .labelled_by()
                    .iter()
                    .filter_map(|target| by_id.get(target))
                    .filter_map(|target| target.label().or_else(|| target.value()))
                    .collect();
                (!parts.is_empty()).then(|| parts.join(" "))
            });

            let mut next = 0u8;
            let actions = std::iter::from_fn(|| {
                let action = accesskit::Action::n(next)?;
                next += 1;
                Some(action)
            })
            .filter(|action| node.supports_action(*action))
            .map(|action| format!("{action:?}"))
            .collect();

            nodes.push(AccessibilityNode {
                index: index_of[id],
                depth: *depth,
                parent: *parent,
                children: resolve(node.children(), "child")?,
                role: format!("{:?}", node.role()),
                name,
                focused: *id == update.focus && *id != root,
                properties,
                relations,
                actions,
            });
        }
        Ok(Self { nodes })
    }

    /// All nodes in preorder; index `0` is the root.
    pub fn nodes(&self) -> &[AccessibilityNode] {
        &self.nodes
    }

    /// The nodes directly below the root, i.e. the top-level semantics of the
    /// rendered view.
    pub fn top_level(&self) -> impl Iterator<Item = &AccessibilityNode> {
        self.nodes
            .first()
            .into_iter()
            .flat_map(|root| root.children.iter())
            .map(|&i| &self.nodes[i])
    }

    /// Stable text with one node per line, indented by depth. A `#N` marker
    /// appears only on nodes that another node references by relation.
    pub fn as_text(&self) -> String {
        let referenced: HashSet<usize> = self
            .nodes
            .iter()
            .flat_map(|node| node.relations.iter().flat_map(|(_, targets)| targets.iter().copied()))
            .collect();
        let mut lines = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let mut line = format!("{}{}", "  ".repeat(node.depth), node.role);
            if referenced.contains(&node.index) {
                line.push_str(&format!(" #{}", node.index));
            }
            if node.focused {
                line.push_str(" [focused]");
            }
            if let Some(name) = &node.name {
                line.push_str(&format!(" name={name:?}"));
            }
            for (key, value) in &node.properties {
                line.push_str(&format!(" {key}={value}"));
            }
            for (key, targets) in &node.relations {
                line.push_str(&format!(" {key}={}", AccessibilityValue::Nodes(targets.clone())));
            }
            if !node.actions.is_empty() {
                line.push_str(&format!(" actions=[{}]", node.actions.join(", ")));
            }
            lines.push(line);
        }
        lines.join("\n")
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

    fn update(nodes: Vec<(u64, accesskit::Node)>, focus: u64) -> accesskit::TreeUpdate {
        accesskit::TreeUpdate {
            nodes: nodes.into_iter().map(|(id, node)| (accesskit::NodeId(id), node)).collect(),
            tree: Some(accesskit::Tree::new(accesskit::NodeId(1))),
            tree_id: accesskit::TreeId::ROOT,
            focus: accesskit::NodeId(focus),
        }
    }

    fn node(role: accesskit::Role, children: &[u64]) -> accesskit::Node {
        let mut node = accesskit::Node::new(role);
        node.set_children(children.iter().copied().map(accesskit::NodeId).collect::<Vec<_>>());
        node
    }

    #[test]
    fn tree_update_normalizes_states_relations_and_aria_projection() {
        let mut label = node(accesskit::Role::Label, &[]);
        label.set_value("Volume");
        let mut slider = node(accesskit::Role::Slider, &[]);
        slider.push_labelled_by(accesskit::NodeId(7));
        slider.set_numeric_value(5.0);
        slider.set_min_numeric_value(0.0);
        slider.set_max_numeric_value(10.0);
        slider.set_disabled();
        slider.add_action(accesskit::Action::Focus);
        let mut check = node(accesskit::Role::CheckBox, &[]);
        check.set_label("Mute");
        check.set_toggled(accesskit::Toggled::Mixed);
        // Node order in the update is irrelevant; the child order is not.
        let tree = AccessibilityTree::from_tree_update(&update(
            vec![
                (9, check),
                (1, node(accesskit::Role::Window, &[7, 8, 9])),
                (8, slider),
                (7, label),
            ],
            9,
        ))
        .unwrap();
        assert_eq!(
            tree.as_text(),
            "Window\n  Label #1 value=\"Volume\"\n  Slider name=\"Volume\" disabled=true numeric_value=5 min_numeric_value=0 max_numeric_value=10 labelled_by=[#1] actions=[Focus]\n  CheckBox [focused] name=\"Mute\" toggled=Mixed"
        );
        let slider = &tree.nodes()[2];
        assert_eq!(slider.aria_role(), "slider");
        let aria = slider.aria_properties();
        assert_eq!(aria["name"], AccessibilityValue::Text("Volume".into()));
        assert_eq!(aria["aria-disabled"], AccessibilityValue::Bool(true));
        assert_eq!(aria["disabled"], AccessibilityValue::Bool(true));
        assert_eq!(aria["aria-valuenow"], AccessibilityValue::Number(5.0));
        assert_eq!(aria["aria-labelledby"], AccessibilityValue::Nodes(vec![1]));
        let check = &tree.nodes()[3];
        assert_eq!(
            check.aria_properties()["aria-checked"],
            AccessibilityValue::Text("mixed".into())
        );
    }

    #[test]
    fn tree_update_rejects_malformed_trees() {
        let window = |children: &[u64]| (1, node(accesskit::Role::Window, children));
        let missing = update(vec![window(&[2])], 1);
        assert!(AccessibilityTree::from_tree_update(&missing).unwrap_err().contains("absent"));
        let reused = update(vec![window(&[2, 2]), (2, node(accesskit::Role::Button, &[]))], 1);
        assert!(
            AccessibilityTree::from_tree_update(&reused)
                .unwrap_err()
                .contains("referenced more than once")
        );
        let orphan = update(vec![window(&[]), (2, node(accesskit::Role::Button, &[]))], 1);
        assert!(AccessibilityTree::from_tree_update(&orphan).unwrap_err().contains("unreachable"));
        let mut dangling = node(accesskit::Role::Button, &[]);
        dangling.set_controls(vec![accesskit::NodeId(40)]);
        let dangling = update(vec![window(&[2]), (2, dangling)], 1);
        assert!(
            AccessibilityTree::from_tree_update(&dangling).unwrap_err().contains("controls target")
        );
    }

    #[test]
    fn rejects_invalid_tree_data_instead_of_reporting_no_frame() {
        assert!(serialize_accessibility_json("not json").is_err());
    }
}
