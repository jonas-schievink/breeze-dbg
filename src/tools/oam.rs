//! View OAM contents

use super::Tool;
use view::RealMainView;
use util::*;
use data::ModelData;

use gtk::{self, TreeView, ListStore, ScrolledWindow};
use gtk::prelude::*;

use std::rc::Rc;

#[derive(Clone)]    //:
pub struct Oam {
    oam: ListStore,
}

impl Tool for Oam {
    fn new() -> Self {
        let model = ListStore::new(&[
            gtk::Type::U8,      // #
            gtk::Type::I32,     // X
            gtk::Type::U8,      // Y
            gtk::Type::String,  // Size
            gtk::Type::String,  // Tile addr (Hex)
            gtk::Type::U8,      // Prio
            gtk::Type::U8,      // Palette
            gtk::Type::Bool,    // HFlip
            gtk::Type::Bool,    // VFlip
        ]);
        Oam {
            oam: model,
        }
    }

    fn get_name(&self) -> &'static str { "OAM" }

    fn init_tab(&mut self, win: &ScrolledWindow) {
        let treeview = TreeView::new_with_model(&self.oam);
        add_text_column(&treeview, "#");
        add_text_column(&treeview, "X");
        add_text_column(&treeview, "Y");
        add_text_column(&treeview, "Size");
        add_text_column(&treeview, "Tile Addr.");
        add_text_column(&treeview, "Priority");
        add_text_column(&treeview, "Color #0");
        add_text_column(&treeview, "HFlip");
        add_text_column(&treeview, "VFlip");

        win.add(&treeview);
    }

    fn connect_events(&mut self, _view: Rc<RealMainView>) {
    }

    fn update_model_data(&mut self, data: &ModelData) {
        // Clearing and refilling the `ListStore` causes the `TreeView` to scroll up, which I don't
        // want. So we ensure that there are enough entries and modify them.
        let entry_count = self.oam.iter_n_children(None) as usize;
        for _ in entry_count..data.sprites.len() {
            self.oam.append();
        }

        for (id, sprite) in data.sprites.iter().enumerate() {
            let entry = self.oam.iter_nth_child(None, id as i32).expect(&format!("child #{} not found", id));

            self.oam.set(&entry, &[0, 1, 2, 3, 4, 5, 6, 7, 8], &[
                &(id as u8),
                &(sprite.x as i32),
                &sprite.y,
                &format!("{}x{}", sprite.size.0, sprite.size.1),
                &format!("0x{:04X}", sprite.tile_addr),
                &sprite.priority,
                &sprite.color_start,
                &sprite.hflip,
                &sprite.vflip,
            ]);
        }
    }
}
