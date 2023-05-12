use anyhow::format_err;
use image::DynamicImage;

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
    use std::{io::Write, process::Stdio};

    let mut temp = tempfile::NamedTempFile::new()?;
    image.write_to(&mut temp, ImageOutputFormat::Png)?;
    let temp_file_path = temp.path().to_str().unwrap();

    let xclip_cmd = Command::new("xclip")
        .args(["-sel", "clip", "-t", "image/png", temp_file_path])
        .status();

    let wl_copy_cmd = Command::new("wl-copy")
        .args(["-t", "image/png"])
        .stdin(Stdio::piped())
        .spawn();

    match (wl_copy_cmd, xclip_cmd) {
        (Err(wl_copy_err), Err(xclip_err)) => {
            return Err(format_err!(
                "Both wl-copy & xclip failed to copy, wl-copy error: {} | xclip error: {}",
                wl_copy_err,
                xclip_err
            ));
        }
        (Ok(mut wl_copy), _) => {
            // NOTE: We get to do all this reading and writing from the files because wl-copy only accepts
            // files on STDIN
            let file_content = std::fs::read(temp_file_path)
                .map_err(|e| format_err!("Unable to open {}, error: {}", temp_file_path, e))?;
            wl_copy
                .stdin
                .take()
                .expect("Failed to open stdin")
                .write_all(&file_content[..])
                .map_err(|e| format_err!("Unable to write to stdin of wl-copy, error: {}", e))?;
        }
        _ => (),
    }

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