//! Various GTK utilities

use gtk::{self, TreeView, CellRendererPixbuf, CellRendererText, TreeViewColumn, Frame, CheckButton,
    Orientation, Label, ComboBoxText};
use gtk::prelude::*;

pub fn add_pixbuf_column(tree_view: &TreeView, title: &str) {
    let next_col = tree_view.get_columns().len();
    let render = CellRendererPixbuf::new();
    let column = TreeViewColumn::new();
    column.set_title(title);
    column.pack_start(&render, false);
    column.add_attribute(&render, "pixbuf", next_col as i32);
    tree_view.append_column(&column);
}

/// Add a named column to a tree view, using a text renderer for the cells of this column
pub fn add_text_column(tree_view: &TreeView, title: &str) {
    let next_col = tree_view.get_columns().len();
    let render = CellRendererText::new();
    let column = TreeViewColumn::new();
    column.set_title(title);
    column.pack_start(&render, false);
    column.add_attribute(&render, "text", next_col as i32);
    tree_view.append_column(&column);
}

/// Creates a frame with title, containing 5 `CheckButtons` that will be stored in `layers`: BG1-4
/// and OBJ (and optionally Backdrop).
///
/// The frame will contain (laid out horizontally), a label with `descr` and the `CheckButtons`.
pub fn ppu_layers_frame(title: &str,
                        descr: &str,
                        layers: &mut Vec<CheckButton>,
                        backdrop: bool)
                        -> Frame {
    layers.push(CheckButton::new_with_label("BG1"));
    layers.push(CheckButton::new_with_label("BG2"));
    layers.push(CheckButton::new_with_label("BG3"));
    layers.push(CheckButton::new_with_label("BG4"));
    layers.push(CheckButton::new_with_label("OBJ"));
    if backdrop {
        layers.push(CheckButton::new_with_label("Backdrop"));
    }

    for btn in layers.iter() { btn.set_sensitive(false); } // FIXME implement changing registers

    let frame = Frame::new(Some(title));
    let hbox = gtk::Box::new(Orientation::Horizontal, 5);
    hbox.set_border_width(5);

    hbox.pack_start(&Label::new(Some(descr)), false, true, 0);
    for layer in layers {
        hbox.pack_start(layer, false, true, 0);
    }

    frame.add(&hbox);
    frame
}

/// Creates a `ComboBoxText` (dropdown box) with the given entries.
pub fn combo_box_text(entries: &[&str]) -> ComboBoxText {
    let cb = ComboBoxText::new();
    for entry in entries {
        cb.append_text(entry);
    }
    cb
}
