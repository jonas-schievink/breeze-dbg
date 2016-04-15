#[macro_use] extern crate log;
extern crate gtk;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate breeze_core;
extern crate breeze_frontends;

#[macro_use]
mod clone;
mod data;
mod view;
mod model;

use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    gtk::init().unwrap();

    let model = Rc::new(RefCell::new(model::Model::new()));
    let view = view::MainView::new(model.clone());
    let weak_view = view.get_weak_ref_to_view();
    model.borrow_mut().set_view(weak_view);
    view.main_loop();
}
