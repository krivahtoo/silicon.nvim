#[macro_use]
extern crate anyhow;

use std::path::PathBuf;

use nvim_oxi::{
    self as oxi, api, object, opts::*, types::*, Dictionary, FromObject, Function, Object, Result,
    ToObject,
};

use image::{DynamicImage, Rgba};
use serde::{Deserialize, Serialize};
use silicon::formatter::ImageFormatterBuilder;
use silicon::utils::{init_syntect, Background, ShadowAdder, ToRgba};
use syntect::easy::HighlightLines;
use syntect::util::LinesWithEndings;

#[cfg(target_os = "windows")]
use {
    clipboard_win::{formats, Clipboard, Setter},
    image::ImageOutputFormat,
};
#[cfg(target_os = "macos")]
use {image::ImageOutputFormat, pasteboard::Pasteboard};
#[cfg(target_os = "linux")]
use {image::ImageOutputFormat, std::process::Command};

#[cfg(target_os = "linux")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    Command::new("xclip")
        .args(&[
            "-sel",
            "clip",
            "-t",
            "image/png",
            temp.path().to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> anyhow::Result<()> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    unsafe {
        Pasteboard::Image.copy(temp.path().to_str().unwrap());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> anyhow::Result<()> {
    let mut temp: Vec<u8> = Vec::new();

    // Convert the image to RGB without alpha because the clipboard
    // of windows doesn't support it.
    let image = DynamicImage::ImageRgb8(image.to_rgb());

    image.write_to(&mut temp, ImageOutputFormat::Bmp)?;

    let _clip =
        Clipboard::new_attempts(10).map_err(|e| format_err!("Couldn't open clipboard: {}", e))?;

    formats::Bitmap
        .write_clipboard(&temp)
        .map_err(|e| format_err!("Failed copy image: {}", e))?;
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn dump_image_to_clipboard(_image: &DynamicImage) -> anyhow::Result<()> {
    Err(format_err!(
        "This feature hasn't been implemented for your system"
    ))
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct ShadowOpts {
    #[serde(default)]
    blur_radius: f32,
    #[serde(default)]
    offset_x: i32,
    #[serde(default)]
    offset_y: i32,
    color: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Opts {
    font: Option<String>,

    theme: Option<String>,

    background: Option<String>,

    #[serde(default)]
    shadow: ShadowOpts,

    pad_horiz: Option<u32>,
    pad_vert: Option<u32>,

    line_number: Option<bool>,
    line_pad: Option<u32>,
    line_offset: Option<u32>,

    tab_width: Option<u8>,

    round_corner: Option<bool>,
    window_controls: Option<bool>,

    output: Option<PathBuf>,

    #[serde(alias = "line1")]
    #[serde(default)]
    start: usize,
    #[serde(alias = "line2")]
    #[serde(default)]
    end: usize,
}

impl FromObject for Opts {
    fn from_obj(obj: Object) -> Result<Self> {
        Self::deserialize(object::Deserializer::new(obj))
    }
}

impl ToObject for Opts {
    fn to_obj(self) -> Result<Object> {
        self.serialize(object::Serializer::new())
    }
}

fn parse_str_color(s: &str) -> anyhow::Result<Rgba<u8>, anyhow::Error> {
    s.to_rgba()
        .map_err(|_| format_err!("Invalid color: `{}`", s))
}

fn parse_font_str(s: &str) -> Vec<(String, f32)> {
    let mut result = vec![];
    for font in s.split(';') {
        let tmp = font.split('=').collect::<Vec<_>>();
        let font_name = tmp[0].to_owned();
        let font_size = tmp
            .get(1)
            .map(|s| s.parse::<f32>().unwrap())
            .unwrap_or(26.0);
        result.push((font_name, font_size));
    }
    result
}

fn save_image(opts: Opts) -> Result<()> {
    let (ps, ts) = init_syntect();
    if opts.start == 0 || opts.end == 0 {
        return Err(oxi::Error::Other(
            "line1 and line2 are required when calling `capture` directly".to_owned(),
        ));
    }
    let code = api::get_current_buf()
        .get_lines(opts.start - 1, opts.end, false)?
        .fold(String::new(), |a, b| a + b.to_string().as_str() + "\n")
        .as_str()
        .to_owned();
    let ft: oxi::String = api::get_current_buf().get_option("filetype")?;

    let syntax = ps
        .find_syntax_by_token(ft.as_str().unwrap())
        .ok_or_else(|| oxi::Error::Other("Could not find syntax for filetype.".to_owned()))?;
    let theme = &ts.themes[opts.theme.unwrap_or("Dracula".to_owned()).as_str()];

    let mut h = HighlightLines::new(syntax, theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight(line, &ps))
        .collect::<Vec<_>>();

    let adder = ShadowAdder::default()
        .background(Background::Solid(
            parse_str_color(opts.background.unwrap_or("#eef".to_owned()).as_str()).unwrap(),
        ))
        .shadow_color(
            parse_str_color(opts.shadow.color.unwrap_or("#555".to_owned()).as_str()).unwrap(),
        )
        .blur_radius(opts.shadow.blur_radius)
        .offset_x(opts.shadow.offset_x)
        .offset_y(opts.shadow.offset_y)
        .pad_horiz(opts.pad_horiz.unwrap_or(80))
        .pad_vert(opts.pad_vert.unwrap_or(100));

    let mut formatter = ImageFormatterBuilder::new()
        .font(parse_font_str(
            opts.font.unwrap_or("Hack=20".to_owned()).as_str(),
        ))
        .tab_width(opts.tab_width.unwrap_or(4))
        .line_pad(opts.line_pad.unwrap_or(2))
        .line_offset(opts.line_offset.unwrap_or(1))
        .line_number(opts.line_number.unwrap_or(false))
        .window_controls(opts.window_controls.unwrap_or(true))
        .round_corner(opts.round_corner.unwrap_or(true))
        .shadow_adder(adder)
        .build()
        .unwrap();
    let image = formatter.format(&highlight, theme);

    if opts.output.is_some() {
        match image.save(opts.output.unwrap().as_path()) {
            Err(e) => {
                api::err_writeln(format!("[silicon.nvim]: Failed to save image: {e}").as_str())
            }
            Ok(_) => {
                api::notify(
                    "Image saved to file",
                    LogLevel::Info,
                    Some(&NotifyOpts::default()),
                )?;
            }
        };
    } else {
        match dump_image_to_clipboard(&image) {
            Err(e) => api::err_writeln(format!("[silicon.nvim]: {e}").as_str()),
            Ok(_) => {
                api::notify(
                    "Image saved to clipboard",
                    LogLevel::Info,
                    Some(&NotifyOpts::default()),
                )?;
            }
        };
    }

    Ok(())
}

fn setup(cmd_opts: Opts) -> Result<()> {
    // Create a new `Silicon` command.
    let opts = CreateCommandOpts::builder()
        .range(CommandRange::CurrentLine)
        .desc("create a beautiful image of your source code.")
        .nargs(CommandNArgs::ZeroOrOne)
        .build();

    let silicon_cmd = move |args: CommandArgs| -> Result<()> {
        let output = args
            .args
            .is_some()
            .then(|| PathBuf::from(args.args.unwrap()));
        save_image(Opts {
            start: args.line1,
            end: args.line2,
            output,
            ..cmd_opts.clone()
        })
    };
    api::create_user_command("Silicon", silicon_cmd, Some(&opts))?;
    // Remaps `SS` to `Silicon` in visual mode.
    api::set_keymap(
        Mode::Visual,
        "SS",
        "Silicon",
        Some(
            &SetKeymapOptsBuilder::default()
                .desc("Save image of code")
                .silent(true)
                .build(),
        ),
    )
}

#[oxi::module]
fn silicon() -> Result<Dictionary> {
    Ok(Dictionary::from_iter([
        ("capture", Function::from_fn(save_image)),
        ("setup", Function::from_fn(setup)),
    ]))
}
