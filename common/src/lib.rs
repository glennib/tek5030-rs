pub mod cam {
    use image::{ImageBuffer, RgbImage};
    pub use nokhwa::utils::CameraIndex;
    use nokhwa::{
        pixel_format::RgbFormat,
        utils::{RequestedFormat, RequestedFormatType},
        Camera,
    };
    use std::{sync::mpsc, thread};
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

pub mod util {
    use opencv::{
        core::{self, MatExprTraitConst, MatTraitConst, MatTraitConstManual},
        imgproc,
    };
    use std::collections::VecDeque;

    pub struct MatMovingAverage {
        size: usize,
        buffer: VecDeque<::opencv::core::Mat>,
    }

    impl MatMovingAverage {
        #[must_use]
        pub fn new(size: usize) -> Self {
            Self {
                size,
                buffer: VecDeque::with_capacity(size),
            }
        }

        pub fn push(&mut self, mat: ::opencv::core::Mat) {
            if self.buffer.len() == self.size {
                self.buffer.pop_front();
            }

            self.buffer.push_back(mat);
        }

        #[must_use]
        pub fn average(&self) -> Option<::opencv::core::Mat> {
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
            let mut out = ::opencv::core::Mat::zeros(size.height, size.width, core::CV_32FC3)
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
                let mut res = ::opencv::core::Mat::default();
                out.convert_to(&mut res, depth, 1., 0.)
                    .expect("should be able to convert");
                res
            };

            Some(out)
        }
    }
}
