pub mod cam {
    use image::{ImageBuffer, RgbImage};
    use nokhwa::{
        pixel_format::RgbFormat,
        utils::{CameraIndex, RequestedFormat, RequestedFormatType},
        Camera,
    };
    use std::sync::mpsc;
    use std::thread;
    use tracing::{debug, error};

    pub type VectorImageBuffer<P> = ImageBuffer<P, Vec<u8>>;

    #[must_use]
    pub fn create_camera_stream_identity(index: CameraIndex) -> mpsc::Receiver<RgbImage> {
        let process = |img| img;
        create_camera_stream::<_, RgbImage>(index, process)
    }

    #[must_use]
    pub fn create_camera_stream<F, I>(index: CameraIndex, process: F) -> mpsc::Receiver<I>
    where
        F: Fn(RgbImage) -> I + Sized + Send + 'static,
        I: Send + 'static,
    {
        let (img_sender, img_receiver) = mpsc::sync_channel(2);

        thread::spawn(move || {
            const ERROR_LIMIT: u32 = 30;
            let mut fails = 0;
            let mut camera = Camera::new(
                index,
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
            )
            .expect("expected a camera on received index");
            if !camera.is_stream_open() {
                camera
                    .open_stream()
                    .expect("should be able to open stream on camera");
            }
            {
                let info = camera.info();
                debug!(?info, "opened camera");
            }

            loop {
                match camera
                    .frame()
                    .and_then(|frame| frame.decode_image::<RgbFormat>())
                {
                    Ok(frame) => {
                        let frame = process(frame);
                        if img_sender.send(frame).is_err() {
                            debug!("image receiver dropped");
                            break;
                        }
                    }
                    Err(e) => {
                        fails += 1;
                        error!(%fails, %e);
                        if fails >= ERROR_LIMIT {
                            error!(ERROR_LIMIT, "exceeded error limit");
                            break;
                        }
                    }
                }
            }
            debug!("end of loop");
        });

        img_receiver
    }
}

pub mod convert {
    use image::{GrayImage, RgbImage};

    pub struct MyImageData(pub egui::ImageData);

    impl From<RgbImage> for MyImageData {
        fn from(value: RgbImage) -> Self {
            let image = egui::ColorImage::from_rgb(
                [value.width() as usize, value.height() as usize],
                value.as_ref(),
            );
            MyImageData(image.into())
        }
    }

    impl From<GrayImage> for MyImageData {
        fn from(value: GrayImage) -> Self {
            let image = value.expand_palette(&(*PALETTE), None);
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                image.as_ref(),
            );
            MyImageData(image.into())
        }
    }

    use lazy_static::lazy_static;

    lazy_static! {
        static ref PALETTE: [(u8, u8, u8); 256] = {
            let mut palette = [(0_u8, 0_u8, 0_u8); 256];
            for v in 0_u8..=255 {
                palette[v as usize] = (v, v, v);
            }
            palette
        };
    }
}
