
use error_iter::ErrorIter as _;
use fltk::{app, prelude::*, window::Window};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::{cell::RefCell, rc::Rc};
use std::time::Instant;

const WIDTH: u32 = 512 + 1;
const HEIGHT: u32 = 256;

mod graphics;

enum World {
    Sprites {
        bitmap: Box<[[(u8, u8, u8); 512]; 256]>,
        start_time: Instant,
    },
    Animation {
        fetch_color: graphics::FetchColor,
        last_frame: usize,
        start_time: Instant,
    },
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
        let bitmap = graphics::create_bitmap(&tiles, &tiles2);
        World::Sprites {
            bitmap,
            start_time: Instant::now(),
        }
    }

    fn transform(&mut self) {
        *self = World::Animation {
            fetch_color: graphics::FetchColor::new(),
            last_frame: 0,
            start_time: Instant::now(),
        };
    }

    /// Update the `World` internal state; bounce the circle around the screen.
    fn update(&mut self) {
        match self {
            World::Sprites { start_time, .. } => {
                if start_time.elapsed().as_secs() > 10 {
                    self.transform();
                }
            }
            World::Animation {
                    fetch_color, start_time, last_frame
                } => {
                    let duration = start_time.elapsed().as_millis();
                    let frame_count = duration / 100;
                    if frame_count > 7 {
                        return;
                    }
                    if frame_count > *last_frame as _ {
                        println!("Frame: {}", frame_count);
                        *last_frame = frame_count as _;
                        fetch_color.skip_to(*last_frame);
                    }
                }
        }
        


    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            if x >= 512 || y >= 256 {
                pixel.copy_from_slice(&[0, 0, 0, 0xFF]);
                continue;
            }

            match self {
                World::Sprites { bitmap, .. } => {
                    let rgb = bitmap[y][x];
                    pixel.copy_from_slice(&[rgb.0, rgb.1, rgb.2, 0xFF]);
                }
                World::Animation { fetch_color, .. } => {
                    if y >= 160 && x >= 256 && y + x / 2 > 356 {
                        pixel.copy_from_slice(&[0, 0, 0, 0xFF]);
                        continue;
                    }

                    if y >= 160 && x < 256 && y + x / 2 > 228 {
                        pixel.copy_from_slice(&[0, 0, 0, 0xFF]);
                        continue;
                    }

                    let color = fetch_color.get_color(x % 256, y).unwrap_or((0, 0, 0));
                    pixel.copy_from_slice(&[color.0, color.1, color.2, 0xFF]);
                }

            }

        }
    }
}
