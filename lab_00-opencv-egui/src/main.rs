use common::cam::{create_camera_stream, CameraIndex};
use eframe::egui::{CentralPanel, Context, ImageData, Key, SidePanel, Slider, TextureOptions};
use eframe::{egui, App, Frame};
use image::RgbImage;
use lab_00_opencv_egui::MyImageData;
use opencv::core::{Mat, MatTraitConst};
use std::sync::mpsc::TryRecvError;
use std::sync::{Arc, RwLock};

struct MyApp<ImageStreamFn>
where
    ImageStreamFn: FnMut() -> Option<ImageData>,
{
    image_stream: ImageStreamFn,
    latest_image: Option<ImageData>,
    image_processing_configuration: Arc<RwLock<ImageProcessingConfiguration>>,
}

impl<ImageStreamFn> MyApp<ImageStreamFn>
where
    ImageStreamFn: FnMut() -> Option<ImageData>,
{
    fn new(
        image_stream: ImageStreamFn,
        image_processing_configuration: Arc<RwLock<ImageProcessingConfiguration>>,
    ) -> Self {
        Self {
            image_stream,
            latest_image: None,
            image_processing_configuration,
        }
    }
}

impl<ImageStreamFn> App for MyApp<ImageStreamFn>
where
    ImageStreamFn: FnMut() -> Option<ImageData>,
{
    fn update(&mut self, ctx: &Context, epi_frame: &mut Frame) {
        if let Some(image) = (self.image_stream)() {
            self.latest_image = Some(image);
        }

        let mut configuration = self.image_processing_configuration.read().unwrap().clone();

        let mut configuration_changed = false;

        SidePanel::left("Configure").show(ctx, |sidebar| {
            sidebar.spacing_mut().item_spacing.y = 10.;

            configuration_changed |= sidebar
                .add(
                    Slider::new(&mut configuration.blur, 1.0..=10.0)
                        .step_by(1.)
                        .text("blur"),
                )
                .changed();

            configuration_changed |= sidebar
                .add(
                    Slider::new(&mut configuration.canny_low, 1.0..=configuration.canny_high)
                        .step_by(0.5)
                        .text("canny low"),
                )
                .changed();
            configuration_changed |= sidebar
                .add(
                    Slider::new(&mut configuration.canny_high, configuration.canny_low..=50.)
                        .step_by(0.5)
                        .text("canny high"),
                )
                .changed();
        });

        if configuration_changed {
            self.image_processing_configuration
                .write()
                .unwrap()
                .clone_from(&configuration);
        }

        CentralPanel::default().show(ctx, |image_draw_area| {
            if let Some(ref image) = self.latest_image {
                let texture = image_draw_area.ctx().load_texture(
                    "frame",
                    image.clone(),
                    TextureOptions::LINEAR,
                );
                image_draw_area.image(&texture, image_draw_area.available_size());
            } else {
                image_draw_area.colored_label(
                    image_draw_area.visuals().error_fg_color,
                    "no image received from processing pipeline",
                );
            }
        });

        if ctx.input(|i| {
            [Key::Q, Key::Escape]
                .into_iter()
                .any(|key| i.key_pressed(key))
        }) {
            epi_frame.close();
        }

        ctx.request_repaint();
    }
}

#[derive(Debug, Clone)]
struct ImageProcessingConfiguration {
    blur: f64,
    canny_low: f64,
    canny_high: f64,
}

impl Default for ImageProcessingConfiguration {
    fn default() -> Self {
        Self {
            blur: 1.,
            canny_low: 10.,
            canny_high: 15.,
        }
    }
}

/// Turns an `image::RgbImage` into an `opencv::core::Mat`
///
/// # Arguments
///
/// * `image`: RgbImage
///
/// returns: Result<Mat, Error>
///
/// # Errors
///
/// * `opencv::Error` if constructing or copying Mats fails.
///
/// ```
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

impl ImageProcessingConfiguration {
    fn call(&self, image: RgbImage) -> ImageData {
        let mat = to_mat(image).expect("RgbImage should be convertible to Mat");

        // Do processing here
        let mat = {
            let mut out = Mat::default();
            opencv::imgproc::gaussian_blur(
                &mat,
                &mut out,
                opencv::core::Size::new(0, 0),
                f64::from(self.blur),
                f64::from(self.blur),
                opencv::core::BORDER_REFLECT,
            )
            .unwrap();
            out
        };

        let mat = {
            let mut out = Mat::default();
            opencv::imgproc::canny(&mat, &mut out, self.canny_low, self.canny_high, 3, false)
                .unwrap();
            out
        };

        // convert to image data here
        MyImageData::from(mat).0
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800., 600.)),
        ..Default::default()
    };
    let processor = Arc::new(RwLock::new(ImageProcessingConfiguration::default()));

    let camera_stream_receiver = create_camera_stream(CameraIndex::Index(0), {
        let processor = processor.clone();
        move |img| processor.read().unwrap().call(img)
    });

    let stream = {
        move || match camera_stream_receiver.try_recv() {
            Ok(img) => Some(img),
            Err(TryRecvError::Disconnected) => {
                panic!("stream has no updater")
            }
            _ => None,
        }
    };

    let app = MyApp::new(stream, processor);

    eframe::run_native("lab 00", options, Box::new(|_cc| Box::new(app))).unwrap();
}
