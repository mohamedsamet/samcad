use crate::core::model::Tool;

/// A logical icon identifier. The egui adapter decides which image asset
/// corresponds to each variant; presentation only says which icon is wanted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconId {
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
