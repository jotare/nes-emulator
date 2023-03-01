use std::sync::Arc;
use std::sync::RwLock;
/// GTK-4 UI
///
/// User Interface built on top of GTK-4 library
use std::thread::{spawn, JoinHandle};

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, Application, ApplicationWindow};
use log::debug;
use once_cell::sync::OnceCell;

use crate::graphics::ui::*;

const APP_ID: &str = "jotare-nes-emulator";
const APP_NAME: &str = "NES Emulator (by jotare)";

static RENDER_SIGNALER: OnceCell<Arc<RwLock<RenderSignaler>>> = OnceCell::new();

pub struct GtkUi {
    handle: Option<JoinHandle<()>>,
}

impl GtkUi {
    pub fn new() -> Self {
        Self { handle: None }
    }

    /// Starts GtkUi a running GUI. It should only be called once
    /// during the whole program. If called more than once, it'll panic.
    pub fn start(&mut self) {
        let already_initialized = RENDER_SIGNALER
            .set(Arc::new(RwLock::new(RenderSignaler::default())))
            .is_err();

        if already_initialized {
            panic!("GtkUi should be initialized only once!");
        }

        let join_handle = spawn(|| {
            let app = Application::builder().application_id(APP_ID).build();

            app.connect_activate(|app| {
                // Create main window
                let window = ApplicationWindow::builder()
                    .application(app)
                    .title(APP_NAME)
                    .build();

                // Screen
                let paintable = NesScreen::new();
                let picture = gtk::Picture::builder()
                    .width_request(SCREEN_WIDTH as i32)
                    .height_request(SCREEN_HEIGHT as i32)
                    .halign(gtk::Align::Center)
                    .valign(gtk::Align::Center)
                    .paintable(&paintable)
                    .build();
                window.set_child(Some(&picture));

                // Signal a re-render every time we have a new frame to paint
                picture.add_tick_callback(|area, _clock| {
                    let signaler = RENDER_SIGNALER.get().unwrap().write().unwrap();
                    if signaler.should_render() {
                        area.queue_draw();
                    }

                    Continue(true)
                });

                // Present window
                window.present();
            });

            app.run();
        });

        self.handle.replace(join_handle);
    }

    pub fn join(&mut self) {
        let handle = self
            .handle
            .take()
            .expect("Can't join an uninitialized GtkUi");
        debug!("Waiting UI thread to end...");
        handle.join().expect("Error waiting UI thread");
        debug!("UI thread ended correctly");
    }

    pub fn render(&self, frame: Frame) {
        let mut writer = RENDER_SIGNALER.get().unwrap().write().unwrap();
        writer.set_frame(frame);
    }
}

impl Ui for GtkUi {}

struct RenderSignaler {
    frame: Option<Frame>,
}

impl RenderSignaler {
    pub fn new() -> Self {
        Self { frame: None }
    }

    pub fn should_render(&self) -> bool {
        self.frame.is_some()
    }

    pub fn set_frame(&mut self, frame: Frame) {
        self.frame.replace(frame);
    }
}

impl Default for RenderSignaler {
    fn default() -> Self {
        Self::new()
    }
}

glib::wrapper! {
    struct NesScreen(ObjectSubclass<PaintableScreen>) @implements gdk::Paintable;

}

impl NesScreen {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

impl Default for NesScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct PaintableScreen {}

impl PaintableScreen {}

#[glib::object_subclass]
impl ObjectSubclass for PaintableScreen {
    const NAME: &'static str = "CustomPaintable";
    type Type = NesScreen;
    type Interfaces = (gdk::Paintable,);
}

impl ObjectImpl for PaintableScreen {}

impl PaintableImpl for PaintableScreen {
    fn flags(&self) -> gdk::PaintableFlags {
        // Fixed size
        gdk::PaintableFlags::SIZE
    }

    fn intrinsic_width(&self) -> i32 {
        SCREEN_WIDTH as i32
    }

    fn intrinsic_height(&self) -> i32 {
        SCREEN_HEIGHT as i32
    }

    fn snapshot(&self, snapshot: &gdk::Snapshot, _width: f64, _height: f64) {
        let frame = {
            let mut writer = RENDER_SIGNALER.get().unwrap().write().unwrap();
            match writer.frame.take() {
                Some(frame) => frame,
                None => {
                    debug!("Trying to render without any frame");
                    return;
                }
            }
        };

        let width = self.intrinsic_width();
        let height = self.intrinsic_height();

        let context =
            snapshot.append_cairo(&graphene::Rect::new(0.0, 0.0, width as f32, height as f32));
        let pixel_size = 0.9;

        for (h, row) in frame.iter().enumerate().take(ORIGINAL_SCREEN_HEIGHT) {
            for (w, pixel) in row.iter().enumerate().take(ORIGINAL_SCREEN_WIDTH) {
                context.set_source_rgb(pixel.red(), pixel.green(), pixel.blue());
                context.rectangle(
                    (h * PIXEL_SCALE_FACTOR) as f64,
                    (w * PIXEL_SCALE_FACTOR) as f64,
                    pixel_size * PIXEL_SCALE_FACTOR as f64,
                    pixel_size * PIXEL_SCALE_FACTOR as f64,
                );
                context.fill().unwrap();
            }
        }
    }
}
