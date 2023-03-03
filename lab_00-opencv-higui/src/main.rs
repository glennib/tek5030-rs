use common::util::MatMovingAverage;
use opencv::{
    core::{self, Mat, MatTraitConstManual, Size},
    highgui,
    imgproc::{self},
    videoio::{self, VideoCaptureTrait, VideoCaptureTraitConst},
};

const FREQUENCY: f64 = 60.;
#[allow(clippy::cast_possible_truncation)]
const WAIT_MS: i32 = ((1.0_f64 / FREQUENCY) * 1000.) as i32;

fn main() -> opencv::Result<()> {
    let camera_index = 0;
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_GUI_NORMAL)?;
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
    let opened = cam.is_opened()?;
    assert!(opened, "Could not open camera at index {camera_index}");

    let mut read_frame = Mat::default().clone();
    let mut buffer = MatMovingAverage::new(5);
    loop {
        cam.read(&mut read_frame)?;
        if read_frame.size()?.width > 0 {
            buffer.push(read_frame.clone());
            let mut frame = buffer
                .average()
                .expect("should have an image after pushing just before");
            frame = {
                let mut result = Mat::default();
                imgproc::gaussian_blur(
                    &frame,
                    &mut result,
                    Size::new(5, 5),
                    4.,
                    4.,
                    core::BORDER_DEFAULT,
                )?;
                result
            };

            frame = {
                let mut result = Mat::default();
                imgproc::bilateral_filter(
                    &frame,
                    &mut result,
                    10,
                    100.,
                    100.,
                    core::BORDER_DEFAULT,
                )?;
                result
            };

            frame = {
                let mut result = Mat::default();
                imgproc::canny(&frame, &mut result, 10., 15., 3, true)?;
                result
            };

            frame = {
                let mut result = Mat::default();
                core::flip(&frame, &mut result, 1).expect("should be able to flip");
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
