use ratatui::style::{palette::tailwind, Color, Style};

pub const SHOW_HELP: &str = " <type ? to show help> ";
pub const TITLE_PROGRAM: &str = " Terminal Attribute Table ";
pub const TITLE_DATASET_INFO: &str = " Dataset ";
pub const TITLE_LAYER_INFO: &str = " Layer Information ";
pub const TITLE_GDAL_LOG: &str = " GDAL Log ";
pub const TITLE_DEBUG_LOG: &str = " Debug Log ";
pub const TITLE_FULL_VALUE: &str = " Debug Log ";
pub const TITLE_LAYER_LIST: &str = " Layers ";
pub const TITLE_HELP: &str = " Help ";
pub const POPUP_HINT: &str = " <press q to close> ";
pub const HELP_TEXT_LAYERSELECT: &str = "Keybinds for Main Menu
----------------------
Basic Navigation:
     or h: Left
     or j: Down
     or k: Up
     or l: Right
    Enter: Open Table
    Tab and SHIFT+Tab or Left/Right: Switch between Layer List or Info

General:
    q: Previous Menu
    CTRL + Q or q when in Main Menu: Quit
    ?: Show Help

Advanced Navigation:
    g: Scroll to Top
    G: Scroll to Bottom
    CTRL + D: Scroll Down (half page)
    CTRL + U: Scroll Up (half page)
    CTRL + F or PageDown: Scroll Down (full page)
    CTRL + B or PageUp: Scroll Up (full page)

Miscellaneous:
    L: Open GDAL Log

Remarks
-------
This is the main menu of tat. You may go through and inspect the available layers
in the selected dataset. You can also select a layer and enter into its attribute
table.
";
pub const HELP_TEXT_TABLE: &str = "Keybinds for Attribute Table
----------------------------
Basic Navigation:
     or h: Left
     or j: Down
     or k: Up
     or l: Right

Table:
    Enter: Display Selected Value in Pop-Up 
    CTRL + C: Copy Selected Value to Clipboard

General:
    q: Previous Menu
    CTRL + Q: Quit
    ?: Show Help

Advanced Navigation:
    Home or 0: Jump to First Column
    End or $: Jump to Last Column
    g: Scroll to Top
    G: Scroll to Bottom
    CTRL + D: Scroll Down (half page)
    CTRL + U: Scroll Up (half page)
    CTRL + F or PageDown: Scroll Down (full page)
    CTRL + B or PageUp: Scroll Up (full page)

Miscellaneous:
    L: Open GDAL Log

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
