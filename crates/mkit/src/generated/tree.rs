//! Keyboard navigable tree with owner-driven lazy loading.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, IntoElement, KeyBinding, Render, Window, actions, div, prelude::*, px,
};
use mkit_core::theme::Theme;
use std::collections::HashSet;

pub const KEY_CONTEXT: &str = "MkitTree";
actions!(tree, [Next, Previous, Expand, Collapse, First, Last]);
pub fn default_key_bindings() -> [KeyBinding; 6] {
    [
        KeyBinding::new("down", Next, Some(KEY_CONTEXT)),
        KeyBinding::new("up", Previous, Some(KEY_CONTEXT)),
        KeyBinding::new("right", Expand, Some(KEY_CONTEXT)),
        KeyBinding::new("left", Collapse, Some(KEY_CONTEXT)),
        KeyBinding::new("home", First, Some(KEY_CONTEXT)),
        KeyBinding::new("end", Last, Some(KEY_CONTEXT)),
    ]
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub children: Option<Vec<TreeNode>>,
}
impl TreeNode {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), children: Some(Vec::new()) }
    }
    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self { id: id.into(), label: label.into(), children: Some(children) }
    }
    pub fn lazy(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self { id: id.into(), label: label.into(), children: None }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveChanged(pub Option<String>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpansionChanged {
    pub id: String,
    pub expanded: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadChildrenRequested(pub String);
impl EventEmitter<ActiveChanged> for Tree {}
impl EventEmitter<ExpansionChanged> for Tree {}
impl EventEmitter<LoadChildrenRequested> for Tree {}

pub struct Tree {
    label: String,
    roots: Vec<TreeNode>,
    expanded: HashSet<String>,
    active: Option<String>,
    controlled: bool,
    loading: HashSet<String>,
}
impl Tree {
    pub fn new(label: impl Into<String>, roots: Vec<TreeNode>) -> Self {
        let active = roots.first().map(|node| node.id.clone());
        Self {
            label: label.into(),
            roots,
            expanded: HashSet::new(),
            active,
            controlled: false,
            loading: HashSet::new(),
        }
    }
    pub fn controlled(
        label: impl Into<String>,
        roots: Vec<TreeNode>,
        expanded: HashSet<String>,
    ) -> Self {
        let mut t = Self::new(label, roots);
        t.expanded = expanded;
        t.controlled = true;
        t
    }
    pub fn active(&self) -> Option<&str> {
        self.active.as_deref()
    }
    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.contains(id)
    }
    pub fn set_expanded(&mut self, ids: HashSet<String>, cx: &mut Context<Self>) {
        self.expanded = ids;
        cx.notify();
    }
    pub fn set_children(
        &mut self,
        id: &str,
        children: Vec<TreeNode>,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(node) = find_mut(&mut self.roots, id) else {
            return false;
        };
        node.children = Some(children);
        self.loading.remove(id);
        cx.notify();
        true
    }
    fn visible(&self) -> Vec<(String, String, usize, bool, bool)> {
        let mut rows = Vec::new();
        flatten(&self.roots, 1, &self.expanded, &self.loading, &mut rows);
        rows
    }
    fn set_active_index(&mut self, index: usize, cx: &mut Context<Self>) {
        let rows = self.visible();
        if let Some(id) = rows.get(index).map(|r| r.0.clone()) {
            self.set_active_id(id, cx);
        }
    }
    fn set_active_id(&mut self, id: String, cx: &mut Context<Self>) {
        if self.active.as_deref() != Some(id.as_str()) {
            self.active = Some(id);
            cx.emit(ActiveChanged(self.active.clone()));
            cx.notify();
        }
    }
    fn move_active(&mut self, step: isize, cx: &mut Context<Self>) {
        let rows = self.visible();
        if rows.is_empty() {
            return;
        }
        let Some(i) = self.active.as_ref().and_then(|id| rows.iter().position(|r| &r.0 == id))
        else {
            self.set_active_index(if step >= 0 { 0 } else { rows.len() - 1 }, cx);
            return;
        };
        self.set_active_index((i as isize + step).clamp(0, rows.len() as isize - 1) as usize, cx);
    }
    fn expand_active(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        let Some(node) = find(&self.roots, &id) else { return };
        if self.expanded.contains(&id) {
            if let Some(child) = node.children.as_ref().and_then(|children| children.first()) {
                self.set_active_id(child.id.clone(), cx);
            }
            return;
        }
        self.request_expand(&id, cx);
    }
    fn request_expand(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(node) = find(&self.roots, id) else { return };
        match &node.children {
            Some(children) if !children.is_empty() => {
                if !self.controlled {
                    self.expanded.insert(id.to_owned());
                }
                cx.emit(ExpansionChanged { id: id.to_owned(), expanded: true });
                cx.notify();
            }
            None => {
                if !self.controlled {
                    self.expanded.insert(id.to_owned());
                }
                cx.emit(ExpansionChanged { id: id.to_owned(), expanded: true });
                if self.loading.insert(id.to_owned()) {
                    cx.emit(LoadChildrenRequested(id.to_owned()));
                }
                cx.notify();
            }
            _ => {}
        }
    }
    fn toggle_disclosure(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.expanded.contains(id) {
            if !self.controlled {
                self.expanded.remove(id);
            }
            cx.emit(ExpansionChanged { id: id.to_owned(), expanded: false });
            cx.notify();
        } else {
            self.request_expand(id, cx);
        }
    }
    fn collapse_active(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.active.clone() else { return };
        if self.expanded.contains(&id) {
            if !self.controlled {
                self.expanded.remove(&id);
            }
            cx.emit(ExpansionChanged { id, expanded: false });
            cx.notify();
            return;
        }
        let rows = self.visible();
        if let Some((parent, _)) = parent_id(&self.roots, &id).zip(rows.iter().find(|r| r.0 == id))
        {
            self.set_active_id(parent, cx);
        }
    }
    fn click_row(&mut self, id: &str, cx: &mut Context<Self>) {
        if find(&self.roots, id).is_some() {
            self.set_active_id(id.to_owned(), cx);
        }
    }
}
fn find<'a>(nodes: &'a [TreeNode], id: &str) -> Option<&'a TreeNode> {
    for n in nodes {
        if n.id == id {
            return Some(n);
        }
        if let Some(v) = &n.children
            && let Some(found) = find(v, id)
        {
            return Some(found);
        }
    }
    None
}
fn find_mut<'a>(nodes: &'a mut [TreeNode], id: &str) -> Option<&'a mut TreeNode> {
    for n in nodes {
        if n.id == id {
            return Some(n);
        }
        if let Some(v) = &mut n.children
            && let Some(found) = find_mut(v, id)
        {
            return Some(found);
        }
    }
    None
}
fn flatten(
    nodes: &[TreeNode],
    level: usize,
    expanded: &HashSet<String>,
    loading: &HashSet<String>,
    out: &mut Vec<(String, String, usize, bool, bool)>,
) {
    for n in nodes {
        let expandable = n.children.as_ref().is_none_or(|c| !c.is_empty());
        let open = expanded.contains(&n.id);
        out.push((n.id.clone(), n.label.clone(), level, expandable, open));
        if open && loading.contains(&n.id) {
            out.push((format!("{}:loading", n.id), "Loading…".into(), level + 1, false, false));
        } else if open && let Some(c) = &n.children {
            flatten(c, level + 1, expanded, loading, out)
        }
    }
}
fn parent_id(nodes: &[TreeNode], id: &str) -> Option<String> {
    for n in nodes {
        if n.children.as_ref().is_some_and(|c| c.iter().any(|x| x.id == id)) {
            return Some(n.id.clone());
        }
        if let Some(c) = &n.children
            && let Some(p) = parent_id(c, id)
        {
            return Some(p);
        }
    }
    None
}
impl Render for Tree {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let entity = cx.entity();
        let rows = self.visible();
        let mut root = div()
            .id(self.label.clone())
            .key_context(KEY_CONTEXT)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::Tree)
            .aria_label(self.label.clone())
            .bg(theme.colors.surface)
            .border(px(theme.borders.hairline))
            .border_color(theme.colors.border)
            .on_action(cx.listener(|s, _: &Next, _, cx| s.move_active(1, cx)))
            .on_action(cx.listener(|s, _: &Previous, _, cx| s.move_active(-1, cx)))
            .on_action(cx.listener(|s, _: &Expand, _, cx| s.expand_active(cx)))
            .on_action(cx.listener(|s, _: &Collapse, _, cx| s.collapse_active(cx)))
            .on_action(cx.listener(|s, _: &First, _, cx| s.set_active_index(0, cx)))
            .on_action(cx.listener(|s, _: &Last, _, cx| {
                let n = s.visible().len();
                if n > 0 {
                    s.set_active_index(n - 1, cx)
                }
            }));
        for (id, label, level, expandable, open) in rows {
            let active = self.active.as_deref() == Some(id.as_str());
            let loading = id.ends_with(":loading");
            let row_id = id.clone();
            let row_entity = entity.clone();
            let prefix = if loading {
                "◌"
            } else if expandable && open {
                "▾"
            } else if expandable {
                "▸"
            } else {
                ""
            };
            root = root.child(
                div()
                    .id(id.clone())
                    .role(gpui_pre::accesskit::Role::TreeItem)
                    .aria_label(label.clone())
                    .aria_level(level)
                    .when(active, |e| e.aria_active_descendant())
                    .when(expandable, |e| e.aria_expanded(open))
                    .on_click(move |_, _, cx| {
                        row_entity.update(cx, |tree, cx| tree.click_row(&row_id, cx));
                    })
                    .flex()
                    .flex_row()
                    .pl(px(theme.spacing.small))
                    .ml(px(theme.spacing.medium * (level as f32 - 1.0)))
                    .bg(if active { theme.colors.accent } else { theme.colors.surface })
                    .text_color(if active { theme.colors.accent_text } else { theme.colors.text })
                    .child(
                        div()
                            .id(format!("{id}:disclosure"))
                            .w(px(theme.spacing.medium * 2.0))
                            .flex_shrink_0()
                            .text_center()
                            .when(expandable && !loading, |disclosure| {
                                let node_id = id.clone();
                                let entity = entity.clone();
                                disclosure.on_click(move |_, _, cx| {
                                    cx.stop_propagation();
                                    entity.update(cx, |tree, cx| {
                                        tree.toggle_disclosure(&node_id, cx)
                                    });
                                })
                            })
                            .child(prefix),
                    )
                    .child(div().child(label)),
            );
        }
        root
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gpui_pre::{Modifiers, TestAppContext, point};
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn flatten_hides_collapsed_descendants_and_shows_expanded() {
        let roots = vec![TreeNode::branch("a", "A", vec![TreeNode::leaf("b", "B")])];
        let mut tree = Tree::new("files", roots);
        assert_eq!(tree.visible().len(), 1);
        tree.expanded.insert("a".into());
        assert_eq!(tree.visible().len(), 2);
    }
    #[test]
    fn lazy_nodes_are_expandable() {
        assert!(TreeNode::lazy("x", "X").children.is_none());
    }

    #[gpui_pre::test]
    fn pointer_and_arrows_navigate_and_expand_tree(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (tree, visual) = cx.add_window_view(|_, _| {
            Tree::new(
                "Files",
                vec![
                    TreeNode::branch("a", "Alpha", vec![TreeNode::leaf("child", "Child")]),
                    TreeNode::leaf("b", "Beta"),
                ],
            )
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        let second = point(px(15.0), px(30.0));
        visual.simulate_click(second, Modifiers::default());
        assert_eq!(
            tree.read_with(visual, |tree, _| tree.active().map(str::to_owned)),
            Some("b".into())
        );

        visual.simulate_keystrokes("home right");
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("a"));
            assert!(tree.is_expanded("a"));
        });
        visual.simulate_keystrokes("right");
        assert_eq!(
            tree.read_with(visual, |tree, _| tree.active().map(str::to_owned)),
            Some("child".into())
        );
        visual.simulate_keystrokes("left left");
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("a"));
            assert!(!tree.is_expanded("a"));
        });
    }

    #[gpui_pre::test]
    fn disclosure_click_expands_without_activating_row(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (tree, visual) = cx.add_window_view(|_, _| {
            Tree::new(
                "Files",
                vec![
                    TreeNode::leaf("active", "Active"),
                    TreeNode::branch("branch", "Branch", vec![TreeNode::leaf("child", "Child")]),
                ],
            )
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&tree, move |_, event: &ActiveChanged, _| {
                log.borrow_mut().push(format!("active:{:?}", event.0));
            })
        });
        let log = events.clone();
        let _expansion_subscription = visual.update(|_, app| {
            app.subscribe(&tree, move |_, event: &ExpansionChanged, _| {
                log.borrow_mut().push(format!("expand:{}:{}", event.id, event.expanded));
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        // The second row's disclosure occupies the leading two spacing.medium units.
        visual.simulate_click(point(px(12.0), px(30.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("active"));
            assert!(tree.is_expanded("branch"));
        });
        assert_eq!(*events.borrow(), vec!["expand:branch:true"]);

        // The label area activates the row independently of its disclosure target.
        visual.simulate_click(point(px(48.0), px(30.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| assert_eq!(tree.active(), Some("branch")));
        visual.simulate_click(point(px(12.0), px(30.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("branch"));
            assert!(!tree.is_expanded("branch"));
        });
        assert_eq!(
            *events.borrow(),
            vec!["expand:branch:true", "active:Some(\"branch\")", "expand:branch:false"]
        );
    }

    #[gpui_pre::test]
    fn controlled_lazy_disclosure_waits_for_owner_without_row_activation(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        let (tree, visual) = cx.add_window_view(|_, _| {
            Tree::controlled(
                "Files",
                vec![TreeNode::leaf("active", "Active"), TreeNode::lazy("lazy", "Lazy")],
                HashSet::new(),
            )
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let load_events = events.clone();
        let _load_subscription = visual.update(|_, app| {
            app.subscribe(&tree, move |_, event: &LoadChildrenRequested, _| {
                load_events.borrow_mut().push(format!("load:{}", event.0));
            })
        });
        let expansion_events = events.clone();
        let _expansion_subscription = visual.update(|_, app| {
            app.subscribe(&tree, move |_, event: &ExpansionChanged, _| {
                expansion_events
                    .borrow_mut()
                    .push(format!("expand:{}:{}", event.id, event.expanded));
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));

        visual.simulate_click(point(px(12.0), px(30.0)), Modifiers::default());
        visual.simulate_click(point(px(12.0), px(30.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("active"));
            assert!(!tree.is_expanded("lazy"));
            assert_eq!(tree.visible().len(), 2, "controlled mode waits for owner apply");
        });
        assert_eq!(*events.borrow(), vec!["expand:lazy:true", "load:lazy", "expand:lazy:true"]);

        tree.update(visual, |tree, cx| tree.set_expanded(HashSet::from(["lazy".into()]), cx));
        assert_eq!(tree.read_with(visual, |tree, _| tree.visible().len()), 3);
        tree.update(visual, |tree, cx| {
            assert!(tree.set_children("lazy", vec![TreeNode::leaf("child", "Child")], cx));
        });
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("active"));
            assert!(tree.loading.is_empty());
            assert_eq!(tree.visible().len(), 3);
        });
        assert_eq!(events.borrow().iter().filter(|event| event.as_str() == "load:lazy").count(), 1);
    }

    #[gpui_pre::test]
    fn controlled_lazy_branch_requests_children_once_and_waits_for_owner(cx: &mut TestAppContext) {
        cx.update(mkit_core::theme::set_light_theme);
        cx.update(|app| app.bind_keys(default_key_bindings()));
        let (tree, visual) = cx.add_window_view(|_, _| {
            Tree::controlled("Files", vec![TreeNode::lazy("lazy", "Lazy")], HashSet::new())
        });
        let loads = Rc::new(RefCell::new(Vec::new()));
        let log = loads.clone();
        let _subscription = visual.update(|_, app| {
            app.subscribe(&tree, move |_, event: &LoadChildrenRequested, _| {
                log.borrow_mut().push(event.0.clone());
            })
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_click(point(px(40.0), px(10.0)), Modifiers::default());
        visual.simulate_keystrokes("right right");
        tree.read_with(visual, |tree, _| {
            assert!(!tree.is_expanded("lazy"));
            assert_eq!(tree.visible().len(), 1, "controlled loading waits for expansion");
        });
        assert_eq!(*loads.borrow(), vec!["lazy".to_owned()]);

        tree.update(visual, |tree, cx| tree.set_expanded(HashSet::from(["lazy".into()]), cx));
        assert_eq!(tree.read_with(visual, |tree, _| tree.visible().len()), 2);
        tree.update(visual, |tree, cx| {
            assert!(tree.set_children("lazy", vec![TreeNode::leaf("child", "Child")], cx));
        });
        tree.read_with(visual, |tree, _| {
            assert!(tree.loading.is_empty());
            assert_eq!(tree.visible().len(), 2);
        });
    }
}
