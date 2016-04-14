//! Main Window View

use model::Model;

use breeze_core::ppu::oam::OamEntry;

use gdk_pixbuf::{Pixbuf, InterpType};

use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Image, Orientation, Button};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub trait View {
    fn update_frame_data(&self, data: &[u8]);
    fn update_oam(&self, sprites: &[OamEntry]);
    fn error(&self, msg: &str);
}

pub struct MainView(Rc<RealMainView>);

struct RealMainView {
    win: Window,
    frame: Image,
    pixbuf: RefCell<Pixbuf>,
    btn_open_rom: Button,
    btn_open_save: Button,
    oam: gtk::TreeStore,

    model: Rc<RefCell<Model>>,
}

impl View for RealMainView {
    fn update_frame_data(&self, data: &[u8]) {
        const W: i32 = 256;
        const H: i32 = 224;
        const SCALE: i32 = 2;
        let pixbuf = Pixbuf::new_from_vec(Vec::from(data), 0, false, 8, W, H, W * 3);
        *self.pixbuf.borrow_mut() = pixbuf.scale_simple(W * SCALE, H * SCALE, InterpType::Nearest).unwrap();
        self.frame.set_from_pixbuf(Some(&self.pixbuf.borrow()));       // Display Updates
    }

    fn update_oam(&self, sprites: &[OamEntry]) {
        self.oam.clear();
        for (id, sprite) in sprites.iter().enumerate() {
            self.oam.insert_with_values(None, None, &[0, 1, 2, 3], &[&(id as u8), &(sprite.x as i32), &sprite.y, &"???"]);
        }
    }

    fn error(&self, msg: &str) {
        let dialog = gtk::MessageDialog::new(Some(&self.win),
                                             gtk::DialogFlags::empty(),
                                             gtk::MessageType::Error,
                                             gtk::ButtonsType::Close,
                                             msg);
        dialog.run();
        dialog.destroy();
    }
}

impl MainView {
    pub fn get_weak_ref_to_view(&self) -> Weak<View> {
        Rc::downgrade(&self.0) as Weak<View>
    }

    pub fn main_loop(&self) {
        self.0.win.show_all();
        gtk::main();
    }

    pub fn new(model: Rc<RefCell<Model>>) -> MainView {
        let this = MainView(Rc::new(RealMainView::build(model)));
        this.connect_events();
        this
    }

    /// Connects event handlers to GUI components. This is called after the GUI is set up, which
    /// neatly separates the look of the GUI from its behaviour.
    fn connect_events(&self) {
        let this = self.0.clone();
        self.0.btn_open_rom.connect_clicked(move |_| {
            let file_chooser = gtk::FileChooserDialog::new(Some("Open SNES ROM"), Some(&this.win), gtk::FileChooserAction::Open);
            file_chooser.add_buttons(&[
                ("Open", gtk::ResponseType::Ok as i32),
                ("Cancel", gtk::ResponseType::Cancel as i32),
            ]);

            let result = file_chooser.run();
            let filename = file_chooser.get_filename();
            file_chooser.destroy();
            drop(file_chooser);

            if result == gtk::ResponseType::Ok as i32 {
                match this.model.borrow_mut().load_rom(filename.unwrap()) {
                    Ok(_) => {},
                    Err(e) => this.error(&format!("Error while loading ROM: {}", e)),
                }
            }
        });

        let this = self.0.clone();
        self.0.btn_open_save.connect_clicked(move |_| {
            let file_chooser = gtk::FileChooserDialog::new(Some("Open Save State"), Some(&this.win), gtk::FileChooserAction::Open);
            file_chooser.add_buttons(&[
                ("Open", gtk::ResponseType::Ok as i32),
                ("Cancel", gtk::ResponseType::Cancel as i32),
            ]);

            let result = file_chooser.run();
            let filename = file_chooser.get_filename();
            file_chooser.destroy();
            drop(file_chooser);

            if result == gtk::ResponseType::Ok as i32 {
                match this.model.borrow_mut().load_save_state(filename.unwrap()) {
                    Ok(_) => {},
                    Err(e) => this.error(&format!("Error while loading save state: {}", e)),
                }
            }
        });
    }
}

/// Add a named column to a tree view, using a text renderer for the cells of this column
fn add_text_column(tree_view: &gtk::TreeView, title: &str) {
    let next_col = tree_view.get_columns().len();
    let render = gtk::CellRendererText::new();
    let column = gtk::TreeViewColumn::new();
    column.set_title(title);
    column.pack_start(&render, false);
    column.add_attribute(&render, "text", next_col as i32);
    tree_view.append_column(&column);
}

impl RealMainView {
    fn build_oam_treeview(&self) -> gtk::TreeView {
        let treeview = gtk::TreeView::new_with_model(&self.oam);
        add_text_column(&treeview, "#");
        add_text_column(&treeview, "X");
        add_text_column(&treeview, "Y");
        add_text_column(&treeview, "Size");
        treeview
    }

    fn fill_tools_notebook(&self, book: &gtk::Notebook) {
        let oam_view = self.build_oam_treeview();
        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.add(&oam_view);
        book.append_page(&scroll, Some(&gtk::Label::new(Some("OAM"))));
    }

    fn build(model: Rc<RefCell<Model>>) -> RealMainView {
        let this = RealMainView {
            win: Window::new(WindowType::Toplevel),
            frame: Image::new(),
            pixbuf: RefCell::new(unsafe { Pixbuf::new(0 /* RGB */, false, 8, 1, 1).unwrap() }),
            btn_open_rom: Button::new_with_label("Open ROM"),
            btn_open_save: Button::new_with_label("Open Save State"),
            oam: gtk::TreeStore::new(&[
                gtk::Type::String,
                gtk::Type::I32,
                gtk::Type::U8,
                gtk::Type::String,
            ]),

            model: model,
        };

        this.win.set_title("Breeze Emulator Tool");
        this.win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let tools = gtk::Notebook::new();
        this.fill_tools_notebook(&tools);

        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.set_border_width(5);
        scroll.set_shadow_type(gtk::ShadowType::In);
        scroll.add(&this.frame);

        let hsplit = gtk::Box::new(Orientation::Horizontal, 0);
        hsplit.pack_start(&scroll, true, true, 0);
        hsplit.pack_end(&tools, true, true, 0);

        let menu = gtk::Box::new(Orientation::Horizontal, 10);
        menu.set_border_width(5);
        menu.add(&this.btn_open_rom);
        menu.add(&this.btn_open_save);

        let vsplit = gtk::Box::new(Orientation::Vertical, 0);
        vsplit.pack_start(&menu, false, false, 0);
        vsplit.pack_end(&hsplit, true, true, 0);

        this.win.add(&vsplit);

        this
    }
}
