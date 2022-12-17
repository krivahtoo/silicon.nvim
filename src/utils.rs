use image::Rgba;
use silicon::{font::FontStyle, utils::ToRgba};

pub trait IntoFont {
    fn to_font(self) -> Vec<(String, f32)>;
}

impl IntoFont for &str {
    fn to_font(self) -> Vec<(String, f32)> {
        let mut result = vec![];
        let mut has_hack = false;
        for font in self.split(';') {
            let tmp = font.split('=').collect::<Vec<_>>();
            let font_name = tmp[0].to_owned();
            let font_size = tmp
                .get(1)
                .map(|s| s.parse::<f32>().unwrap_or(26.0))
                .unwrap_or(26.0);
            has_hack = has_hack || font_name == "Hack";
            result.push((font_name, font_size));
        }
        // fallback for now until it is fixed in silicon upstream
        if !has_hack {
            result.push(("Hack".into(), 26.0));
        }
        result
    }
}

pub trait IntoFontStyle {
    fn to_style(self) -> FontStyle;
}

impl IntoFontStyle for &str {
    fn to_style(self) -> FontStyle {
        match self {
            "bold" => FontStyle::BOLD,
            "italic" => FontStyle::ITALIC,
            "bolditalic" => FontStyle::BOLDITALIC,
            _ => FontStyle::REGULAR,
        }
    }
}

pub fn parse_str_color(s: &str) -> anyhow::Result<Rgba<u8>, anyhow::Error> {
    s.to_rgba()
        .map_err(|_| format_err!("Invalid color: `{}`", s))
}
