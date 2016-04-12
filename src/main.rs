extern crate gtk;
extern crate gdk_pixbuf;
extern crate breeze_core;

mod view;

fn main() {
    gtk::init().unwrap();
    view::MainView::new().main_loop();
}
