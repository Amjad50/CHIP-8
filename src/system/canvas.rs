use super::display::DEFAULT_PIXEL_SIZE;
use cairo;
use gtk::DrawingArea;

pub struct Canvas {
    pub data: Vec<bool>,
    pub area: DrawingArea,
    width: u16,
    height: u16,
}

impl Canvas {
    pub fn new(data: Vec<bool>, graph_area: DrawingArea, width: u16, height: u16) -> Canvas {
        Canvas {
            data: data,
            area: graph_area,
            width: width,
            height: height,
        }
    }

    pub fn draw(&self, cr: &cairo::Context) {
        for i in 0..self.height {
            for j in 0..self.width {
                // black if true, white if false
                let color = if self.data[(i * self.width + j) as usize] {
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
    }
}
