//! Keyboard navigable tree with owner-driven lazy loading.
extern crate gpui_pre as gpui;
use gpui_pre::{
    Context, EventEmitter, FocusHandle, IntoElement, KeyBinding, PathBuilder, Render, Rgba, Window,
    actions, canvas, div, point, prelude::*, px,
};
use mkit_core::{
    contrast::{composite, relative_luminance},
    theme::Theme,
};
use std::collections::HashSet;

/// Colours derived from theme tokens; see the spec's "Theme tokens used" table.
#[derive(Clone, Copy)]
struct Look {
    border: Rgba,
    active_bg: Rgba,
    active_text: Rgba,
    icon: Rgba,
    active_icon: Rgba,
    /// Pointer-hover fill for rows (shadcn); `None` outlines the row instead.
    hover_bg: Option<Rgba>,
    hover_border: Rgba,
    focus: Rgba,
}
/// Mix `foreground` into `base` by `weight`, like CSS `color-mix(in srgb, ...)`.
fn mix(foreground: Rgba, base: Rgba, weight: f32) -> Rgba {
    composite(Rgba { a: weight * foreground.a, ..foreground }, Rgba { a: 1.0, ..base })
}
fn look(t: &Theme) -> Look {
    let c = t.colors;
    if t.name == "high-contrast" {
        return Look {
            border: c.border,
            active_bg: c.accent,
            active_text: c.accent_text,
            icon: c.text,
            active_icon: c.accent_text,
            hover_bg: None,
            hover_border: c.border,
            focus: c.focus,
        };
    }
    let dark = relative_luminance(c.background) < 0.5;
    // shadcn "accent": text mixed 4% (light) or 12% (dark) into the background.
    let accent = mix(c.text, c.background, if dark { 0.12 } else { 0.04 });
    Look {
        border: if dark { c.text.opacity(0.1) } else { c.border },
        active_bg: accent,
        active_text: c.text,
        icon: c.text_muted,
        active_icon: c.text_muted,
        hover_bg: Some(accent),
        hover_border: c.border,
        focus: c.focus,
    }
}
/// Decorative Lucide icon drawn as a vector stroke so it stays crisp at every scale. Each
/// polyline is a list of points on a 24-unit grid; the stroke is 2 units, Lucide's default.
fn icon(size: f32, lines: &'static [&'static [(f32, f32)]], color: Rgba) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let unit = bounds.size.width / 24.0;
            let origin = bounds.origin;
            let mut path = PathBuilder::stroke(unit * 2.0);
            for line in lines {
                for (i, (x, y)) in line.iter().enumerate() {
                    let at = origin + point(unit * *x, unit * *y);
                    if i == 0 { path.move_to(at) } else { path.line_to(at) }
                }
            }
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .size(px(size))
    .flex_none()
}
/// Lucide `chevron-right`.
const CHEVRON_RIGHT: &[&[(f32, f32)]] = &[&[(9., 6.), (15., 12.), (9., 18.)]];
/// Lucide `chevron-down`.
const CHEVRON_DOWN: &[&[(f32, f32)]] = &[&[(6., 9.), (12., 15.), (18., 9.)]];
/// Lucide `loader`: eight rays around the centre.
const LOADER: &[&[(f32, f32)]] = &[
    &[(12., 2.), (12., 6.)],
    &[(16.2, 7.8), (19.1, 4.9)],
    &[(18., 12.), (22., 12.)],
    &[(16.2, 16.2), (19.1, 19.1)],
    &[(12., 18.), (12., 22.)],
    &[(4.9, 19.1), (7.8, 16.2)],
    &[(2., 12.), (6., 12.)],
    &[(4.9, 4.9), (7.8, 7.8)],
];

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
    focus: Option<FocusHandle>,
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
            focus: None,
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = *cx.global::<Theme>();
        let look = look(&theme);
        // The tree is one tab stop. Tracking the handle here (rather than letting `tab_index`
        // create one in element state) lets render show the active row's focus outline.
        let focus =
            self.focus.get_or_insert_with(|| cx.focus_handle().tab_stop(true).tab_index(0)).clone();
        // `:focus-visible`: the active row outline shows for keyboard focus only.
        let focused = focus.is_focused(window) && window.last_input_was_keyboard();
        let entity = cx.entity();
        let rows = self.visible();
        let hairline = px(theme.borders.hairline);
        let transparent = theme.colors.background.opacity(0.);
        let mut root = div()
            .id(self.label.clone())
            .key_context(KEY_CONTEXT)
            .track_focus(&focus)
            .tab_index(0)
            .role(gpui_pre::accesskit::Role::Tree)
            .aria_label(self.label.clone())
            .flex()
            .flex_col()
            .p(px(theme.spacing.xsmall))
            .bg(theme.colors.surface)
            .border(hairline)
            .border_color(look.border)
            .rounded(px(theme.radii.large))
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
        let icon_size = theme.spacing.large;
        for (id, label, level, expandable, open) in rows {
            let active = self.active.as_deref() == Some(id.as_str());
            let loading = id.ends_with(":loading");
            let row_id = id.clone();
            let row_entity = entity.clone();
            let icon_color = if active { look.active_icon } else { look.icon };
            let glyph = if loading {
                Some(LOADER)
            } else if expandable && open {
                Some(CHEVRON_DOWN)
            } else if expandable {
                Some(CHEVRON_RIGHT)
            } else {
                None
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
                    .items_center()
                    .flex_none()
                    .h(px(theme.controls.small))
                    // Each level indents by one icon width, as the web preview's 16px step.
                    .pl(px(theme.spacing.large * (level as f32 - 1.0)))
                    .pr(px(theme.spacing.small))
                    .gap(px(theme.spacing.small))
                    .rounded(px(theme.radii.small))
                    .border(hairline)
                    .border_color(if focused && active { look.focus } else { transparent })
                    .text_size(px(theme.typography.body))
                    .when(active, |e| e.bg(look.active_bg).text_color(look.active_text))
                    .when(!active, |e| {
                        e.text_color(if loading {
                            theme.colors.text_muted
                        } else {
                            theme.colors.text
                        })
                    })
                    .when(!active && !loading, |e| {
                        e.hover(move |style| match look.hover_bg {
                            Some(fill) => style.bg(fill),
                            None => style.border_color(look.hover_border),
                        })
                    })
                    .child(
                        // The disclosure target spans the row's leading padding and the chevron,
                        // so the target is `spacing.small` plus the icon (24px) wide.
                        div()
                            .id(format!("{id}:disclosure"))
                            .flex_none()
                            .h_full()
                            .pl(px(theme.spacing.small))
                            .flex()
                            .items_center()
                            .w(px(theme.spacing.small + icon_size))
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
                            .when_some(glyph, |disclosure, glyph| {
                                disclosure.child(icon(icon_size, glyph, icon_color))
                            }),
                    )
                    .child(div().flex_1().truncate().child(label)),
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
        let second = point(px(15.0), px(48.0));
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

        // The second row starts 37 px down; its disclosure spans the leading 24 px of the row.
        visual.simulate_click(point(px(12.0), px(48.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| {
            assert_eq!(tree.active(), Some("active"));
            assert!(tree.is_expanded("branch"));
        });
        assert_eq!(*events.borrow(), vec!["expand:branch:true"]);

        // The label area activates the row independently of its disclosure target.
        visual.simulate_click(point(px(48.0), px(48.0)), Modifiers::default());
        tree.read_with(visual, |tree, _| assert_eq!(tree.active(), Some("branch")));
        visual.simulate_click(point(px(12.0), px(48.0)), Modifiers::default());
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

        visual.simulate_click(point(px(12.0), px(48.0)), Modifiers::default());
        visual.simulate_click(point(px(12.0), px(48.0)), Modifiers::default());
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
