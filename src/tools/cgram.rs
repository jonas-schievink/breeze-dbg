//! View and edit CGRAM

use super::Tool;
use view::{View, RealMainView};
use util::*;
use data::ModelData;

use gdk::enums::key;
use gdk_pixbuf::{Pixbuf, InterpType};
use gtk::{self, TreeView, ListStore, ScrolledWindow};
use gtk::prelude::*;

use std::rc::Rc;

#[derive(Clone)]    //:
pub struct Cgram {
    treeview: TreeView,
    cgram: ListStore,
}

impl Tool for Cgram {
    fn new() -> Self {
        let model = ListStore::new(&[
            gtk::Type::U8,      // #
            Pixbuf::static_type(),  // Color
            gtk::Type::String,  // Raw (Hex)
            gtk::Type::U8,      // R
            gtk::Type::U8,      // G
            gtk::Type::U8,      // B
        ]);
        Cgram {
            treeview: TreeView::new_with_model(&model),
            cgram: model,
        }
    }

    fn get_name(&self) -> &'static str { "CGRAM" }

    fn init_tab(&mut self, win: &ScrolledWindow) {
        self.treeview.set_model(Some(&self.cgram));
        self.treeview.set_rubber_banding(true);
        self.treeview.get_selection().set_mode(gtk::SelectionMode::Multiple);
        add_text_column(&self.treeview, "#");
        add_pixbuf_column(&self.treeview, "Color");
        add_text_column(&self.treeview, "Raw");
        add_text_column(&self.treeview, "R");
        add_text_column(&self.treeview, "G");
        add_text_column(&self.treeview, "B");

        win.add(&self.treeview);
    }

    fn connect_events(&mut self, view: Rc<RealMainView>) {
        let this = self.clone();
        self.treeview.connect_key_press_event(move |_, event| {
            match event.get_keyval() {
                key::Delete => {
                    for row in this.treeview.get_selection().get_selected_rows().0 {
                        let index = row.get_indices()[0];
                        match view.model.borrow_mut().set_cgram(index as u8, 0) {
                            Ok(_) => {},
                            Err(e) => view.error(&format!("Error: {}", e)),
                        }
                    }
                }
                _ => {}
            }

            Inhibit(false)
        });
    }

    fn update_model_data(&mut self, data: &ModelData) {
        let entry_count = self.cgram.iter_n_children(None) as usize;
        for _ in entry_count..256 {
            self.cgram.append();
        }
        for id in 0..256u16 {
            let id = id as u8;
            // FIXME Not sure if we should display adjusted RGB value...
            let raw = data.ppu.cgram.get_color_raw(id);
            let rgb = data.ppu.cgram.get_color(id).to_adjusted_rgb();
            let entry = self.cgram.iter_nth_child(None, id as i32).expect(&format!("child #{} not found", id));

            // Render color to pixbuf
            // FIXME Make pixbuf size depend on row height
            let pixbuf = Pixbuf::new_from_vec(vec![rgb.r, rgb.g, rgb.b], 0, false, 8, 1, 1, 3);
            let pixbuf = pixbuf.scale_simple(16, 16, InterpType::Nearest).unwrap();

            self.cgram.set(&entry, &[0, 1, 2, 3, 4, 5], &[
                &id,
                &pixbuf,
                &format!("0x{:04X}", raw),
                &rgb.r,
                &rgb.g,
                &rgb.b,
            ]);
        }
    }
}
