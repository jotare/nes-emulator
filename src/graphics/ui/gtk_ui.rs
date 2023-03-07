/// GTK-4 UI
///
/// User Interface built on top of GTK-4 library
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
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
    screen_width: usize,
    screen_height: usize,
    pixel_scale_factor: usize,
    handle: Option<JoinHandle<()>>,
}

impl GtkUi {
    pub fn new(screen_width: usize, screen_height: usize, pixel_scale_factor: usize) -> Self {
        Self {
            screen_width,
            screen_height,
            pixel_scale_factor,
            handle: None,
        }
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

        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let pixel_scale_factor = self.pixel_scale_factor;

        let join_handle = spawn(move || {
            let app = Application::builder().application_id(APP_ID).build();

            app.connect_activate(move |app| {
                // Create main window
                let window = ApplicationWindow::builder()
                    .application(app)
                    .title(APP_NAME)
                    .build();

                // Screen
                let paintable = NesScreen::new();
                paintable.setup(screen_width, screen_height, pixel_scale_factor);

                let picture = gtk::Picture::builder()
                    .width_request((screen_width * pixel_scale_factor) as i32)
                    .height_request((screen_height * pixel_scale_factor) as i32)
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

impl Default for GtkUi {
    fn default() -> Self {
        Self::new(
            ORIGINAL_SCREEN_WIDTH,
            ORIGINAL_SCREEN_HEIGHT,
            PIXEL_SCALE_FACTOR,
        )
    }
}

struct RenderSignaler {
    screen_frame: Option<Frame>,
}

impl RenderSignaler {
    pub fn new() -> Self {
        Self { screen_frame: None }
    }

    pub fn should_render(&self) -> bool {
        self.screen_frame.is_some()
    }

    pub fn set_frame(&mut self, frame: Frame) {
        self.screen_frame.replace(frame);
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

    fn setup(&self, width: usize, height: usize, pixel_scale_factor: usize) {
        self.imp().setup(width, height, pixel_scale_factor);
    }
}

impl Default for NesScreen {
    fn default() -> Self {
        Self::new()
    }
}

struct PaintableScreenInner {
    width: usize,
    height: usize,
    pixel_scale_factor: usize,
}

impl Default for PaintableScreenInner {
    fn default() -> Self {
        Self {
            width: ORIGINAL_SCREEN_WIDTH,
            height: ORIGINAL_SCREEN_HEIGHT,
            pixel_scale_factor: PIXEL_SCALE_FACTOR,
        }
    }
}

#[derive(Default)]
struct PaintableScreen {
    inner: Rc<RefCell<PaintableScreenInner>>,
}

impl PaintableScreen {
    fn setup(&self, width: usize, height: usize, pixel_scale_factor: usize) {
        *self.inner.borrow_mut() = PaintableScreenInner {
            width,
            height,
            pixel_scale_factor,
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for PaintableScreen {
    const NAME: &'static str = "PaintableScreen";
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
        let inner = self.inner.borrow();
        (inner.width * inner.pixel_scale_factor) as i32
    }

    fn intrinsic_height(&self) -> i32 {
        let inner = self.inner.borrow();
        (inner.height * inner.pixel_scale_factor) as i32
    }

    fn snapshot(&self, snapshot: &gdk::Snapshot, _width: f64, _height: f64) {
        let frame = {
            let mut writer = RENDER_SIGNALER.get().unwrap().write().unwrap();
            match writer.screen_frame.take() {
                Some(frame) => frame,
                None => {
                    debug!("Trying to render without any frame");
                    return;
                }
            }
        };

        let (width, height, pixel_scale_factor) = {
            let inner = self.inner.borrow();
            (inner.width, inner.height, inner.pixel_scale_factor)
        };
        let context = snapshot.append_cairo(&graphene::Rect::new(
            0.0,
            0.0,
            self.intrinsic_width() as f32,
            self.intrinsic_height() as f32,
        ));
        let pixel_size = 0.9;

        for (h, row) in frame.iter().enumerate().take(height) {
            for (w, pixel) in row.iter().enumerate().take(width) {
                context.set_source_rgb(pixel.red(), pixel.green(), pixel.blue());
                context.rectangle(
                    (w * pixel_scale_factor) as f64,
                    (h * pixel_scale_factor) as f64,
                    pixel_size * pixel_scale_factor as f64,
                    pixel_size * pixel_scale_factor as f64,
                );
                context.fill().unwrap();
            }
        }
    }
}
