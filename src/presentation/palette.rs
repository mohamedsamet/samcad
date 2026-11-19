use crate::core::model::Tool;
use crate::core::ports::ToolUseCase;

use super::component::{Component, Emphasis, IconId};

const TITLE: &str = "Tools";

/// Builds the complete palette component tree from the current tool state.
///
/// All decisions — which tools exist, their labels and icons, and which one is
/// highlighted — are made here, so they can be tested without a GUI.
pub fn components(tools: &dyn ToolUseCase) -> Vec<Component> {
    let selected = tools.selected();

    let mut items = Vec::with_capacity(Tool::ALL.len() + 2);
    items.push(Component::Label {
        text: TITLE.to_owned(),
        emphasis: Emphasis::Strong,
    });
    items.push(Component::Separator);

    items.extend(Tool::ALL.into_iter().map(|tool| Component::IconButton {
        tool,
        icon: icon_for(tool),
        tooltip: label_for(tool),
        selected: selected == Some(tool),
        enabled: true,
    }));

    items
}

fn icon_for(tool: Tool) -> IconId {
    match tool {
        Tool::Line => IconId::Line,
        Tool::Circle => IconId::Circle,
        Tool::Rectangle => IconId::Rectangle,
        Tool::Arc => IconId::Arc,
    }
}

fn label_for(tool: Tool) -> &'static str {
    match tool {
        Tool::Line => "Line",
        Tool::Circle => "Circle",
        Tool::Rectangle => "Rectangle",
        Tool::Arc => "Arc",
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    struct StubTools {
        selected: Cell<Option<Tool>>,
    }

    impl ToolUseCase for StubTools {
        fn select(&self, tool: Tool) {
            self.selected.set(Some(tool));
        }

        fn selected(&self) -> Option<Tool> {
            self.selected.get()
        }
    }

    fn buttons(items: &[Component]) -> Vec<&Component> {
        items
            .iter()
            .filter(|item| matches!(item, Component::IconButton { .. }))
            .collect()
    }

    #[test]
    fn builds_four_enabled_buttons_with_no_selection() {
        let tools = StubTools {
            selected: Cell::new(None),
        };

        let items = components(&tools);
        let buttons = buttons(&items);

        assert_eq!(buttons.len(), 4);
        assert!(buttons.iter().all(|item| matches!(
            item,
            Component::IconButton {
                selected: false,
                enabled: true,
                ..
            }
        )));
    }

    #[test]
    fn highlights_exactly_the_selected_tool() {
        let tools = StubTools {
            selected: Cell::new(None),
        };
        tools.select(Tool::Arc);

        let selected: Vec<Tool> = components(&tools)
            .iter()
            .filter_map(|item| match item {
                Component::IconButton {
                    tool,
                    selected: true,
                    ..
                } => Some(*tool),
                _ => None,
            })
            .collect();

        assert_eq!(selected, vec![Tool::Arc]);
    }
}
