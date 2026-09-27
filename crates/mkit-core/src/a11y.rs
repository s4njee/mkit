//! Small, consistent accessibility helpers for GPUI elements.
//!
//! These helpers are thin wrappers around GPUI 0.3.5's `StatefulInteractiveElement`
//! accessibility properties. An element still needs a stable GPUI id and a role
//! to appear in the accessibility tree.

use gpui_pre::{Role, StatefulInteractiveElement, Toggled, accesskit::Live};

/// Announcement politeness for a live accessibility region.
///
/// Polite updates wait for an appropriate pause; assertive updates may
/// interrupt the current announcement. The platform adapter and assistive
/// technology still determine how and when the update is spoken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveRegionPriority {
    /// Announce an update at the next appropriate opportunity.
    Polite,
    /// Announce an urgent update with higher priority.
    Assertive,
}

impl LiveRegionPriority {
    const fn accesskit_live(self) -> Live {
        match self {
            Self::Polite => Live::Polite,
            Self::Assertive => Live::Assertive,
        }
    }
}

fn apply_live_region(node: &mut gpui_pre::accesskit::Node, priority: LiveRegionPriority) {
    node.set_live(priority.accesskit_live());
}

/// Convenience methods for the accessibility properties supported by GPUI.
///
/// This trait is blanket-implemented for GPUI stateful interactive elements,
/// so it adds a shorter and discoverable vocabulary without introducing a
/// second accessibility model.
pub trait AccessibilityExt: StatefulInteractiveElement {
    /// Set the semantic role exposed to assistive technology.
    fn a11y_role(self, role: Role) -> Self {
        StatefulInteractiveElement::role(self, role)
    }

    /// Set the accessible name (label) of this element.
    fn a11y_name(self, name: impl Into<gpui_pre::SharedString>) -> Self {
        StatefulInteractiveElement::aria_label(self, name)
    }

    /// Set supplementary information announced with the accessible name.
    fn a11y_description(self, description: impl Into<gpui_pre::SharedString>) -> Self {
        StatefulInteractiveElement::aria_description(self, description)
    }

    /// Set the string value reported for this element.
    fn a11y_value(self, value: impl Into<gpui_pre::SharedString>) -> Self {
        StatefulInteractiveElement::aria_value(self, value)
    }

    /// Set a numeric value, such as a slider's current position.
    fn a11y_numeric_value(self, value: f64) -> Self {
        StatefulInteractiveElement::aria_numeric_value(self, value)
    }

    /// Set whether this element is selected.
    fn a11y_selected(self, selected: bool) -> Self {
        StatefulInteractiveElement::aria_selected(self, selected)
    }

    /// Set whether this element is expanded.
    fn a11y_expanded(self, expanded: bool) -> Self {
        StatefulInteractiveElement::aria_expanded(self, expanded)
    }

    /// Set the checked or mixed state.
    fn a11y_toggled(self, toggled: Toggled) -> Self {
        StatefulInteractiveElement::aria_toggled(self, toggled)
    }

    /// Mark this element's accessibility node as a live region.
    ///
    /// The element must also have a stable `.id(...)` and a non-generic
    /// `.a11y_role(...)` for GPUI to include it in the accessibility tree.
    /// Update its accessible value with `.a11y_value(...)` when status text
    /// changes, and provide an accessible name that includes the current text
    /// on adapters that use name changes to trigger live-region events.
    /// This sets AccessKit's live-region property on that node through GPUI's
    /// supported synthetic-node builder. It does not guarantee a particular
    /// spoken phrase or timing on every platform/assistive-technology pair.
    /// GPUI stores one synthetic-children callback per element, replacing any
    /// prior callback. Do not combine this with another
    /// `a11y_synthetic_children` call on the same element.
    fn a11y_live_region(self, priority: LiveRegionPriority) -> Self {
        StatefulInteractiveElement::a11y_synthetic_children(self, move |builder| {
            apply_live_region(builder.parent_node(), priority);
        })
    }
}

impl<T: StatefulInteractiveElement> AccessibilityExt for T {}

#[cfg(test)]
mod tests {
    use super::AccessibilityExt;
    use super::LiveRegionPriority;
    use gpui_pre::{
        InteractiveElement, Role, Toggled,
        accesskit::{Live, Node},
        div,
    };

    #[test]
    fn accessibility_helpers_compose_on_a_gpui_element() {
        let _element = div()
            .id("status")
            .a11y_role(Role::Status)
            .a11y_name("Upload status")
            .a11y_description("Current upload progress")
            .a11y_value("Complete")
            .a11y_numeric_value(100.0)
            .a11y_selected(false)
            .a11y_expanded(false)
            .a11y_toggled(Toggled::False);
    }

    #[test]
    fn live_region_helper_maps_priorities_to_accesskit_live_values() {
        for (priority, expected) in [
            (LiveRegionPriority::Polite, Live::Polite),
            (LiveRegionPriority::Assertive, Live::Assertive),
        ] {
            let _element = div()
                .id("upload-status")
                .a11y_role(Role::Status)
                .a11y_name("Upload status")
                .a11y_live_region(priority);

            // AccessKit is the supported live-property boundary. The rendered
            // GPUI test adapter does not activate accessibility, so it cannot
            // expose this node through `Window::debug_a11y_tree_json`.
            let mut node = Node::new(Role::Status);
            super::apply_live_region(&mut node, priority);
            assert_eq!(node.live(), Some(expected));
        }
    }
}
