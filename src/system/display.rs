use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, DrawingArea, Window, WindowType};
use std::cell::RefCell;
use std::rc::Rc;

const APPLICATION_ID: Option<&str> = Some("com.amjad.chip-8");
pub static mut APPLICATION: Option<Application> = None;
const DISPLAY_TITLE: &str = "CHIP-8";
pub const DEFAULT_PIXEL_SIZE: u16 = 10;

pub struct Display {
    window: Rc<RefCell<Window>>,
    area: DrawingArea,
    width: u16,
    height: u16,
    data: Rc<RefCell<Vec<bool>>>,
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

        let _application = Application::new(APPLICATION_ID, Default::default())
            .expect("failed to initialize GTK application");

        let area = DrawingArea::new();

        // will be added to application before run
        let window = Display::create_root_window(
            (width * DEFAULT_PIXEL_SIZE) as i32,
            (height * DEFAULT_PIXEL_SIZE) as i32,
            &area,
        );

        let display = Display {
            window: Rc::new(RefCell::new(window)),
            area: area,
            width: width,
            height: height,
            data: Rc::new(RefCell::new(vec![false; (width * height) as usize])),
        };

        let c_window = display.window.clone();

        _application.connect_activate(move |app: &Application| {
            let window = &*c_window.borrow();
            app.add_window(window);
        });
        display.setup_drawing();

        unsafe {
            APPLICATION = Some(_application);
        }

        display
    }

    fn setup_drawing(&self) {
        let height = self.height;
        let width = self.width;
        let c_data = self.data.clone();
        self.area.connect_draw(move |_, cr| {
            for i in 0..height {
                for j in 0..width {
                    // black if true, white if false
                    let color = if c_data.borrow()[(i * width + j) as usize] {
                        0.
                    } else {
                        1.
                    };
                    // if its 0, it will result in #000(white)
                    // if its 1, it will result in #fff(black)
                    cr.set_source_rgb(color, color, color);
                    cr.rectangle(
                        (j * DEFAULT_PIXEL_SIZE) as f64,
                        (i * DEFAULT_PIXEL_SIZE) as f64,
                        DEFAULT_PIXEL_SIZE as f64,
                        DEFAULT_PIXEL_SIZE as f64,
                    );
                    cr.fill();
                }
            }
            Inhibit(false)
        });
    }

    pub fn redraw(&self) {
        let window = self.window.borrow();
        window.queue_draw_area(
            0,
            0,
            window.get_allocated_width(),
            window.get_allocated_height(),
        );
    }

    pub fn run_in_loop<F: 'static>(&self, interval: u32, func: F)
    where
        F: Fn() -> (),
    {
        timeout_add(interval, move || {
            func();
            Continue(true)
        });
    }

    pub fn run_application() {
        unsafe {
            match &APPLICATION {
                Some(app) => {
                    app.run(&[]);
                },
                None => {
                    // if there is no application, don't run
                },
            }
        }
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn draw_pixel(&mut self, x: u16, y: u16, value: bool) {
        assert_eq!(y * self.width + x < self.width * self.height, true);
        self.data.borrow_mut()[(y * self.width + x) as usize] = value;
    }

    pub fn xor_pixel(&mut self, x: u16, y: u16, value: bool) -> bool {
        assert_eq!(y * self.width + x < self.width * self.height, true);
        // get a pointer to the value to change
        let data_ref = &mut self.data.borrow_mut()[(y * self.width + x) as usize];
        // collide if both are 1, meaning when XORing, the pixel in the screen
        // will be erased
        let collision = *data_ref & value;
        *data_ref ^= value;
        return collision;
    }
}
