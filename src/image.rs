use std::process::id;
use eframe::{egui, epi};
use eframe::egui::Vec2;
use image::GenericImageView;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GrayImage, RgbImage};


#[derive(Default, Clone)]
pub struct MyApp {
    texture: Option<(egui::Vec2, egui::TextureId)>,
    id: String
}

impl epi::App for MyApp {
    fn name(&self) -> &str {
        "Pagina Trovata"
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if self.texture.is_none() {
            // Load the image:

            println!("{}", self.id);

            let image = ImageReader::open(self.id.clone()).unwrap().decode().expect("Failed to load image");
            let image_buffer = image.to_rgba8();
            let size = (image.width() as usize, image.height() as usize);
            let pixels = image_buffer.into_vec();
            assert_eq!(size.0 * size.1 * 4, pixels.len());
            let pixels: Vec<_> = pixels
                .chunks_exact(4)
                .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
                .collect();

            // Allocate a texture:
            let texture = frame
                .tex_allocator()
                .alloc_srgba_premultiplied(size, &pixels);
            let size = egui::Vec2::new(size.0 as f32, size.1 as f32);
            self.texture = Some((size, texture));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some((size, texture)) = self.texture {
                ui.image(texture, size);
            }
        });
    }
}


pub fn openImage(pathImg: String) {

    let img = ImageReader::open(pathImg.clone()).unwrap().decode().expect("Failed to load image");

    let options = eframe::NativeOptions {
        always_on_top: false,
        maximized: false,
        decorated: true,
        drag_and_drop_support: true,
        icon_data: None,
        initial_window_size: Option::from(Vec2::new( img.width() as f32, img.height() as f32)),
        resizable: true,
        transparent: true,
    };

    let appImage = MyApp { id: pathImg.clone().to_string(), ..Default::default() };

    eframe::run_native(Box::new(appImage), options);
}