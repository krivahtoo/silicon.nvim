#[macro_use]
extern crate anyhow;

mod config;
mod utils;

use std::path::PathBuf;

use config::Opts;
use utils::{parse_str_color, IntoFont, IntoFontStyle};

use image::DynamicImage;
use nvim_oxi as oxi;
use oxi::{
    api::{self, opts::*, types::*, Buffer},
    Dictionary, Function,
};
use silicon::{
    assets::HighlightingAssets,
    font::FontCollection,
    formatter::ImageFormatterBuilder,
    utils::{Background, ShadowAdder, ToRgba},
};
use syntect::{easy::HighlightLines, util::LinesWithEndings};

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
        .args([
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

fn save_image(opts: Opts) -> oxi::Result<()> {
    let ha = HighlightingAssets::new();
    let (ps, ts) = (ha.syntax_set, ha.theme_set);
    if opts.start == 0 || opts.end == 0 {
        return Err(api::Error::Other(
            "line1 and line2 are required when calling `capture` directly".to_owned(),
        ))
        .map_err(Into::into);
    }
    let code = Buffer::current()
        .get_lines((opts.start - 1)..=opts.end, false)?
        .fold(String::new(), |a, b| a + b.to_string().as_str() + "\n");
    let ft: oxi::String = Buffer::current().get_option("filetype")?;

    let syntax = ps
        .find_syntax_by_token(ft.as_str().unwrap())
        .ok_or_else(|| api::Error::Other("Could not find syntax for filetype.".to_owned()))?;
    let theme = match ts.themes.get(
        opts.theme
            .clone()
            .unwrap_or_else(|| "Dracula".to_owned())
            .as_str(),
    ) {
        Some(theme) => theme,
        _ => {
            api::err_writeln(&format!(
                "Could not load '{}' theme.",
                opts.theme.unwrap_or_default()
            ));
            ts.themes
                .get("Dracula")
                .ok_or_else(|| api::Error::Other("Error loading dracula theme".to_owned()))?
        }
    };

    let mut h = HighlightLines::new(syntax, theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight_line(line, &ps))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| api::Error::Other(format!("Error highlighting lines: {}", e)))?;

    let adder = ShadowAdder::default()
        .background(Background::Solid(
            parse_str_color(
                opts.background
                    .unwrap_or_else(|| "#eef".to_owned())
                    .as_str(),
            )
            .unwrap(),
        ))
        .shadow_color(
            parse_str_color(
                opts.shadow
                    .color
                    .unwrap_or_else(|| "#555".to_owned())
                    .as_str(),
            )
            .unwrap(),
        )
        .blur_radius(opts.shadow.blur_radius)
        .offset_x(opts.shadow.offset_x)
        .offset_y(opts.shadow.offset_y)
        .pad_horiz(opts.pad_horiz.unwrap_or(80))
        .pad_vert(opts.pad_vert.unwrap_or(100));

    let fonts = opts.font.unwrap_or_else(|| "Hack=20".to_owned()).to_font();

    let mut formatter = ImageFormatterBuilder::new()
        .font(fonts.clone())
        .tab_width(opts.tab_width.unwrap_or(4))
        .line_pad(opts.line_pad.unwrap_or(2))
        .line_offset(opts.line_offset.unwrap_or(1))
        .line_number(opts.line_number.unwrap_or(false))
        .window_controls(opts.window_controls.unwrap_or(true))
        .round_corner(opts.round_corner.unwrap_or(true))
        .shadow_adder(adder)
        .build()
        .map_err(|e| api::Error::Other(format!("font error: {}", e)))?;
    let mut image = formatter.format(&highlight, theme);

    if let Some(text) = opts.watermark.text {
        let font = FontCollection::new(fonts.as_slice()).unwrap();

        let (x, y) = (
            image.to_rgba8().width() - (font.get_text_len(text.as_str()) + font.get_text_len("  ")),
            image.to_rgba8().height() - (font.get_font_height() * 2),
        );

        font.draw_text_mut(
            &mut image,
            opts.watermark
                .color
                .unwrap_or_else(|| "#222".to_owned())
                .to_rgba()
                .unwrap(),
            x,
            y,
            opts.watermark
                .style
                .unwrap_or_else(|| "bold".to_owned())
                .to_style(),
            text.as_str(),
        );
    }

    if let Some(output) = opts.output {
        match image.save(output.as_path()) {
            Err(e) => {
                api::err_writeln(format!("[silicon.nvim]: Failed to save image: {e}").as_str())
            }
            Ok(_) => {
                api::notify(
                    "Image saved to file",
                    LogLevel::Info,
                    &NotifyOpts::default(),
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
                    &NotifyOpts::default(),
                )?;
            }
        };
    }

    Ok(())
}

fn setup(cmd_opts: Opts) -> oxi::Result<()> {
    // Create a new `Silicon` command.
    let opts = CreateCommandOpts::builder()
        .range(CommandRange::WholeFile)
        .desc("create a beautiful image of your source code.")
        .nargs(CommandNArgs::ZeroOrOne)
        .build();

    let silicon_cmd = move |args: CommandArgs| {
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
        .ok();
        Ok(())
    };
    api::create_user_command("Silicon", silicon_cmd, &opts)?;
    // Remaps `SS` to `Silicon` in visual mode.
    api::set_keymap(
        Mode::Visual,
        "SS",
        "Silicon",
        &SetKeymapOptsBuilder::default()
            .desc("Save image of code")
            .silent(true)
            .build(),
    )
    .map_err(Into::into)
}

#[oxi::module]
fn silicon() -> oxi::Result<Dictionary> {
    Ok(Dictionary::from_iter([
        ("capture", Function::from_fn(save_image)),
        ("setup", Function::from_fn(setup)),
    ]))
}
