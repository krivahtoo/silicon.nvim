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
