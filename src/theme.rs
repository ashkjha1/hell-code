use ratatui::prelude::*;

#[allow(dead_code)]
pub struct Theme {
    // Catppuccin Mocha Palette
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,
    pub text: Color,
    pub subtext1: Color,
    pub subtext0: Color,
    pub overlay2: Color,
    pub overlay1: Color,
    pub overlay0: Color,
    pub surface2: Color,
    pub surface1: Color,
    pub surface0: Color,
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,

    // Functional aliases (Legacy support or common names)
    pub midnight: Color,
    pub charcoal: Color,
    pub blood_red: Color,
    pub gray: Color,
    pub text_bright: Color,
}

impl Default for Theme {
    fn default() -> Self {
        let mocha = (
            Color::Rgb(245, 224, 220), // Rosewater
            Color::Rgb(242, 205, 205), // Flamingo
            Color::Rgb(245, 194, 231), // Pink
            Color::Rgb(203, 166, 247), // Mauve
            Color::Rgb(243, 139, 168), // Red
            Color::Rgb(235, 160, 172), // Maroon
            Color::Rgb(250, 179, 135), // Peach
            Color::Rgb(249, 226, 175), // Yellow
            Color::Rgb(166, 227, 161), // Green
            Color::Rgb(148, 226, 213), // Teal
            Color::Rgb(137, 220, 235), // Sky
            Color::Rgb(116, 199, 236), // Sapphire
            Color::Rgb(137, 180, 250), // Blue
            Color::Rgb(180, 190, 254), // Lavender
            Color::Rgb(205, 214, 244), // Text
            Color::Rgb(186, 194, 222), // Subtext1
            Color::Rgb(166, 173, 200), // Subtext0
            Color::Rgb(147, 153, 178), // Overlay2
            Color::Rgb(127, 132, 156), // Overlay1
            Color::Rgb(108, 112, 134), // Overlay0
            Color::Rgb(88, 91, 112),   // Surface2
            Color::Rgb(69, 71, 90),    // Surface1
            Color::Rgb(49, 50, 68),    // Surface0
            Color::Rgb(30, 30, 46),    // Base
            Color::Rgb(24, 24, 37),    // Mantle
            Color::Rgb(17, 17, 27),    // Crust
        );

        Self {
            rosewater: mocha.0,
            flamingo: mocha.1,
            pink: mocha.2,
            mauve: mocha.3,
            red: mocha.4,
            maroon: mocha.5,
            peach: mocha.6,
            yellow: mocha.7,
            green: mocha.8,
            teal: mocha.9,
            sky: mocha.10,
            sapphire: mocha.11,
            blue: mocha.12,
            lavender: mocha.13,
            text: mocha.14,
            subtext1: mocha.15,
            subtext0: mocha.16,
            overlay2: mocha.17,
            overlay1: mocha.18,
            overlay0: mocha.19,
            surface2: mocha.20,
            surface1: mocha.21,
            surface0: mocha.22,
            base: mocha.23,
            mantle: mocha.24,
            crust: mocha.25,

            // Functional mappings
            midnight: mocha.23,    // Base
            charcoal: mocha.21,    // Surface1
            blood_red: mocha.3,    // Mauve (Logo accent)
            gray: mocha.19,        // Overlay0
            text_bright: mocha.14, // Text
        }
    }
}

pub fn get_message_style(msg_type: &crate::app::MessageType, theme: &Theme) -> (String, Color, String) {
    match msg_type {
        crate::app::MessageType::User => ("USER".to_string(), theme.blue, "λ".to_string()),
        crate::app::MessageType::Assistant => ("HELL-CODE".to_string(), theme.mauve, "🧠".to_string()),
        crate::app::MessageType::System => ("SYSTEM".to_string(), theme.overlay1, "⚙".to_string()),
        crate::app::MessageType::Tool => ("TOOL".to_string(), theme.peach, "🛠".to_string()),
    }
}
