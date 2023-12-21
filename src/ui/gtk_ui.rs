/// GTK-4 UI
///
/// User Interface built on top of GTK-4 library
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread::{spawn, JoinHandle};

use crossbeam_channel::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, graphene};
use gtk::{Application, ApplicationWindow, Inhibit};
use log::debug;
use once_cell::sync::OnceCell;

use crate::ui::{Frame, Ui, PIXEL_SCALE_FACTOR};
use crate::hardware::{SCREEN_HEIGHT, SCREEN_WIDTH};

const APP_ID: &str = "jotare-nes-emulator";
const APP_NAME: &str = "NES Emulator (by jotare)";

static RENDER_SIGNALER: OnceCell<Arc<RwLock<RenderSignaler>>> = OnceCell::new();

// Used only inside GtkUi thread
thread_local! {
    static KEYBOARD_CHANNEL: OnceCell<Option<Sender<char>>> = OnceCell::new();
}

pub struct GtkUi {
    screen_width: usize,
    screen_height: usize,
    pixel_scale_factor: usize,
    handle: Option<JoinHandle<()>>,
    keyboard_channel: Option<Sender<char>>,
}

impl GtkUi {
    pub fn builder() -> GtkUiBuilder {
        GtkUiBuilder::default()
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
        let keyboard_channel = self.keyboard_channel.take();

        let join_handle = spawn(move || {
            KEYBOARD_CHANNEL
                .with(|cell| cell.set(keyboard_channel))
                .expect("Unreachable error initializing keboard channel thread local");

            let app = Application::builder().application_id(APP_ID).build();

            app.connect_activate(move |app| {
                // Create main window
                let window = ApplicationWindow::builder()
                    .application(app)
                    .title(APP_NAME)
                    .build();

                let quit_action = gio::SimpleAction::new("quit", None);
                quit_action.connect_activate(glib::clone!(@weak window => move |_, _| {
                    window.close();
                }));
                window.add_action(&quit_action);

                let event_controller = gtk::EventControllerKey::builder()
                    .name("NES Keyboard Controller")
                    .build();

                event_controller.connect_key_pressed(|event_controller, keyval, keycode, state| {
                    Self::on_key_pressed(event_controller, keyval, keycode, state)
                });
                window.add_controller(event_controller);

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
                    let signaler = RENDER_SIGNALER.get().unwrap().read().unwrap();
                    if signaler.should_render() {
                        area.queue_draw();
                    }

                    Continue(true)
                });

                // Present window
                window.present();
            });

            app.set_accels_for_action("win.quit", &["<Ctrl>Q"]);

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

    fn on_key_pressed(
        event_controller: &gtk::EventControllerKey,
        keyval: gdk::Key,
        keycode: u32,
        modifier_type: gdk::ModifierType,
    ) -> Inhibit {
        println!("KEY PRESSED: {keyval} {modifier_type:?}");

        // We only handle lowercase and uppercase chars and ignore other combinations (C-, M-, ...)
        if !(modifier_type == gdk::ModifierType::empty()
            || modifier_type == gdk::ModifierType::SHIFT_MASK
            || modifier_type == gdk::ModifierType::LOCK_MASK)
        {
            return Inhibit(false);
        }

        let character = match keyval.to_unicode() {
            Some(c) => c,
            None => return Inhibit(false),
        };

        KEYBOARD_CHANNEL.with(|cell| {
            match cell
                .get()
                .expect("Keyboard channel once cell should be initialized by now")
            {
                Some(sender) => {
                    sender.send(character);
                    Inhibit(true)
                }
                None => Inhibit(false),
            }
        })
    }
}

impl Ui for GtkUi {
    fn render(&self, frame: Frame) {
        if let Some(signaler) = RENDER_SIGNALER.get() {
            signaler.write().unwrap().set_frame(frame);
        }
    }
}

#[derive(Default)]
pub struct GtkUiBuilder {
    screen_width: Option<usize>,
    screen_height: Option<usize>,
    pixel_scale_factor: Option<usize>,
    keyboard_channel: Option<Sender<char>>,
}

impl GtkUiBuilder {
    pub fn build(self) -> GtkUi {
        GtkUi {
            screen_width: self.screen_width.unwrap_or(SCREEN_WIDTH),
            screen_height: self.screen_height.unwrap_or(SCREEN_HEIGHT),
            pixel_scale_factor: self.pixel_scale_factor.unwrap_or(PIXEL_SCALE_FACTOR),
            handle: None,
            keyboard_channel: self.keyboard_channel,
        }
    }

    pub fn screen_width(mut self, width: usize) -> Self {
        self.screen_width.replace(width);
        self
    }

    pub fn screen_height(mut self, height: usize) -> Self {
        self.screen_height.replace(height);
        self
    }

    pub fn pixel_scale_factor(mut self, factor: usize) -> Self {
        self.pixel_scale_factor.replace(factor);
        self
    }

    pub fn keyboard_channel(mut self, sender: Sender<char>) -> Self {
        self.keyboard_channel.replace(sender);
        self
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
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
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
        let pixel_size = 0.95;

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
