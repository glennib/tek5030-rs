use opencv::core::{self, Size};
use opencv::imgproc::{self};
use opencv::prelude::*;
use opencv::{highgui, videoio};
use std::collections::VecDeque;

const FREQUENCY: f64 = 60.;
#[allow(clippy::cast_possible_truncation)]
const WAIT_MS: i32 = ((1.0_f64 / FREQUENCY) * 1000.) as i32;

struct ImageBuffer {
    size: usize,
    buffer: VecDeque<Mat>,
}

impl ImageBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            buffer: VecDeque::with_capacity(size),
        }
    }

    pub fn push(&mut self, mat: Mat) {
        if self.buffer.len() == self.size {
            self.buffer.pop_front();
        }

        self.buffer.push_back(mat);
    }

    pub fn average(&self) -> Option<Mat> {
        if self.buffer.is_empty() {
            return None;
        }
        let an_image = self
            .buffer
            .get(0)
            .expect("should be an element at index 0 after checking emptiness prior");
        let size = an_image
            .size()
            .expect("should be able to check size on image");
        let depth = an_image.typ() & core::Mat_DEPTH_MASK;
        let mut out = Mat::zeros(size.height, size.width, core::CV_32FC3)
            .expect("should be able to create matrix")
            .to_mat()
            .expect("should be able to convert to matrix");
        for img in &self.buffer {
            imgproc::accumulate(img, &mut out, &core::no_array())
                .expect("should be able to accumulate");
        }

        #[allow(clippy::cast_precision_loss)]
        let mut out = (out / (self.buffer.len() as f64))
            .into_result()
            .expect("should be able to divide")
            .to_mat()
            .expect("should be able to convert to matrix");

        out = {
            let mut res = Mat::default();
            out.convert_to(&mut res, depth, 1., 0.)
                .expect("should be able to convert");
            res
        };

        Some(out)
    }
}

fn main() -> opencv::Result<()> {
    let camera_index = 4;
    let window = "video capture";
    highgui::named_window(window, highgui::WINDOW_GUI_NORMAL)?;
    let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)?;
    let opened = cam.is_opened()?;
    assert!(opened, "Could not open camera at index {camera_index}");

    let mut read_frame = Mat::default();
    let mut buffer = ImageBuffer::new(5);
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
