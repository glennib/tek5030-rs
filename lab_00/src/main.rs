use opencv::core::{self, Point, Size};
use opencv::imgproc::{self};
use opencv::prelude::*;
use opencv::{highgui, videoio};

const FREQUENCY: f64 = 60.;
#[allow(clippy::cast_possible_truncation)]
const WAIT_MS: i32 = ((1.0_f64 / FREQUENCY) * 1000.) as i32;

fn main() -> opencv::Result<()> {
    let camera_index = 4;
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_GUI_NORMAL)?;
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
    let opened = cam.is_opened()?;
    assert!(opened, "Could not open camera at index {camera_index}");

    let mut read_frame = Mat::default();
    loop {
        cam.read(&mut read_frame)?;
        if read_frame.size()?.width > 0 {
            let mut frame = {
                let mut result = Mat::default();
                imgproc::blur(
                    &read_frame,
                    &mut result,
                    Size::new(10, 10),
                    Point::new(-1, -1),
                    core::BORDER_DEFAULT,
                )?;
                result
            };

            frame = {
                let mut result = Mat::default();
                imgproc::canny(&frame, &mut result, 5., 15., 3, false)?;
                result
            };

            highgui::imshow(window, &frame)?;
        }
        let key = highgui::wait_key(WAIT_MS)?;

        if key > 0 && key != 255 {
            break;
        }
    }

    Ok(())
}
