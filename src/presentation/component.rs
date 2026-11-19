use crate::core::model::Tool;

/// A logical icon identifier. The egui adapter decides which image asset
/// corresponds to each variant; presentation only says which icon is wanted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconId {
    Point,
    Line,
    Circle,
    Rectangle,
    Arc,
}

/// Text styling chosen by presentation, interpreted by the adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emphasis {
    Normal,
    Strong,
}

/// An action attached to a menu item. Presentation decides that the action
/// exists; the adapter decides how to trigger it in the current UI framework.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    NewProject,
    Save,
    Export,
}

/// A single entry in a menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub label: &'static str,
    pub action: MenuAction,
}

/// A window control offered by the floating top bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowButton {
    Minimize,
    Close,
}

/// A ready-to-render element produced by the presentation layer.
///
/// It contains no egui types: the adapter maps it to widgets or painter calls.
/// Canvas shape previews will be added here as drawing features land.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Component {
    IconButton {
        tool: Tool,
        icon: IconId,
        tooltip: &'static str,
        selected: bool,
        enabled: bool,
    },
    Label {
        text: String,
        emphasis: Emphasis,
    },
    Separator,
}
