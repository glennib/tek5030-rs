use eframe::egui::{ColorImage, ImageData};
use image::RgbImage;
use opencv::core::{Mat, MatTraitConst, MatTraitConstManual};
use opencv::imgproc::{self, cvt_color};

pub struct MyImageData(pub ImageData);

impl From<Mat> for MyImageData {
    fn from(value: Mat) -> Self {
        // Set depth
        let value = match value.depth() {
            opencv::core::CV_8U => value,
            d => panic!("unsupported depth {d}"),
        };
        // CV_8U here

        // Set color
        let value = match value.channels() {
            1 => {
                let mut out = Mat::default();
                cvt_color(&value, &mut out, imgproc::COLOR_GRAY2RGB, 3)
                    .expect("should be able to convert grayscale to rgb");
                out
            }
            3 => value,
            c => {
                panic!("unsupported number of channels {c}")
            }
        };

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

/// Turns an `image::RgbImage` into an `opencv::core::Mat`
///
/// # Arguments
///
/// * `image`: `RgbImage`
///
/// returns: Result<Mat, Error>
///
/// # Errors
///
/// * `opencv::Error` if constructing or copying Mats fails.
pub fn to_mat(mut image: RgbImage) -> Result<Mat, opencv::Error> {
    let data = image.as_mut_ptr();
    let step = opencv::core::Mat_AUTO_STEP;
    let mat = unsafe {
        // SAFETY
        // The Mat from this block references the owned image, which is dropped at the end of the
        // function. Before the drop, the data pointed to by this Mat is cloned to an owning Mat,
        // which makes it safe.
        Mat::new_rows_cols_with_data(
            i32::try_from(image.height()).expect("image size should fit in an i32"),
            i32::try_from(image.width()).expect("image size should fit in an i32"),
            opencv::core::CV_8UC3,
            data.cast::<std::ffi::c_void>(),
            step,
        )?
    };
    let mut out = Mat::default();
    mat.copy_to(&mut out)?;
    Ok(out)
}
