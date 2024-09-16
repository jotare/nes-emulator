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
use gtk::{gdk, gio, glib, graphene};
use gtk::{Application, ApplicationWindow, Inhibit};
use log::debug;
use once_cell::sync::OnceCell;

use crate::events::KeyboardPublisher;
use crate::events::SharedEventBus;
use crate::hardware::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::settings::DEFAULT_PIXEL_SCALE_FACTOR;
use crate::ui::{Frame, Ui};

use super::UiError;

const APP_ID: &str = "jotare-nes-emulator";
const APP_NAME: &str = "NES Emulator (by jotare)";

static RENDER_SIGNALER: OnceCell<Arc<RwLock<RenderSignaler>>> = OnceCell::new();

// Used only inside GtkUi thread
thread_local! {
    static RENDER_THREAD_STATE: OnceCell<RenderThreadState> = const { OnceCell::new() };
}

pub struct GtkUi {
    screen_width: usize,
    screen_height: usize,
    pixel_scale_factor: usize,
    handle: Option<JoinHandle<()>>,
    keyboard_channel: Option<KeyboardPublisher>,
    event_bus: Option<SharedEventBus>,
}

#[derive(Debug)]
struct RenderThreadState {
    keyboard: Option<KeyboardPublisher>,
    event_bus: Option<SharedEventBus>,
}

impl GtkUi {
    pub fn builder() -> GtkUiBuilder {
        GtkUiBuilder::new()
    }

    /// GTK UI is based in a secondary thread that listens for a render event and renders a Frame.
    ///
    /// Communication is done using a global variable that notifies the thread
    /// when a new Frame can be drawn
    ///
    /// TODO: Some internal data is hold as thread locals as the current GTK
    /// usage has some limitations on custom state. If a better way to handle
    /// local state is found, it'd be welcomed :)
    fn render_thread(
        screen_size: (usize, usize),
        pixel_scale_factor: usize,
        event_bus: Option<SharedEventBus>,
        keyboard: Option<KeyboardPublisher>,
    ) {
        let (screen_width, screen_height) = screen_size;

        // setup thread local variables
        RENDER_THREAD_STATE
            .with(|cell| {
                cell.set(RenderThreadState {
                    keyboard,
                    event_bus,
                })
            })
            .expect("Unreachable error initializing render thread state");

        let app = Application::builder().application_id(APP_ID).build();

        app.connect_activate(move |app| {
            // Create main window
            let window = ApplicationWindow::builder()
                .application(app)
                .title(APP_NAME)
                .build();

            // Setup a quit hook that sends a shutdown event to the NES
            let quit_action = gio::SimpleAction::new("quit", None);
            quit_action.connect_activate(glib::clone!(@weak window => move |_, _| {
                window.close();

                RENDER_THREAD_STATE.with(|cell| {
                    let state = cell.get()
                        .expect("Thread local once cell should be initialized by now");
                    if let Some(ref event_bus) = state.event_bus {
                        event_bus.access().emit(crate::events::Event::SwitchOff);
                    }
                })
            }));
            window.add_action(&quit_action);

            // Keyboard controll so the GUI can forward key presses to the
            // controllers
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

        // Standard C-q to quit the GUI window
        app.set_accels_for_action("win.quit", &["<Ctrl>Q"]);

        app.run();
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

        RENDER_THREAD_STATE.with(|cell| {
            let state = cell
                .get()
                .expect("Thread local once cell should be initialized by now");

            match state.keyboard {
                Some(ref keyboard_publisher) => {
                    keyboard_publisher.push_char(character);
                    Inhibit(true)
                }
                None => Inhibit(false),
            }
        })
    }
}

impl Ui for GtkUi {
    /// Starts a GTK running GUI. It should only be called once during the whole
    /// program. If called more than once, it'll panic.
    ///
    /// TODO: overcome the limitation and be able to start/stop the UI more
    /// times
    fn start(&mut self) -> Result<(), UiError> {
        let already_initialized = RENDER_SIGNALER
            .set(Arc::new(RwLock::new(RenderSignaler::default())))
            .is_err();

        if already_initialized {
            return Err(UiError::AlreadyStarted(
                "GTK UI is already started, can't start it twice".to_string(),
            ));
        }

        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let pixel_scale_factor = self.pixel_scale_factor;
        let keyboard_channel = self.keyboard_channel.take();
        let event_bus = self.event_bus.take();

        let join_handle = spawn(move || {
            Self::render_thread(
                (screen_width, screen_height),
                pixel_scale_factor,
                event_bus,
                keyboard_channel,
            )
        });

        self.handle.replace(join_handle);

        Ok(())
    }

    /// Signal the GUI to render a new frame. This will be communicated to the
    /// GTK render thread and it'll update the frame as soon as possible
    fn render(&mut self, frame: Frame) {
        if let Some(signaler) = RENDER_SIGNALER.get() {
            signaler.write().unwrap().set_frame(frame);
        }
    }

    fn stop(&mut self) -> Result<(), UiError> {
        let handle = self.handle.take().ok_or(UiError::NotStarted)?;
        debug!("Waiting UI thread to end...");
        handle.join().map_err(|_| {
            UiError::Unhandled("Error waiting UI thread to join (stop)".to_string())
        })?;
        debug!("UI thread ended correctly");

        // TODO: cleanup thread local and global variables.
        //
        // XXX: Right now, as global variables are not mutable and we don't do
        // anything to mutate them, we can't clean the state so a second cycle
        // of start/stop is not possible

        Ok(())
    }
}

pub struct GtkUiBuilder {
    screen_width: usize,
    screen_height: usize,
    pixel_scale_factor: usize,
    keyboard: Option<KeyboardPublisher>,
    event_bus: Option<SharedEventBus>,
}

impl GtkUiBuilder {
    pub fn new() -> Self {
        Self {
            screen_height: SCREEN_HEIGHT,
            screen_width: SCREEN_WIDTH,
            pixel_scale_factor: DEFAULT_PIXEL_SCALE_FACTOR,
            keyboard: None,
            event_bus: None,
        }
    }

    pub fn build(self) -> GtkUi {
        GtkUi {
            screen_width: self.screen_width,
            screen_height: self.screen_height,
            pixel_scale_factor: self.pixel_scale_factor,
            handle: None,
            keyboard_channel: self.keyboard,
            event_bus: self.event_bus,
        }
    }

    pub fn screen_size(mut self, width: usize, height: usize) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
    }

    pub fn pixel_scale_factor(mut self, factor: usize) -> Self {
        self.pixel_scale_factor = factor;
        self
    }

    pub fn with_keyboard_publisher(mut self, keyboard_publisher: KeyboardPublisher) -> Self {
        self.keyboard = Some(keyboard_publisher);
        self
    }

    pub fn with_event_bus(mut self, event_bus: SharedEventBus) -> Self {
        self.event_bus.replace(event_bus);
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
            pixel_scale_factor: DEFAULT_PIXEL_SCALE_FACTOR,
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
