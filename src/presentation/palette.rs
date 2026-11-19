use crate::core::model::Tool;

use super::component::{Component, IconId};

/// Builds the palette components from the current selection.
///
/// All decisions — which tools exist, their labels and icons, and which one is
/// highlighted — are made here, so they can be tested without a GUI. The
/// palette has no title; only the icon buttons are shown.
///
/// Taking the selection as plain data keeps presentation independent of the
/// application's driving ports; the adapter passes `tools.selected()`.
pub fn components(selected: Option<Tool>) -> impl Iterator<Item = Component> {
    let buttons = Tool::ALL
        .into_iter()
        .map(move |tool| Component::IconButton {
            tool,
            icon: icon_for(tool),
            tooltip: label_for(tool),
            selected: selected == Some(tool),
            enabled: true,
        });

    std::iter::once(Component::Separator).chain(buttons)
}

fn icon_for(tool: Tool) -> IconId {
    match tool {
        Tool::Point => IconId::Point,
        Tool::Line => IconId::Line,
        Tool::Circle => IconId::Circle,
        Tool::Rectangle => IconId::Rectangle,
        Tool::Arc => IconId::Arc,
    }
}

fn label_for(tool: Tool) -> &'static str {
    match tool {
        Tool::Point => "Point",
        Tool::Line => "Line",
        Tool::Circle => "Circle",
        Tool::Rectangle => "Rectangle",
        Tool::Arc => "Arc",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buttons(items: impl Iterator<Item = Component>) -> Vec<Component> {
        items
            .filter(|item| matches!(item, Component::IconButton { .. }))
            .collect()
    }

    #[test]
    fn builds_one_enabled_button_per_tool_with_no_selection() {
        let buttons = buttons(components(None));

        assert_eq!(buttons.len(), Tool::ALL.len());
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
        let selected: Vec<Tool> = components(Some(Tool::Arc))
            .filter_map(|item| match item {
                Component::IconButton {
                    tool,
                    selected: true,
                    ..
                } => Some(tool),
                _ => None,
            })
            .collect();

        assert_eq!(selected, vec![Tool::Arc]);
    }
}
