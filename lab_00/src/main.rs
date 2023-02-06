use opencv::core::Size;
use opencv::imgproc::{resize, INTER_NEAREST};
use opencv::prelude::*;
use opencv::{highgui, videoio};

#[allow(clippy::cast_possible_truncation)]
const WAIT_MS: i32 = ((1.0_f64 / 60.0_f64) * 1000.) as i32;

fn main() -> opencv::Result<()> {
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_GUI_NORMAL)?;
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = cam.is_opened()?;
    assert!(opened, "Could not open default camera");

    let mut read_frame = Mat::default();
    let mut work_frame = Mat::default();
    loop {
        cam.read(&mut read_frame)?;
        if read_frame.size()?.width > 0 {
            resize(
                &read_frame,
                &mut work_frame,
                Size::default(),
                0.3,
                0.3,
                INTER_NEAREST,
            )?;
            highgui::imshow(window, &work_frame)?;
        }
        let key = highgui::wait_key(WAIT_MS)?;

        if key > 0 && key != 255 {
            break;
        }
    }

    Ok(())
}
