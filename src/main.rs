use eframe::{epi, egui};
use eframe::egui::{ColorImage, Color32};
use eframe::egui::widgets::color_picker;
use egui_extras::RetainedImage;
use arboard::Clipboard;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;

struct MyEguiApp {
  img_width: f32,
  image_data: ColorImage,
  current_color: Color32,
  image: RetainedImage,
  minimap: RetainedImage,
}

impl MyEguiApp {
  fn paste_image(&mut self) {
    let mut clipboard = Clipboard::new().unwrap();
    
    match clipboard.get_image() {
      Ok(img) => {
        let result = ColorImage::from_rgba_unmultiplied(
          [img.width, img.height],
          &img.bytes
        );

        self.image_data = result.clone();

        self.image = RetainedImage::from_color_image(
          "clipboard",
          result,
        );
      },
      Err(error) => {
        println!("Clipboard error: {}", error);
      },
    }
  }

  fn screen_capture(&mut self) {
    let (width, height, data) = screenshot();

    let result = ColorImage::from_rgba_unmultiplied(
      [width, height],
      &data
    );

    self.image_data = result.clone();

    self.image = RetainedImage::from_color_image(
      "screencapture",
      result,
    );
  }
}

impl Default for MyEguiApp {
  fn default() -> Self {
    Self {
      img_width: 300.0,
      image_data: ColorImage::example(),
      current_color: Color32::BLACK,
      image: RetainedImage::from_color_image(
        "example",
        ColorImage::example()
      ),
      minimap: RetainedImage::from_color_image(
        "minimap",
        ColorImage::new([100 as usize, 100 as usize], Color32::BLACK)
      ),
    }
  }
}

impl epi::App for MyEguiApp {
  fn name(&self) -> &str {
    "My egui App"
  }

  fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading("MY COLOR PICKER");

      let mut img_size = self.image.size_vec2();
      let img_size_hint = img_size[1] / img_size[0];

      img_size[0] = self.img_width;
      img_size[1] = img_size[0] * img_size_hint;

      ui.add(egui::Slider::new(&mut self.img_width, 10.0..=500.0).text("width"));

      ui.label(format!("Image Size {}x{}", img_size[0], img_size[1]));

      ui.label(format!("Color: ({}, {}, {}, {})",
        self.current_color.r(),
        self.current_color.g(),
        self.current_color.b(),
        self.current_color.a())
      );

      if ui.button("Get Image from Clipboard").clicked() {
        self.paste_image();
      }

      if ui.button("Screen capture").clicked() {
        self.screen_capture();
      }

      
      let color_data = ColorImage::new([100 as usize, 100 as usize], self.current_color);
      let color_img = RetainedImage::from_color_image("minimap", color_data);
      ui.add(egui::Image::new(color_img.texture_id(ctx), color_img.size_vec2()));

      // FIXME:
      // let mut colorpicker = Color32::TRANSPARENT;
      // color_picker::color_picker_color32(ui, &mut colorpicker, egui::widgets::color_picker::Alpha::Opaque);

      // FIXME:
      let viewer = ui.add(egui::Image::new(self.image.texture_id(ctx), img_size));
      // let viewer = ui.add(egui::Image::new(self.image.texture_id(ctx), self.image.size_vec2()));

      let viewer_pos = viewer.rect.left_top();
      let viewer_width = viewer.rect.width();
      let viewer_height = viewer.rect.height();
      
      let mouse = ctx.input().pointer.press_origin();

      if let Some(mouse) = mouse {
        if mouse.x >= viewer_pos.x && mouse.x <= viewer_pos.x + viewer_width &&
           mouse.y >= viewer_pos.y && mouse.y <= viewer_pos.y + viewer_height {

            // Get image origin size
            let size = self.image.size_vec2();
            let origin_size_hint = size[0] / img_size[0];
            // Get mouse position in window
            let x = ((viewer_pos.x - mouse.x).abs() * origin_size_hint).floor();
            let y = ((viewer_pos.y - mouse.y).abs() * origin_size_hint).floor();

            // NOTE: DEBUG MESSAGE
            println!("({}, {}), ({})", x, y, origin_size_hint);

            // Set pixel index
            let index: usize = (y * size[0] + x) as usize;
            // Set index limit
            let limit: usize = (size[0] * size[1]) as usize;

            if index < 0 as usize || index > limit {
              // ERROR
              println!("index is over: {}", index);
              
            } else {
              // OK
              println!("index: {}", index);
              self.current_color = self.image_data.pixels[index];

              // TODO: Update minimap
            }
          }
      }
    });

    // Resize the native window to be just the size we need it to be:
    frame.set_window_size(ctx.used_size());
  }
}

fn screenshot() -> (usize, usize, Vec<u8>) {
  let display = Display::primary().expect("Couldn't find primary display.");
  let mut capturer = Capturer::new(display).expect("Couln't begin capture");
  let (width, height) = (capturer.width(), capturer.height());

  let one_second = Duration::new(1, 0);
  let one_frame = one_second / 60;

  loop {
    let buffer = match capturer.frame() {
      Ok(data) => data,
      Err(error) => {
        if error.kind() == WouldBlock {
          thread::sleep(one_frame);
          continue;
        } else {
          panic!("Error: {}", error);
        }
      }
    };

    let stride = buffer.len() / height;
    let mut result = Vec::with_capacity(width * height * 4);

    // Flip the ARGB image into a BGRA image.
    for y in 0..height {
      for x in 0..width {
        let i = stride * y + 4 * x;
        result.extend_from_slice(&[
          buffer[i + 2],
          buffer[i + 1],
          buffer[i],
          255,
        ]);
      }
    }

    return (width, height, result);
  }
}

fn main() {
  let app = MyEguiApp::default();
  let options = eframe::NativeOptions {
    decorated: true,
    transparent: true,
    min_window_size: Some(egui::vec2(100.0, 200.0)),
    ..Default::default()
  };
  
  eframe::run_native(Box::new(app), options);
}
