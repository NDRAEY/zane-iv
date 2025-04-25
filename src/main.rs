use std::{cell::RefCell, rc::Rc};

use gi_ui::{
    Drawable,
    canvas::Canvas,
    components::{self, image, layout::linear::LinearLayout, text8x8},
    helpers::i_am_sure_mut,
};
use gi_ui_app::Application;

type Trusted<T> = Rc<RefCell<Box<T>>>;

struct App {
    app: gi_ui_app::Application,
    ui: Trusted<dyn gi_ui::Drawable>,

    statusbar_text: Trusted<dyn gi_ui::Drawable>,
    canvas: Trusted<dyn Drawable>,
}

impl App {
    pub fn new() -> Self {
        let (ui, status_text, canvas) = Self::build_ui();
        let mut app = Application::new(300, 300).unwrap();

        let ui_ref = app.attach_main_drawable(ui).clone();

        Self {
            ui: ui_ref,
            app,
            statusbar_text: status_text,
            canvas,
        }
    }

    fn build_ui() -> (
        Box<dyn Drawable>,
        Trusted<dyn Drawable>,
        Trusted<dyn Drawable>,
    ) {
        let mut ui = LinearLayout::new();

        let status_text = text8x8::Text::new()
            .with_color(0xff_ffffff)
            .with_size(12)
            .with_text("Zane v0.1 by NDRAEY");

        let st = ui.push(status_text);

        let mut canvas = Canvas::new(100, 100);
        canvas.fill(0xff_112cef);

        let canv = ui.push(canvas);

        (Box::new(ui), st, canv)
    }

    pub fn set_status(&mut self, status: &String) {
        let mut binding = self.statusbar_text.borrow_mut();

        let statusbar = i_am_sure_mut::<text8x8::Text>(binding.as_mut());

        statusbar.set_text(status.clone());
    }

    pub fn load_image_from_file<S: ToString>(&mut self, path: S) -> Result<(), ()> {
        let path = path.to_string();
        let data = std::fs::read(&path).unwrap();

        let image = nimage::import::open(&data);

        if image.is_none() {
            return Err(());
        }

        let (image_type, image) = image.unwrap();

        self.set_status(&format!(
            "{} [{}] - {}x{}",
            path,
            image_type.to_uppercase(),
            image.width(),
            image.height()
        ));
        
        let mut binding = self.canvas.borrow_mut();
        binding.set_size(image.width(), image.height());

        let canvas = i_am_sure_mut::<Canvas>(binding.as_mut());
        let buffer = canvas.buffer_mut();

        for y in 0..image.height() {
            for x in 0..image.width() {
                let _ =
                    canvas.set_pixel(x as _, y as _, 0xff_000000 | image.get_pixel(x, y).unwrap());
            }
        }

        Ok(())
    }

    pub fn run(&mut self) {
        self.app.run().unwrap()
    }
}

fn main() {
    let filename = std::env::args().skip(1).next();

    if filename.is_none() {
        eprintln!("No file!");
        std::process::exit(1);
    }

    let filename = filename.unwrap();

    let mut app = App::new();

    app.load_image_from_file(filename);

    app.run();
}
