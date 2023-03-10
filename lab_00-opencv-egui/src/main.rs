use anyhow::Result;
use common::cam::{create_camera_stream, CameraIndex};
use eframe::{
    egui::{self, CentralPanel, Context, ImageData, Key, SidePanel, Slider, TextureOptions},
    App, Frame,
};
use image::RgbImage;
use lab_00_opencv_egui::{to_mat, MyImageData};
use opencv::{
    core::{Mat, Size},
    imgproc::{self},
};
use std::sync::{mpsc::TryRecvError, Arc, RwLock};

#[derive(Debug, Clone)]
struct ImageProcessingConfiguration {
    blur: bool,
    blur_sigma: f64,
    canny: bool,
    canny_low: f64,
    canny_high: f64,
}

impl Default for ImageProcessingConfiguration {
    fn default() -> Self {
        Self {
            blur: false,
            blur_sigma: 1.,
            canny: false,
            canny_low: 10.,
            canny_high: 15.,
        }
    }
}

impl ImageProcessingConfiguration {
    /// Draws the parameter configuration GUI elements on the provided ui element, and returns
    /// Some(Self) if the user changed the options.
    ///
    /// # Arguments
    ///
    /// * `ui`: egui element to draw on
    ///
    /// returns: Option<ImageProcessingConfiguration>
    fn draw(&self, ui: &mut egui::Ui) -> Option<Self> {
        let mut configuration = self.clone();
        let mut changed = false;

        ui.spacing_mut().item_spacing.y = 10.;

        changed |= ui.checkbox(&mut configuration.blur, "blur").changed();

        if configuration.blur {
            changed |= ui
                .add(
                    Slider::new(&mut configuration.blur_sigma, 1.0..=10.)
                        .step_by(1.)
                        .text("sigma"),
                )
                .changed();
        }

        changed |= ui.checkbox(&mut configuration.canny, "canny").changed();

        if configuration.canny {
            changed |= ui
                .add(
                    Slider::new(&mut configuration.canny_low, 1.0..=configuration.canny_high)
                        .step_by(0.5)
                        .text("low"),
                )
                .changed();
            changed |= ui
                .add(
                    Slider::new(&mut configuration.canny_high, configuration.canny_low..=50.)
                        .step_by(0.5)
                        .text("high"),
                )
                .changed();
        }

        changed.then_some(configuration)
    }

    /// Processing pipeline which converts an `image::RgbImage` to an `egui::ImageData`.
    fn process(&self, image: RgbImage) -> Result<ImageData> {
        let mat = to_mat(image).expect("RgbImage should be convertible to Mat");

        // Do processing here

        let mat = if self.blur {
            let mut out = Mat::default();
            imgproc::gaussian_blur(
                &mat,
                &mut out,
                Size::new(0, 0),
                self.blur_sigma,
                self.blur_sigma,
                opencv::core::BORDER_REFLECT,
            )?;
            out
        } else {
            mat
        };

        let mat = if self.canny {
            let mut out = Mat::default();
            imgproc::canny(&mat, &mut out, self.canny_low, self.canny_high, 3, false)?;
            out
        } else {
            mat
        };

        // convert to image data here
        Ok(MyImageData::from(mat).0)
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800., 600.)),

        ..Default::default()
    };
    let processor = Arc::new(RwLock::new(ImageProcessingConfiguration::default()));

    let camera_stream_receiver = create_camera_stream(CameraIndex::Index(4), {
        let processor = processor.clone();
        move |img| processor.read().unwrap().process(img).ok()
    });

    let stream = {
        move || match camera_stream_receiver.try_recv() {
            Ok(img) => img,
            Err(TryRecvError::Disconnected) => {
                panic!("stream has no updater")
            }
            _ => None,
        }
    };

    let app = MyApp::new(stream, processor);

    eframe::run_native("lab 00", options, Box::new(|_cc| Box::new(app))).unwrap();
}

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

        SidePanel::left("Configure").show(ctx, |sidebar| {
            let changed_configuration = self
                .image_processing_configuration
                .read()
                .unwrap()
                .draw(sidebar);
            if let Some(configuration) = changed_configuration {
                self.image_processing_configuration
                    .write()
                    .unwrap()
                    .clone_from(&configuration);
            }
        });

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
