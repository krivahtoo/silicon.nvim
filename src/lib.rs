#[macro_use]
extern crate anyhow;

use std::path::PathBuf;

use nvim_oxi::{
    self as oxi, api, object, opts::*, types::*, Dictionary, FromObject, Function, Object, Result,
    ToObject,
};

use image::DynamicImage;
use serde::{Deserialize, Serialize};
use silicon::formatter::ImageFormatterBuilder;
use silicon::utils::{init_syntect, ShadowAdder};
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
pub fn dump_image_to_clipboard(image: &DynamicImage) -> anyhow::Result<(), Error> {
    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    unsafe {
        Pasteboard::Image.copy(temp.path().to_str().unwrap());
    }
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn dump_image_to_clipboard(image: &DynamicImage) -> anyhow::Result<(), Error> {
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
pub fn dump_image_to_clipboard(_image: &DynamicImage) -> anyhow::Result<(), Error> {
    Err(format_err!(
        "This feature hasn't been implemented for your system"
    ))
}

#[derive(Clone, Serialize, Deserialize)]
struct Opts {
    font: Option<String>,
    font_size: Option<f32>,

    theme: Option<String>,

    line_number: Option<bool>,
    line_pad: Option<u32>,
    line_offset: Option<u32>,

    round_corner: Option<bool>,
    window_controls: Option<bool>,

    output: Option<PathBuf>,

    start: usize,
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

fn save_image(opts: Opts) -> Result<()> {
    let (ps, ts) = init_syntect();
    let code = api::get_current_buf()
        .get_lines(opts.start, opts.end, false)?
        .fold(String::new(), |a, b| a + b.to_string().as_str() + "\n")
        .as_str()
        .to_owned();
    let ft: oxi::String = api::get_current_buf().get_option("filetype")?;

    let syntax = ps.find_syntax_by_token(ft.as_str().unwrap()).unwrap();
    let theme = &ts.themes[opts.theme.unwrap_or(String::from("Dracula")).as_str()];

    let mut h = HighlightLines::new(syntax, theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight(line, &ps))
        .collect::<Vec<_>>();

    let mut formatter = ImageFormatterBuilder::new()
        .font(vec![(
            opts.font.unwrap_or(String::from("Hack")).as_str(),
            opts.font_size.unwrap_or(20.0),
        )])
        .line_pad(opts.line_pad.unwrap_or(2))
        .line_offset(opts.line_offset.unwrap_or(1))
        .line_number(opts.line_number.unwrap_or(false))
        .window_controls(opts.window_controls.unwrap_or(true))
        .round_corner(opts.round_corner.unwrap_or(true))
        .shadow_adder(ShadowAdder::default())
        .build()
        .unwrap();
    let image = formatter.format(&highlight, theme);

    if opts.output.is_some() {
        if let Err(e) = image.save(opts.output.unwrap().as_path()) {
            api::err_writeln(format!("[silicon.nvim]: Failed to save image: {e}").as_str())
        };
    } else {
        if let Err(e) = dump_image_to_clipboard(&image) {
            api::err_writeln(format!("[silicon.nvim]: {e}").as_str())
        };
    }

    Ok(())
}

#[oxi::module]
fn silicon() -> Result<Dictionary> {
    // Remaps `SS` to `Silicon` in visual mode.
    // api::set_keymap(Mode::Insert, "SS", "Silicon", Some(&SetKeymapOptsBuilder::default().desc("Save image of code").silent(true).build()))?;

    // Create a new `Silicon` command.
    let cmd_opts: Opts = Opts {
        font: None,
        theme: None,
        start: 0,
        end: 1,
        font_size: None,
        line_number: None,
        line_pad: None,
        line_offset: None,
        round_corner: None,
        window_controls: None,
        output: None,
    };

    let opts = CreateCommandOpts::builder()
        .range(CommandRange::CurrentLine)
        .desc("create a beautiful image of your source code.")
        .nargs(CommandNArgs::ZeroOrOne)
        .build();

    let silicon_cmd = move |args: CommandArgs| -> Result<()> {
        let opts = cmd_opts.clone();
        let output = if args.args.is_some() {
            Some(PathBuf::from(args.args.unwrap()))
        } else {
            None
        };
        save_image(Opts {
            start: args.line1 - 1,
            end: args.line2,
            output,
            ..opts
        })
    };
    api::create_user_command("Silicon", silicon_cmd, Some(&opts))?;

    Ok(Dictionary::from_iter([(
        "capture",
        Function::from_fn(save_image),
    )]))
}
