use anyhow::Result;
use eframe::egui::{ColorImage, Context, ImageData, TextureOptions};
use eframe::{
    egui::{self, CentralPanel},
    App,
};
use image::{ImageBuffer, Rgb};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};
use simple_moving_average::SMA;

type ColorImageBuffer = ImageBuffer<Rgb<u8>, Vec<u8>>;

fn main() -> Result<()> {
    let (mut stream, _shutdown) = create_camera_streamer(CameraIndex::Index(0));

    let stream = move || stream.latest().clone();

    setup_gui(stream)
}

fn setup_gui<ImageStreamFn, ToImageData>(stream: ImageStreamFn) -> Result<()>
where
    ToImageData: Into<MyImageData> + Sized + 'static,
    ImageStreamFn: FnMut() -> Option<(ToImageData, Instant)> + 'static,
{
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800., 600.)),
        ..Default::default()
    };

    let app = MyApp::new(stream);

    eframe::run_native("Stream webcam", options, Box::new(|_| Box::new(app)))
        .expect("should be able to run eframe app");

    Ok(())
}

fn create_camera_streamer(
    index: CameraIndex,
) -> (
    single_value_channel::Receiver<Option<(ColorImageBuffer, Instant)>>,
    std::sync::mpsc::SyncSender<()>,
) {
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    let (img_receiver, img_updater) = single_value_channel::channel();
    let (shutdown_sender, shutdown_receiver) = std::sync::mpsc::sync_channel(1);

    std::thread::spawn(move || {
        const LIMIT: u32 = 1;
        let mut fails = 0;
        let mut camera = Camera::new(index, requested).unwrap();
        // let mut ma = simple_moving_average::SumTreeSMA::<_, f64, 10>::new();
        while let Err(RecvTimeoutError::Timeout) = shutdown_receiver.recv_timeout(Duration::ZERO) {
            // let begin = Instant::now();
            if let Ok(rgb) = camera
                .frame()
                .and_then(|frame| frame.decode_image::<RgbFormat>())
            {
                fails = 0;
                if img_updater.update(Some((rgb, Instant::now()))).is_err() {
                    break;
                }
            } else {
                fails += 1;
                if fails >= LIMIT {
                    break;
                }
            }
            // let duration = begin.elapsed();
            // ma.add_sample(duration.as_secs_f64());
            // eprint!("fps: {}\n", 1. / ma.get_average());
        }
    });

    (img_receiver, shutdown_sender)
}

struct MyImageData(ImageData);

impl From<ColorImageBuffer> for MyImageData {
    fn from(value: ColorImageBuffer) -> Self {
        let image = ColorImage::from_rgb(
            [value.width() as usize, value.height() as usize],
            value.as_ref(),
        );
        MyImageData(image.into())
    }
}

struct MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, Instant)>,
{
    image_stream: ImageStreamFn,
}

impl<ImageStreamFn, ToImageData> MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, Instant)>,
{
    fn new(image_stream: ImageStreamFn) -> Self {
        Self { image_stream }
    }
}

impl<ImageStreamFn, ToImageData> App for MyApp<ImageStreamFn, ToImageData>
where
    ToImageData: Into<MyImageData> + Sized,
    ImageStreamFn: FnMut() -> Option<(ToImageData, Instant)>,
{
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let rgb = (self.image_stream)();

        CentralPanel::default().show(ctx, |ui| match rgb {
            Some((image, stamp)) => {
                let image = image.into().0;
                let tex = ui
                    .ctx()
                    .load_texture("frame", image, TextureOptions::LINEAR);
                ui.image(&tex, ui.available_size());
                eprint!("age: {}\n", stamp.elapsed().as_millis());
            }
            None => {
                ui.colored_label(ui.visuals().error_fg_color, "image stream returned nothing");
            }
        });
    }
}
