use gdk::enums::key;
use gdk::keyval_to_upper;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, Builder, DrawingArea, Grid, ListStore, TextBuffer, Window};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

const APPLICATION_ID: Option<&str> = Some("com.amjad.chip-8");
pub static mut APPLICATION: Option<Application> = None;
pub const DEFAULT_PIXEL_SIZE: u16 = 10;

const KEYBOARD_MAPPING: [u32; 16] = [
    key::X,
    key::_1,
    key::_2,
    key::_3,
    key::Q,
    key::W,
    key::E,
    key::A,
    key::S,
    key::D,
    key::Z,
    key::C,
    key::_4,
    key::R,
    key::F,
    key::V,
];

const KEYPAD_GRID_MAPPING: [u8; 16] = [13, 0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 14, 3, 7, 11, 15];

pub struct Display {
    window: Rc<RefCell<Window>>,
    area: DrawingArea,
    registers_buffer: TextBuffer,
    stack_buffer: TextBuffer,
    memory_list_store: ListStore,
    keypad_grid: Rc<RefCell<Grid>>,
    width: u16,
    height: u16,
    data: Rc<RefCell<Vec<bool>>>,
    keyboard: Rc<RefCell<[bool; 16]>>,
}

impl Display {
    fn build_layout(
        width: i32,
        height: i32,
    ) -> (Window, DrawingArea, TextBuffer, TextBuffer, ListStore, Grid) {
        let glade_src = include_str!("../../layout.glade");

        let builder = Builder::new_from_string(glade_src);
        let window: Window = builder.get_object("main_application_window").unwrap();
        let area: DrawingArea = builder.get_object("canvas").unwrap();

        let registers_buffer: TextBuffer = builder.get_object("registersBuffer").unwrap();
        let stack_buffer: TextBuffer = builder.get_object("stackBuffer").unwrap();
        let memory_list_store: ListStore = builder.get_object("memoryViewListStore").unwrap();
        let keypad_grid: Grid = builder.get_object("keypad").unwrap();

        // assign CSS
        let provider = gtk::CssProvider::new();

        provider
            .load_from_data(include_bytes!("../../style.css"))
            .expect("Failed to load CSS");

        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        area.set_size_request(width, height);

        window.show_all();

        (
            window,
            area,
            registers_buffer,
            stack_buffer,
            memory_list_store,
            keypad_grid,
        )
    }

    pub fn new(width: u16, height: u16) -> Display {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
        }

        let _application = Application::new(APPLICATION_ID, Default::default())
            .expect("failed to initialize GTK application");

        // will be added to application before run
        let (window, area, registers_buffer, stack_buffer, memory_list_store, keypad_grid) =
            Display::build_layout(
                (width * DEFAULT_PIXEL_SIZE) as i32,
                (height * DEFAULT_PIXEL_SIZE) as i32,
            );

        let display = Display {
            window: Rc::new(RefCell::new(window)),
            area: area,
            registers_buffer: registers_buffer,
            stack_buffer: stack_buffer,
            memory_list_store: memory_list_store,
            keypad_grid: Rc::new(RefCell::new(keypad_grid)),
            width: width,
            height: height,
            data: Rc::new(RefCell::new(vec![false; (width * height) as usize])),
            keyboard: Rc::new(RefCell::new([false; 16])),
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

    pub fn update_stack_debug(&self, stack: &[u16]) {
        let result = stack
            .iter() // iterater over the stack
            .rev() // in the reverse order
            .map(|x| format!("{:04x}", x)) // convert it to a string representation
            .collect::<Vec<String>>()
            .join("\n"); // join by newline

        self.stack_buffer.set_text(&result);
        self.stack_buffer.set_modified(true)
    }

    pub fn update_registers_debug(&self, V: &[u8; 16], I: u16, PC: u16, DT: u8, ST: u8, SP: u8) {
        let mut result: String = "".to_owned();
        // V registers
        for i in 0..V.len() {
            result.push_str(&format!("V{:1X}: {:02x}  ", i, V[i]));
            if i % 4 == 3 {
                result.push('\n');
            }
        }
        result.push('\n');

        // I
        result.push_str(&format!("I: {:04x}\n\n", I));

        // PC
        result.push_str(&format!("PC: {:04x}\n\n", PC));

        // DT, ST
        result.push_str(&format!("DT: {:02x}\n", DT));
        result.push_str(&format!("ST: {:02x}\n\n", ST));

        // SP
        result.push_str(&format!("SP: {:02x}", SP));

        self.registers_buffer.set_text(&result);
        self.registers_buffer.set_modified(true);
    }

    fn get_hex_string(bytes: &[u8]) -> String {
        let mut result: String = "".to_owned();

        for byte in bytes {
            result.push_str(&format!("{:02x} ", byte));
        }

        result
    }

    fn get_ascii_string(bytes: &[u8]) -> String {
        let mut result: String = "".to_owned();

        for &byte in bytes {
            let c = byte as char;

            result.push(if c.is_ascii() && !c.is_control() {
                c
            } else {
                '.'
            });
        }

        result
    }

    pub fn update_memory_debug(&self, memory: &[u8], first_time: bool) {
        if first_time {
            self.memory_list_store.clear();
        }

        let mut current_item = if first_time {
            self.memory_list_store.append()
        } else {
            self.memory_list_store.get_iter_first().unwrap()
        };

        for i in (0..memory.len()).step_by(16) {
            let address = format!("{:04x}", i);
            let hex = Display::get_hex_string(&memory[i..i + 16]);
            let ascii = Display::get_ascii_string(&memory[i..i + 16]);

            self.memory_list_store
                .set(&current_item, &[0, 1, 2], &[&address, &hex, &ascii]);

            // prepare for the next loop
            if first_time {
                current_item = self.memory_list_store.append();
            } else {
                // in the next loop, this should set current_item to invalid
                // but should not be used
                self.memory_list_store.iter_next(&current_item);
            }
        }
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

    fn update_keyboard(keyval: u32, keyboard: &mut [bool; 16], value: bool) {
        let keyval = keyval_to_upper(keyval);

        match KEYBOARD_MAPPING.iter().position(|&x| x == keyval) {
            Some(index) => {
                keyboard[index] = value;
            }
            None => {}
        };
    }

    fn update_keypad_debug(keypad_grid: &Grid, keyboard: &[bool; 16]) {
        for (i, &value) in keyboard.iter().enumerate() {
            let index = KEYPAD_GRID_MAPPING[i as usize];
            let row = (index / 4) as i32;
            let col = (index % 4) as i32;

            let child = keypad_grid.get_child_at(col, row).unwrap();
            let style_context = child.get_style_context();

            if value {
                style_context.add_class("pressed");
            } else {
                style_context.remove_class("pressed");
            }
        }
    }

    pub fn setup_keyboard(&self) {
        let window = self.window.borrow();
        // FIXME: is there a better way to do this?
        let keyboard_clone_press = self.keyboard.clone();
        let keyboard_clone_release = self.keyboard.clone();
        let keypad_grid_clone_press = self.keypad_grid.clone();
        let keypad_grid_clone_release = self.keypad_grid.clone();

        window.connect_key_press_event(move |_, event| {
            let mut keyboard = keyboard_clone_press.borrow_mut();
            let keypad_grid = keypad_grid_clone_press.borrow();

            Display::update_keyboard(event.get_keyval(), &mut *keyboard, true);
            Display::update_keypad_debug(&*keypad_grid, &keyboard);

            Inhibit(false)
        });

        window.connect_key_release_event(move |_, event| {
            let mut keyboard = keyboard_clone_release.borrow_mut();
            let keypad_grid = keypad_grid_clone_release.borrow();

            Display::update_keyboard(event.get_keyval(), &mut *keyboard, false);
            Display::update_keypad_debug(&*keypad_grid, &keyboard);

            Inhibit(false)
        });
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
                }
                None => {
                    // if there is no application, don't run
                }
            }
        }
    }

    pub fn get_height(&self) -> u16 {
        self.height
    }

    pub fn get_width(&self) -> u16 {
        self.width
    }

    pub fn get_keyboard_data(&self) -> Ref<[bool; 16]> {
        self.keyboard.borrow()
    }

    pub fn get_keyboard_data_copy(&self) -> [bool; 16] {
        let mut x = [false; 16];
        for (i, &xx) in self.keyboard.borrow().iter().enumerate() {
            x[i] = xx;
        }
        x
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
