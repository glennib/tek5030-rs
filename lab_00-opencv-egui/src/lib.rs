use eframe::egui::{ColorImage, ImageData};
use opencv::core::{Mat, MatTraitConst, MatTraitConstManual};
use opencv::imgproc::{self, cvt_color};

pub struct MyImageData(pub ImageData);

// fn debug_print_mat(name: &str, value: &Mat) {
//     eprintln!("{name}: {{");
//     eprintln!("\t size: {:?}", value.size().unwrap());
//     eprintln!("\t depth: {:?}", value.depth());
//     eprintln!("\t channels: {:?}", value.channels());
//     eprintln!("}}");
// }

impl From<Mat> for MyImageData {
    fn from(value: Mat) -> Self {
        // debug_print_mat("in", &value);
        // Set depth
        let value = match value.depth() {
            opencv::core::CV_8U => value,
            d => panic!("unsupported depth {d}"),
        };
        // CV_8U here
        // debug_print_mat("CV_8U", &value);

        // Set color
        let value = match value.channels() {
            1 => {
                let mut out = Mat::default();
                // debug_print_mat("allocated for color conversion", &out);
                cvt_color(&value, &mut out, imgproc::COLOR_GRAY2RGB, 3)
                    .expect("should be able to convert grayscale to rgb");
                out
            }
            3 => value,
            c => {
                panic!("unsupported number of channels {c}")
            }
        };
        // debug_print_mat("converted to rgb", &value);

        let size = value.size().expect("size should be available");
        let size = [
            usize::try_from(size.width).expect("size should be nonnegative"),
            usize::try_from(size.height).expect("size should be nonnegative"),
        ];
        let out = ColorImage::from_rgb(
            size,
            value.data_bytes().expect("data bytes should be available"),
        );
        MyImageData(out.into())
    }
}
