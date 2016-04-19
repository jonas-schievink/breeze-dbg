//! Various GTK utilities

use gtk::{TreeView, CellRendererPixbuf, CellRendererText, TreeViewColumn};
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
