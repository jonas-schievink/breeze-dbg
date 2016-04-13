#[macro_use] extern crate log;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate breeze_core;
extern crate breeze_frontends;

#[macro_use]
mod clone;
mod view;
mod model;

use std::rc::{Rc, Weak};
use std::cell::RefCell;

fn main() {
    gtk::init().unwrap();

    let model = Rc::new(RefCell::new(model::Model::new()));
    let view = Rc::new(view::MainView::new(model.clone()));
    let weak_view = Rc::downgrade(&view) as Weak<view::View>;
    model.borrow_mut().set_view(weak_view);
    view.main_loop();
}
