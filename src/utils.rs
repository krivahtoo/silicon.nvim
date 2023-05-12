use image::Rgba;
use nvim_oxi::api;
use silicon::{font::FontStyle, utils::ToRgba};

use super::error::Error;
use super::config::Opts;

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

pub fn parse_str_color(s: &str) -> Result<Rgba<u8>, Error> {
    Ok(s.to_rgba()?)
}

pub fn get_lines(opts: &Opts) -> Result<String, Error> {
    let mut gobble_len: Option<usize> = None;
    Ok(api::call_function::<_, Vec<String>>(
        "getbufline",
        (
            api::call_function::<_, i32>("bufnr", ('%',))?,
            if opts.highlight_selection.unwrap_or_default() {
                1
            } else {
                opts.start as i32
            },
            if opts.highlight_selection.unwrap_or_default() {
                "$".to_owned()
            } else {
                format!("{}", opts.end)
            },
        ),
    )?
    .iter()
    .map(|s| match (gobble_len, opts.gobble.unwrap_or_default()) {
        (Some(len), true) => s.chars().skip(len).collect(),
        (None, true) => {
            let line = s.clone();
            gobble_len = Some(get_globble_len(&line));
            s.chars().skip(gobble_len.unwrap()).collect::<String>()
        }
        (_, false) => s.clone(),
    })
    .fold(String::new(), |a, b| a + &b + "\n"))
}

fn get_globble_len(line: &str) -> usize {
    let mut len: usize = 0;
    for (ch, i) in line.chars().zip(0..) {
        if ch != ' ' {
            len = i;
            break;
        }
    }
    len
}