use ratatui::style::{palette::tailwind, Color, Style};

pub const SHOW_HELP: &str = " <type ? to show help> ";
pub const TITLE_PROGRAM: &str = " Terminal Attribute Table ";
pub const TITLE_DATASET_INFO: &str = " Dataset ";
pub const TITLE_LAYER_INFO: &str = " Layer Information ";
pub const TITLE_GDAL_LOG: &str = " GDAL Log ";
pub const TITLE_LAYER_LIST: &str = " Layers ";

/// Common colors
pub struct TatPalette {
    pub default_fg: Color,
    pub highlighted_fg: Color,
    pub highlighted_darker_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

impl TatPalette {
    pub fn selected_style(&self) -> Style {
        Style::default()
        .fg(self.selected_fg)
        .bg(self.selected_bg)
    }

    pub fn highlighted_style(&self) -> Style {
        Style::default()
        .fg(self.highlighted_fg)
    }

    pub fn highlighted_darker_fg(&self) -> Style {
        Style::default()
        .fg(self.highlighted_darker_fg)
    }

    pub fn default_style(&self) -> Style {
        Style::default()
        .fg(self.default_fg)
    }
}

pub mod palette {
    use ratatui::style::palette::tailwind;

    use super::TatPalette;

    pub const DEFAULT: TatPalette = TatPalette {
        default_fg: tailwind::SLATE.c100,
        highlighted_fg: tailwind::SLATE.c400,
        highlighted_darker_fg: tailwind::SLATE.c500,
        selected_bg: tailwind::SLATE.c400,
        selected_fg: tailwind::SLATE.c950,
    };
}
