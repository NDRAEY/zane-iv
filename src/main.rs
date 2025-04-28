use std::{cell::RefCell, ops::DerefMut, rc::Rc};

use gi_ui::{
    canvas::Canvas, components::{layout::linear::LinearLayout, text8x8}, helpers::i_am_sure_mut, size::Size, Drawable
};
use gi_ui_app::Application;
use nimage::Image;

type Trusted<T> = Rc<RefCell<T>>;

struct App {
    app: gi_ui_app::Application,
    ui: Trusted<Box<dyn Drawable>>,

    statusbar_text: Trusted<dyn Drawable>,
    canvas: Trusted<dyn Drawable>,

    image: Option<RefCell<Image>>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Application::new(300, 300).unwrap();

        app.set_title("Zane Image Viewer")
            .expect("Failed to set title!");

        let (ui, status_text, canvas) = Self::build_ui();

        let ui_ref = app.attach_main_drawable(ui).clone();

        Self {
            ui: ui_ref,
            app,
            statusbar_text: status_text,
            canvas,
            image: None
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

    pub fn resize_to_fit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = self.ui.borrow().size();

        self.app.resize(width as _, height as _)?;

        Ok(())
    }

    pub fn set_status(&mut self, status: &String) {
        let mut binding = self.statusbar_text.borrow_mut();

        let statusbar = i_am_sure_mut::<text8x8::Text>(binding.deref_mut());

        statusbar.set_text(status.clone());
    }

    pub fn load_image_from_file<S: ToString>(
        &mut self,
        path: S,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.to_string();
        let data = std::fs::read(&path).unwrap();

        let image = nimage::import::open(&data);

        if image.is_none() {
            return Err(Box::new(std::io::Error::other(String::from(
                "Failed to load file!",
            ))));
        }

        let (image_type, image) = image.unwrap();

        
        self.set_status(&format!(
            "{} [{}] - {}x{}",
            path,
            image_type.to_uppercase(),
            image.width(),
            image.height()
        ));

        // WTF? I'll explain.
        //
        // I just need to set canvas size, so I mutually borrowed `self.canvas`.
        // And when I set size of the canvas I need to resize the window to fit whole UI into it.
        // But at this time there's an active borrow on `binding` that doesn't allow us to
        // double-borrow `self`.
        //
        // So limiting `binding`, resizing canvas and borrowing it again works.
        {
            let mut binding = self.canvas.borrow_mut();
            binding.set_size(image.width(), image.height());
        }

        self.resize_to_fit()?;

        let mut binding = self.canvas.borrow_mut();
        let canvas = i_am_sure_mut::<Canvas>(binding.deref_mut());

        for y in 0..image.height() {
            for x in 0..image.width() {
                let _ =
                    canvas.set_pixel(x as _, y as _, 0xff_000000 | image.get_pixel(x, y).unwrap());
            }
        }

        self.image = Some(RefCell::new(image));

        Ok(())
    }

    pub fn resize_canvas(&self, width: usize, height: usize) {
        let mut binding = self.canvas.borrow_mut();
        let canvas = i_am_sure_mut::<Canvas>(binding.deref_mut());

        let image = self.image.as_ref().unwrap().borrow().scale_to_new(width, height);

        canvas.set_size(width, height);

        for y in 0..image.height() {
            for x in 0..image.width() {
                let _ =
                    canvas.set_pixel(x as _, y as _, 0xff_000000 | image.get_pixel(x, y).unwrap());
            }
        }
    }

    pub fn on_resize(width: usize, height: usize) {
        println!("{width} {height}");

        // ...
    }

    pub fn run(&self) {
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

    let app = Rc::new(RefCell::new(App::new()));

    let app_cb = app.clone();
    app.borrow_mut().app.set_resize_callback(move |width, height| {
        let binding = &app_cb.borrow();

        // binding.set_title(&format!{"{width} {height}"});
        binding.resize_canvas(width, height);
    });

    app.borrow_mut().load_image_from_file(filename)
        .expect("Failed to load file!");

    let binding = app.borrow();
    binding.run();
}
