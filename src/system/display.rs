use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, DrawingArea, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;

use super::canvas::Canvas;

const APPLICATION_ID: Option<&str> = Some("com.amjad.chip-8");
const DISPLAY_TITLE: &str = "CHIP-8";
pub const DEFAULT_PIXEL_SIZE: u16 = 10;

pub struct Display {
    application: Application,
    window: Window,
    canvas: Rc<RefCell<Canvas>>,
    width: u16,
    height: u16,
    pixelWidth: i32,
    pixelHeight: i32,
}

impl Display {
    fn create_root_window(width: i32, height: i32, area: &DrawingArea) -> Window {
        let window = Window::new(WindowType::Toplevel);
        window.set_title(DISPLAY_TITLE);
        window.set_default_size(width, height);
        window.set_resizable(false);

        window.add(area);

        window.show_all();
        window
    }

    pub fn new(width: u16, height: u16) -> Display {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
        }

        let application = Application::new(APPLICATION_ID, Default::default())
            .expect("failed to initialize GTK application");

        let area = DrawingArea::new();

        // will be added to application before run
        let window = Display::create_root_window(
            (width * DEFAULT_PIXEL_SIZE) as i32,
            (height * DEFAULT_PIXEL_SIZE) as i32,
            &area,
        );

        let canvas = Canvas::new(vec![false; (width * height) as usize], area, width, height);

        let canvas = Display::connect_canvas(canvas);
        let c_canvas = canvas.clone();

        let display = Display {
            application: application,
            window: window,
            canvas: c_canvas,
            width: width,
            height: height,
            pixelWidth: (width * DEFAULT_PIXEL_SIZE) as i32,
            pixelHeight: (height * DEFAULT_PIXEL_SIZE) as i32,
        };

        display
    }

    fn connect_canvas(canvas: Canvas) -> Rc<RefCell<Canvas>> {
        let area = canvas.area.clone();
        let canvas = Rc::new(RefCell::new(canvas));
        let c_canvas = canvas.clone();
        area.connect_draw(move |_, cr| {
            c_canvas.borrow().draw(&cr);
            Inhibit(false)
        });
        canvas
    }

    pub fn redraw(&self) {
        self.window
            .queue_draw_area(0, 0, self.pixelWidth, self.pixelHeight);
    }

    pub fn run(self) {
        let window = self.window;
        self.application.connect_activate(move |app: &Application| {
            app.add_window(&window);
        });
        self.application.run(&[]);
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, value: bool) {
        assert_eq!(y * self.width + x < self.width * self.height, true);
        self.canvas.borrow_mut().data[(y * self.width + x) as usize] = value;
    }
}
