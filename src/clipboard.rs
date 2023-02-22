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

    let xclip_cmd = Command::new("xclip")
        .args([
            "-sel",
            "clip",
            "-t",
            "image/png",
            temp.path().to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e));

    let wl_copy_cmd = Command::new("wl-copy")
        .args(["-t", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format_err!("Failed to copy image to clipboard: {}", e));

    if wl_copy_cmd.is_err() && xclip_cmd.is_err() {
        let wl_copy_err = wl_copy_cmd.unwrap_err();
        let xclip_err = xclip_cmd.unwrap_err();
        let combined_err = format_err!(
            "Both wl-copy & xclip failed to copy:\nwl-copy error:{}\nxclip error:{}",
            &wl_copy_err.to_string(),
            &xclip_err.to_string()
        );
        return Err(combined_err);
    }

    if wl_copy_cmd.is_ok() {
        // NOTE: We get to do all this reading and writing from the files because wl-copy only accepts
        // files on STDIN
        let mut stdin = wl_copy_cmd?.stdin.take().expect("Failed to open stdin");
        let temp_file_path = temp.path().to_str().unwrap();
        let file_content =
            std::fs::read(temp_file_path).map_err(|e| format_err!("Unable to open {}, error: {}", temp_file_path, e))?;
        stdin
            .write_all(&file_content[..])
            .map_err(|e| format_err!("Unable to write to stdin of wl-copy, error: {}", e))?;
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
