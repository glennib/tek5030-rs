use eframe::egui::{ImageData, Key};
use eframe::egui::{Separator, Widget};
use eframe::{
    egui::{self, CentralPanel, ComboBox, Context, SidePanel, Slider, TextureOptions},
    App, Frame,
};
use image::imageops::{self, FilterType};
use image::{GrayImage, RgbImage};
use lab_00_native_egui::{create_camera_stream, MyImageData};
use nokhwa::utils::CameraIndex;
use std::sync::{mpsc, Arc, RwLock};

#[derive(Debug, Clone)]
struct ImageProcessingConfiguration {
    gray_before_scale: bool,
    scale: u32,
    scale_filter: FilterType,
    blur: f32,
    canny_lo: f32,
    canny_hi: f32,
}

impl ImageProcessingConfiguration {
    #[allow(clippy::needless_pass_by_value)] // to conform with interface
    fn call(&self, image: RgbImage) -> GrayImage {
        let image = if self.gray_before_scale {
            let image = imageops::grayscale(&image);
            imageops::resize(
                &image,
                image.width() / self.scale,
                image.height() / self.scale,
                self.scale_filter,
            )
        } else {
            let image = imageops::resize(
                &image,
                image.width() / self.scale,
                image.height() / self.scale,
                self.scale_filter,
            );
            imageops::grayscale(&image)
        };
        let image = imageproc::filter::gaussian_blur_f32(&image, self.blur);
        let image = imageproc::edges::canny(&image, self.canny_lo, self.canny_hi);

        #[allow(clippy::let_and_return)] // to easily add operations
        image
    }
}

impl Default for ImageProcessingConfiguration {
    fn default() -> Self {
        Self {
            gray_before_scale: false,
            scale: 4,
            scale_filter: FilterType::Nearest,
            blur: 4.,
            canny_lo: 5.,
            canny_hi: 15.,
        }
    }
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800., 600.)),
        ..Default::default()
    };

    let processor = Arc::new(RwLock::new(ImageProcessingConfiguration::default()));

    let stream_receiver = create_camera_stream(CameraIndex::Index(0), {
        let processor = processor.clone();
        move |img| processor.read().unwrap().call(img)
    });

    let stream = {
        move || match stream_receiver.try_recv() {
            Ok(img) => Some(img),
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("stream has no updater");
            }
            _ => None,
        }
    };

    let app = MyApp::new(stream, processor);

    eframe::run_native("lab 00", options, Box::new(|_cc| Box::new(app)))
        .expect("should be able to run app");
}

struct MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, f64)>,
{
    // option_updater: Updater<O>,
    image_stream: ImageStreamFn,
    latest_image: Option<ImageData>,
    latest_fps: f64,
    image_processing_configuration: Arc<RwLock<ImageProcessingConfiguration>>,
}

impl<ImageStreamFn, ToImageData> MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, f64)>,
{
    fn new(
        image_stream: ImageStreamFn,
        image_processing_configuration: Arc<RwLock<ImageProcessingConfiguration>>,
    ) -> Self {
        Self {
            image_stream,
            latest_image: None,
            latest_fps: 0.,
            image_processing_configuration,
        }
    }
}

impl<ImageStreamFn, ToImageData> App for MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, f64)>,
{
    fn update(&mut self, ctx: &Context, epi_frame: &mut Frame) {
        let (rgb, fps) =
            (self.image_stream)().map_or_else(|| (None, None), |(rgb, fps)| (Some(rgb), Some(fps)));

        let mut configuration = self.image_processing_configuration.read().unwrap().clone();

        let mut changed = false;

        SidePanel::left("Configure").show(ctx, |sidebar| {
            sidebar.spacing_mut().item_spacing.y = 10.;

            changed |= sidebar
                .checkbox(&mut configuration.gray_before_scale, "gray before scale")
                .changed();

            Separator::default().ui(sidebar);

            let slider = Slider::new(&mut configuration.scale, 1..=8)
                .step_by(1.)
                .text("scale");
            changed |= sidebar.add(slider).changed();
            changed |= ComboBox::from_label("filter")
                .selected_text(format!("{:?}", configuration.scale_filter))
                .show_ui(sidebar, |ui| {
                    for value in [
                        FilterType::Nearest,
                        FilterType::CatmullRom,
                        FilterType::Gaussian,
                        FilterType::Lanczos3,
                        FilterType::Triangle,
                    ] {
                        changed |= ui
                            .selectable_value(
                                &mut configuration.scale_filter,
                                value,
                                format!("{value:?}"),
                            )
                            .changed();
                    }
                })
                .response
                .clicked();

            Separator::default().ui(sidebar);

            let slider = Slider::new(&mut configuration.blur, 1.0..=20.)
                .step_by(1.)
                .text("blur");
            changed |= sidebar.add(slider).changed();

            let slider = Slider::new(&mut configuration.canny_lo, 1.0..=configuration.canny_hi)
                .step_by(1.)
                .text("canny lo");
            changed |= sidebar.add(slider).changed();

            let slider = Slider::new(&mut configuration.canny_hi, configuration.canny_lo..=30.)
                .step_by(1.)
                .text("canny hi");
            changed |= sidebar.add(slider).changed();

            if let Some(fps) = fps {
                self.latest_fps = fps;
            }

            Separator::default().ui(sidebar);
            sidebar.label(format!("{:.1}", self.latest_fps));
        });

        if changed {
            eprintln!("changing configuration to:\n{configuration:?}");
            self.image_processing_configuration
                .write()
                .unwrap()
                .clone_from(&configuration);
        }

        if let Some(image) = rgb {
            self.latest_image = Some(image.into().0);
        }

        CentralPanel::default().show(ctx, |image_draw_area| match &self.latest_image {
            Some(image) => {
                let tex = image_draw_area.ctx().load_texture(
                    "frame",
                    image.clone(),
                    TextureOptions::LINEAR,
                );
                image_draw_area.image(&tex, image_draw_area.available_size());
            }
            None => {
                image_draw_area.colored_label(
                    image_draw_area.visuals().error_fg_color,
                    "image stream returned nothing",
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
