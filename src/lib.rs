#[macro_use]
extern crate anyhow;

mod clipboard;
mod config;
mod utils;

use std::path::PathBuf;

use clipboard::dump_image_to_clipboard;
use config::{Opts, OutputOpts};
use time::{format_description, OffsetDateTime};
use utils::{parse_str_color, IntoFont, IntoFontStyle};

use nvim_oxi as oxi;
use oxi::{
    api::{self, opts::*, types::*, Buffer, Error},
    Dictionary, Function,
};
use silicon::{
    assets::HighlightingAssets,
    font::FontCollection,
    formatter::ImageFormatterBuilder,
    utils::{Background, ShadowAdder, ToRgba},
};
use syntect::{easy::HighlightLines, util::LinesWithEndings};

fn save_image(opts: Opts) -> Result<(), Error> {
    let ha = HighlightingAssets::new();
    let (ps, ts) = (ha.syntax_set, ha.theme_set);
    if opts.start == 0 || opts.end == 0 {
        return Err(Error::Other(
            "line1 and line2 are required when calling `capture` directly".to_owned(),
        ));
    }

    let code = api::call_function::<_, Vec<String>>(
        "getbufline",
        (
            api::call_function::<_, i32>("bufnr", ('%',))?, // current buffer
            opts.start as i32,
            opts.end as i32,
        ),
    )?
    .iter()
    .fold(String::new(), |a, b| a + &b.to_string() + "\n");

    let ft: oxi::String = Buffer::current().get_option("filetype")?;

    let syntax = ps
        .find_syntax_by_token(
            ft.as_str()
                .map_err(|e| Error::Other(format!("utf error: {e}")))?,
        )
        .ok_or_else(|| Error::Other("Could not find syntax for filetype.".to_owned()))?;

    let theme = match ts
        .themes
        .get(&opts.theme.clone().unwrap_or_else(|| "Dracula".to_owned()))
    {
        Some(theme) => theme,
        _ => {
            api::err_writeln(&format!(
                "Could not load '{}' theme.",
                opts.theme.unwrap_or_default()
            ));
            ts.themes
                .get("Dracula")
                .ok_or_else(|| Error::Other("Error loading dracula theme".to_owned()))?
        }
    };

    let mut h = HighlightLines::new(syntax, theme);
    let highlight = LinesWithEndings::from(&code)
        .map(|line| h.highlight_line(line, &ps))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Error::Other(format!("Error highlighting lines: {}", e)))?;

    let adder = ShadowAdder::default()
        .background(Background::Solid(
            parse_str_color(&opts.background.unwrap_or_else(|| "#eef".to_owned()))
                .map_err(|e| Error::Other(format!("{e}")))?,
        ))
        .shadow_color(
            parse_str_color(&opts.shadow.color.unwrap_or_else(|| "#555".to_owned()))
                .map_err(|e| Error::Other(format!("{e}")))?,
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
        .map_err(|e| Error::Other(format!("font error: {}", e)))?;
    let mut image = formatter.format(&highlight, theme);

    if let Some(text) = opts.watermark.text {
        let font = FontCollection::new(fonts.as_slice()).unwrap();

        let (x, y) = (
            image.to_rgba8().width() - (font.get_text_len(&text) + font.get_text_len("  ")),
            image.to_rgba8().height() - (font.get_font_height() * 2),
        );

        font.draw_text_mut(
            &mut image,
            opts.watermark
                .color
                .unwrap_or_else(|| "#222".to_owned())
                .to_rgba()
                .map_err(|e| Error::Other(format!("{e}")))?,
            x,
            y,
            opts.watermark
                .style
                .unwrap_or_else(|| "bold".to_owned())
                .to_style(),
            &text,
        );
    }

    if let Some(output) = opts.output.file {
        match image.save(output.as_path()) {
            Err(e) => api::err_writeln(&format!("[silicon.nvim]: Failed to save image: {e}")),
            Ok(_) => {
                api::notify(
                    "Image saved to file",
                    LogLevel::Info,
                    &NotifyOpts::default(),
                )?;
            }
        };
    } else if opts.output.clipboard.unwrap_or_default() {
        match dump_image_to_clipboard(&image) {
            Err(e) => api::err_writeln(&format!("[silicon.nvim]: {e}")),
            Ok(_) => {
                api::notify(
                    "Image saved to clipboard",
                    LogLevel::Info,
                    &NotifyOpts::default(),
                )?;
            }
        };
    } else {
        match format_description::parse(&opts.output.format.unwrap_or_else(|| {
            String::from("silicon_[year][month][day]_[hour][minute][second].png")
        })) {
            Ok(fmt) => {
                let file = OffsetDateTime::now_utc()
                    .format(&fmt)
                    .map_err(|e| Error::Other(format!("Error formatting filename: {e}")))?;
                let mut path = opts.output.path.unwrap_or_default();
                path.push(&file);
                match image.save(path) {
                    Err(e) => {
                        api::err_writeln(&format!("[silicon.nvim]: Failed to save image: {e}"))
                    }
                    Ok(_) => {
                        api::notify(
                            &format!("Image saved to {}", file),
                            LogLevel::Info,
                            &NotifyOpts::default(),
                        )?;
                    }
                };
            }
            Err(e) => api::err_writeln(&format!("[silicon.nvim]: {e}")),
        };
    }

    Ok(())
}

fn setup(cmd_opts: Opts) -> Result<(), Error> {
    // Create a new `Silicon` command.
    let opts = CreateCommandOpts::builder()
        .range(CommandRange::WholeFile)
        .desc("create a beautiful image of your source code.")
        .nargs(CommandNArgs::ZeroOrOne)
        .bang(true)
        .build();

    let silicon_cmd = move |args: CommandArgs| {
        let file = args
            .args
            .is_some()
            .then(|| PathBuf::from(args.args.unwrap()));
        save_image(Opts {
            start: args.line1,
            end: args.line2,
            output: OutputOpts {
                file,
                clipboard: Some(!args.bang),
                ..cmd_opts.output.clone()
            },
            ..cmd_opts.clone()
        })?;
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
}

#[oxi::module]
fn silicon() -> oxi::Result<Dictionary> {
    Ok(Dictionary::from_iter([
        ("capture", Function::from_fn(save_image)),
        ("setup", Function::from_fn(setup)),
    ]))
}
