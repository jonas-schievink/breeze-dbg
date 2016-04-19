#[macro_use] extern crate log;
#[macro_use] extern crate lazy_static;
extern crate env_logger;
extern crate clap;
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
mod util;
mod tools;

use clap::{App, Arg};

use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

fn main() {
    env_logger::init().unwrap();

    gtk::init().unwrap();

    let matches = App::new("Breeze Emulator Tool")
                      .author("Jonas Schievink <jonas@schievink.net>")
                      .about("GUI tool for introspection of Breeze save states")
                      .arg(Arg::with_name("rom").takes_value(true))
                      .arg(Arg::with_name("state").takes_value(true))
                      .get_matches();

    let model = Rc::new(RefCell::new(model::Model::new()));
    let view = view::MainView::new(model.clone());
    let weak_view = view.get_weak_ref_to_view();

    {
        let mut model = model.borrow_mut();
        model.set_view(weak_view);
        if let Some(rom) = matches.value_of("rom") {
            model.load_rom(PathBuf::from(rom)).unwrap();    // FIXME dont unwrap
        }
        if let Some(state) = matches.value_of("state") {
            model.load_save_state(PathBuf::from(state)).unwrap();
        }
    }
    view.main_loop();
}
