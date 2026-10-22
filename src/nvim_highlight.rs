use nvim_oxi::{self as oxi, api, serde::Deserializer, Dictionary, Object};
use serde::Deserialize;
use syntect::highlighting::{Color, FontStyle, Style, Theme, ThemeSettings};

use super::config::Opts;
use super::error::Error;
use super::utils::parse_str_color;

const COLLECTOR: &str = include_str!("highlight.lua");

#[derive(Deserialize)]
pub struct Span {
    pub text: String,
    pub fg: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Deserialize)]
pub struct NormalHl {
    pub fg: String,
    pub bg: String,
}

#[derive(Deserialize)]
pub struct NvimHighlight {
    pub normal: NormalHl,
    pub lines: Vec<Vec<Span>>,
}

/// Collect tree-sitter highlights for the requested lines of the current buffer,
/// resolved against the active colorscheme. Returns `None` when the buffer has
/// no tree-sitter parser.
pub fn collect(opts: &Opts) -> Result<Option<NvimHighlight>, Error> {
    let bufnr = api::call_function::<_, i64>("bufnr", ('%',))?;
    let (start, stop) = if opts.highlight_selection.unwrap_or_default() {
        (0, api::call_function::<_, i64>("line", ('$',))?)
    } else {
        (opts.start as i64 - 1, opts.end as i64)
    };

    let args = Dictionary::from_iter([
        ("bufnr", Object::from(bufnr)),
        ("start", Object::from(start)),
        ("stop", Object::from(stop)),
        ("gobble", Object::from(opts.gobble.unwrap_or_default())),
    ]);
    let result = api::call_function::<_, Object>(
        "luaeval",
        (
            "assert(load(_A[1]))()(_A[2])",
            oxi::Array::from_iter([Object::from(COLLECTOR), Object::from(args)]),
        ),
    )?;

    Option::<NvimHighlight>::deserialize(Deserializer::new(result))
        .map_err(|e| Error::Generic(format!("Invalid highlight data: {e}")))
}

fn to_color(s: &str) -> Result<Color, Error> {
    let [r, g, b, a] = parse_str_color(s)?.0;
    Ok(Color { r, g, b, a })
}

/// Build a theme carrying only the colors silicon's formatter reads.
pub fn to_theme(hl: &NvimHighlight) -> Result<Theme, Error> {
    Ok(Theme {
        settings: ThemeSettings {
            foreground: Some(to_color(&hl.normal.fg)?),
            background: Some(to_color(&hl.normal.bg)?),
            ..ThemeSettings::default()
        },
        ..Theme::default()
    })
}

pub fn to_styles(hl: &NvimHighlight) -> Result<Vec<Vec<(Style, &str)>>, Error> {
    let background = to_color(&hl.normal.bg)?;
    hl.lines
        .iter()
        .map(|line| {
            line.iter()
                .map(|span| {
                    let mut font_style = FontStyle::empty();
                    font_style.set(FontStyle::BOLD, span.bold);
                    font_style.set(FontStyle::ITALIC, span.italic);
                    font_style.set(FontStyle::UNDERLINE, span.underline);
                    Ok((
                        Style {
                            foreground: to_color(&span.fg)?,
                            background,
                            font_style,
                        },
                        span.text.as_str(),
                    ))
                })
                .collect()
        })
        .collect()
}
