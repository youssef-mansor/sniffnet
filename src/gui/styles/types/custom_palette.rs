use std::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::{de::Error as DeErrorTrait, Deserialize, Serialize};

use crate::gui::styles::custom_themes::dracula::{
    DRACULA_DARK_PALETTE, DRACULA_DARK_PALETTE_EXTENSION, DRACULA_LIGHT_PALETTE,
    DRACULA_LIGHT_PALETTE_EXTENSION,
};
use crate::gui::styles::custom_themes::gruvbox::{
    GRUVBOX_DARK_PALETTE, GRUVBOX_DARK_PALETTE_EXTENSION, GRUVBOX_LIGHT_PALETTE,
    GRUVBOX_LIGHT_PALETTE_EXTENSION,
};
use crate::gui::styles::custom_themes::nord::{
    NORD_DARK_PALETTE, NORD_DARK_PALETTE_EXTENSION, NORD_LIGHT_PALETTE,
    NORD_LIGHT_PALETTE_EXTENSION,
};
use crate::gui::styles::custom_themes::solarized::{
    SOLARIZED_DARK_PALETTE, SOLARIZED_DARK_PALETTE_EXTENSION, SOLARIZED_LIGHT_PALETTE,
    SOLARIZED_LIGHT_PALETTE_EXTENSION,
};
use crate::gui::styles::types::palette::Palette;
use crate::gui::styles::types::palette_extension::PaletteExtension;

impl Palette {
    /// Deserialize [`CustomPalette`] from `path`.
    ///
    /// # Arguments
    /// * `path` - Path to a UTF-8 encoded file containing a custom style as TOML.
    pub fn from_file<P>(path: P) -> Result<Self, toml::de::Error>
    where
        P: AsRef<Path>,
    {
        // Try to open the file at `path`
        let mut toml_reader = File::open(path)
            .map_err(DeErrorTrait::custom)
            .map(BufReader::new)?;

        // Read the ostensible TOML
        let mut style_toml = String::new();
        toml_reader
            .read_to_string(&mut style_toml)
            .map_err(DeErrorTrait::custom)?;

        toml::de::from_str(&style_toml)
    }
}

/// Built in extra styles
#[derive(Clone, Copy, Debug, Hash, PartialEq, Serialize, Deserialize)]
// #[serde(tag = "custom")]
pub enum ExtraStyles {
    DraculaDark,
    DraculaLight,
    GruvboxDark,
    GruvboxLight,
    NordDark,
    NordLight,
    SolarizedDark,
    SolarizedLight,
    CustomToml(Palette, PaletteExtension),
}

impl ExtraStyles {
    /// [`Palette`] of the [`ExtraStyles`] variant
    pub fn to_palette(self) -> Palette {
        match self {
            ExtraStyles::DraculaDark => *DRACULA_DARK_PALETTE,
            ExtraStyles::DraculaLight => *DRACULA_LIGHT_PALETTE,
            ExtraStyles::GruvboxDark => *GRUVBOX_DARK_PALETTE,
            ExtraStyles::GruvboxLight => *GRUVBOX_LIGHT_PALETTE,
            ExtraStyles::NordDark => *NORD_DARK_PALETTE,
            ExtraStyles::NordLight => *NORD_LIGHT_PALETTE,
            ExtraStyles::SolarizedDark => *SOLARIZED_DARK_PALETTE,
            ExtraStyles::SolarizedLight => *SOLARIZED_LIGHT_PALETTE,
            ExtraStyles::CustomToml(palette, _) => palette,
        }
    }

    /// [`PaletteExtension`] of the [`ExtraStyles`] variant
    pub fn to_palette_extension(self) -> PaletteExtension {
        match self {
            ExtraStyles::DraculaDark => *DRACULA_DARK_PALETTE_EXTENSION,
            ExtraStyles::DraculaLight => *DRACULA_LIGHT_PALETTE_EXTENSION,
            ExtraStyles::GruvboxDark => *GRUVBOX_DARK_PALETTE_EXTENSION,
            ExtraStyles::GruvboxLight => *GRUVBOX_LIGHT_PALETTE_EXTENSION,
            ExtraStyles::NordDark => *NORD_DARK_PALETTE_EXTENSION,
            ExtraStyles::NordLight => *NORD_LIGHT_PALETTE_EXTENSION,
            ExtraStyles::SolarizedDark => *SOLARIZED_DARK_PALETTE_EXTENSION,
            ExtraStyles::SolarizedLight => *SOLARIZED_LIGHT_PALETTE_EXTENSION,
            ExtraStyles::CustomToml(_, palette_extension) => palette_extension,
        }
    }

    /// Slice of all implemented custom styles
    pub const fn all_styles() -> &'static [Self] {
        &[
            ExtraStyles::DraculaDark,
            ExtraStyles::DraculaLight,
            ExtraStyles::GruvboxDark,
            ExtraStyles::GruvboxLight,
            ExtraStyles::NordDark,
            ExtraStyles::NordLight,
            ExtraStyles::SolarizedDark,
            ExtraStyles::SolarizedLight,
        ]
    }
}

impl fmt::Display for ExtraStyles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ExtraStyles::DraculaLight => write!(f, "Dracula (Day)"),
            ExtraStyles::DraculaDark => write!(f, "Dracula (Night)"),
            ExtraStyles::GruvboxDark => write!(f, "Gruvbox (Night)"),
            ExtraStyles::GruvboxLight => write!(f, "Gruvbox (Day)"),
            ExtraStyles::NordLight => write!(f, "Nord (Day)"),
            ExtraStyles::NordDark => write!(f, "Nord (Night)"),
            ExtraStyles::SolarizedLight => write!(f, "Solarized (Day)"),
            ExtraStyles::SolarizedDark => write!(f, "Solarized (Night)"),
            // Custom style names aren't used anywhere so this shouldn't be reached
            ExtraStyles::CustomToml(_, _) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::color;

    use super::{CustomPalette, Palette, PaletteExtension};

    fn style_path(name: &str) -> String {
        format!(
            "{}/resources/themes/{}.toml",
            env!("CARGO_MANIFEST_DIR"),
            name
        )
    }

    // NOTE: This has to be updated if `resources/themes/catppuccin.toml` changes
    fn catppuccin_style() -> CustomPalette {
        CustomPalette {
            palette: Palette {
                primary: color!(0x30, 0x34, 0x46),
                secondary: color!(0xa6, 0xd1, 0x89),
                outgoing: color!(0xf4, 0xb8, 0xe4),
                starred: color!(0xe5, 0xc8, 0x90, 0.6666667),
                text_headers: color!(0x23, 0x26, 0x34),
                text_body: color!(0xc6, 0xd0, 0xf5),
            },
        }
    }

    #[test]
    fn custompalette_from_file_de() -> Result<(), toml::de::Error> {
        let style = catppuccin_style();
        let style_de = CustomPalette::from_file(style_path("catppuccin"))?;

        assert_eq!(style, style_de);
        Ok(())
    }
}
