
use error_iter::ErrorIter as _;
use fltk::{app, prelude::*, window::Window};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::{cell::RefCell, rc::Rc};
use std::time::Instant;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 600;

mod graphics;

struct World {
    tiles: Vec<u8>,
    tiles2: Vec<u8>,
    bg_tiles: graphics::TileMap,
    bitmap: Vec<u8>,
    last_frame: usize,
    start_time: Instant,
}

fn main() -> Result<(), Error> {
    #[cfg(debug_assertions)]
    env_logger::init();

    let app = app::App::default();
    let mut win = Window::default()
        .with_size(WIDTH as i32, HEIGHT as i32)
        .with_label("Hello Pixels");
    win.make_resizable(true);
    win.end();
    win.show();

    let mut world = World::new();

    // Handle resize events
    let surface_size = Rc::new(RefCell::new(None));
    let surface_resize = surface_size.clone();
    win.resize_callback(move |win, _x, _y, width, height| {
        let scale_factor = win.pixels_per_unit();
        let width = (width as f32 * scale_factor) as u32;
        let height = (height as f32 * scale_factor) as u32;

        surface_resize.borrow_mut().replace((width, height));
    });

    let mut pixels = {
        let pixel_width = win.pixel_w() as u32;
        let pixel_height = win.pixel_h() as u32;
        let surface_texture = SurfaceTexture::new(pixel_width, pixel_height, &win);

        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    while app.wait() {
        // Update internal state
        world.update();

        // Resize the window
        if let Some((width, height)) = surface_size.borrow_mut().take() {
            if let Err(err) = pixels.resize_surface(width, height) {
                log_error("pixels.resize_surface", err);
                app.quit();
            }
        }

        // Draw the current frame
        world.draw(pixels.frame_mut());
        if let Err(err) = pixels.render() {
            log_error("pixels.render", err);
            app.quit();
        }

        app::flush();
        app::awake();
    }

    Ok(())
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving circle.
    fn new() -> Self {

        let tiles = graphics::source("./graphics/spriteTiles.inc");
        let tiles2 = graphics::source("./graphics/spriteTiles2.inc");
        let bg_tiles = graphics::TileMap::new();
        let bitmap = graphics::create_bitmap(0, &tiles, &tiles2, &bg_tiles);
        Self {
            tiles,
            tiles2,
            bg_tiles,
            bitmap,
            last_frame: 0,
            start_time: Instant::now(),
        }
    }

    /// Update the `World` internal state; bounce the circle around the screen.
    fn update(&mut self) {
        let duration = self.start_time.elapsed().as_millis();
        let frame_count = duration / 100;
        if frame_count > self.last_frame as _ {
            println!("Frame: {}", frame_count);
            self.last_frame = frame_count as _;
            self.bitmap = graphics::create_bitmap(self.last_frame, &self.tiles, &self.tiles2, &self.bg_tiles);
        }


    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            if x >= 32 * 8 * 2 || y >= 32 * 8 * 2 {
                pixel.copy_from_slice(&[0, 0, 0, 0xFF]);
                continue;
            }

            let offset = x * 3 + y * 3 * 32 * 8 * 2;

            let slice = &self.bitmap[offset..offset + 3];

            pixel.copy_from_slice(&[slice[0], slice[1], slice[2], 0xFF]);
        }
    }
}
