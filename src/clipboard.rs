use std::{
    borrow::Cow,
    time::{Duration, Instant},
};

use image::{DynamicImage, RgbaImage};
use nvim_oxi::api::{self, opts::NotifyOpts, types::LogLevel};

pub fn dump_image_to_clipboard(image: RgbaImage) {
    // using a thread so as not to block neovim while holding
    // clipboard content long enough for the user to paste
    // when there is no clipboard manager.
    std::thread::spawn(move || {
        #[cfg(target_os = "linux")]
        use arboard::SetExtLinux;
        use arboard::{Clipboard, Error, ImageData};

        let mut ctx = match Clipboard::new() {
            Ok(ctx) => ctx,
            Err(Error::ClipboardNotSupported) => {
                api::notify(
                    "Clipboard not supported",
                    LogLevel::Error,
                    &NotifyOpts::default(),
                )
                .unwrap();
                return;
            }
            _ => {
                api::notify("Failed to copy", LogLevel::Warn, &NotifyOpts::default()).unwrap();
                return;
            }
        };
        let img_data = ImageData {
            width: image.width() as usize,
            height: image.height() as usize,
            bytes: Cow::from(image.as_raw()),
        };
        let set = ctx.set();
        #[cfg(target_os = "linux")]
        // wait for 45 secs
        let set = set.wait_until(Instant::now() + Duration::from_secs(45));

        api::notify(
            "Image saved to clipboard",
            LogLevel::Info,
            &NotifyOpts::default(),
        )
        .unwrap();

        if let Err(e) = set.image(img_data) {
            api::notify(
                &format!("Failed to copy to clipboard: {e}"),
                LogLevel::Error,
                &NotifyOpts::default(),
            )
            .unwrap();
        };
    });
}
