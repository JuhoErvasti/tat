use ratatui::style::{palette::tailwind, Color, Style};

pub const SHOW_HELP: &str = " <type ? to show help> ";
pub const TITLE_PROGRAM: &str = " Terminal Attribute Table ";
pub const TITLE_DATASET_INFO: &str = " Dataset ";
pub const TITLE_LAYER_INFO: &str = " Layer Information ";
pub const TITLE_GDAL_LOG: &str = " GDAL Log ";
pub const TITLE_LAYER_LIST: &str = " Layers ";
pub const TITLE_HELP: &str = " Help ";
pub const POPUP_HINT: &str = " <press q to close> ";
pub const HELP_TEXT_LAYERSELECT: &str = "Keybinds for Main Menu
----------------------
Basic Navigation:
    Left:  or h
    Down:  or j
    Up:  or k
    Right:  or l
    Open Table: Enter
    Switch between Layer List or Info: Tab and SHIFT+Tab or Left/Right

General:
    Previous Menu: q
    Quit: CTRL + C or q when in Main Menu
    Show Help: ?

Advanced Navigation:
    Scroll to Top: g
    Scroll to Bottom: G
    Scroll Down (half page): CTRL + D
    Scroll Up (half page): CTRL + U
    Scroll Down (full page): CTRL + F or PageDown
    Scroll Up (full page): CTRL + B or PageUp

Miscellaneous:
    Open GDAL Log: L

Remarks
-------
This is the main menu of tat. You may go through and inspect the available layers
in the selected dataset. You can also select a layer and enter into its attribute
table.
";
pub const HELP_TEXT_TABLE: &str = "Keybinds for Attribute Table
----------------------------
Basic Navigation:
    Left:  or h
    Down:  or j
    Up:  or k
    Right:  or l

General:
    Previous Menu: q
    Quit: CTRL + C
    Show Help: ?

Advanced Navigation:
    Jump to First Column: Home or 0
    Jump to Last Column: End or $
    Scroll to Top: g
    Scroll to Bottom: G
    Scroll Down (half page): CTRL + D
    Scroll Up (half page): CTRL + U
    Scroll Down (full page): CTRL + F or PageDown
    Scroll Up (full page): CTRL + B or PageUp

Miscellaneous:
    Open GDAL Log: L

Remarks
-------
This is the attribute table itself. You can inspect the features in the selected
layer.
";

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
