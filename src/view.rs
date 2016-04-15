//! Main Window View

use model::Model;
use data::ModelData;

use gdk_pixbuf::{Pixbuf, InterpType};

use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Image, Orientation, Button};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub trait View {
    fn update_model_data(&self, data: &ModelData);
    fn error(&self, msg: &str);
}

pub struct MainView(Rc<RealMainView>);

struct RealMainView {
    win: Window,
    frame: Image,
    pixbuf: RefCell<Pixbuf>,
    btn_open_rom: Button,
    btn_open_save: Button,
    btn_step: Button,
    oam: gtk::ListStore,
    cgram: gtk::ListStore,

    model: Rc<RefCell<Model>>,
}

impl View for RealMainView {
    fn update_model_data(&self, data: &ModelData) {
        //---- Frame

        const W: i32 = 256;
        const H: i32 = 224;
        const SCALE: i32 = 2;
        let pixbuf = Pixbuf::new_from_vec(Vec::from(data.frame), 0, false, 8, W, H, W * 3);
        *self.pixbuf.borrow_mut() = pixbuf.scale_simple(W * SCALE, H * SCALE, InterpType::Nearest).unwrap();
        self.frame.set_from_pixbuf(Some(&self.pixbuf.borrow()));       // Display Updates

        //---- Sprites

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

        //----- CGRAM

        let entry_count = self.cgram.iter_n_children(None) as usize;
        for _ in entry_count..256 {
            self.cgram.append();
        }
        for id in 0..256u16 {
            let id = id as u8;
            // FIXME Not sure if we should display adjusted RGB value...
            let raw = data.cgram.get_color_raw(id);
            let rgb = data.cgram.get_color(id);
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

        let this = self.0.clone();
        self.0.btn_step.connect_clicked(move |_| {
            match this.model.borrow_mut().step() {
                Ok(_) => {},
                Err(e) => this.error(&format!("Error: {}", e)),
            }
        });
    }
}

fn add_pixbuf_column(tree_view: &gtk::TreeView, title: &str) {
    let next_col = tree_view.get_columns().len();
    let render = gtk::CellRendererPixbuf::new();
    let column = gtk::TreeViewColumn::new();
    column.set_title(title);
    column.pack_start(&render, false);
    column.add_attribute(&render, "pixbuf", next_col as i32);
    tree_view.append_column(&column);
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
        add_text_column(&treeview, "Tile Addr.");
        add_text_column(&treeview, "Priority");
        add_text_column(&treeview, "Color #0");
        add_text_column(&treeview, "HFlip");
        add_text_column(&treeview, "VFlip");
        treeview
    }

    fn build_cgram_treeview(&self) -> gtk::TreeView {
        let treeview = gtk::TreeView::new_with_model(&self.cgram);
        add_text_column(&treeview, "#");
        add_pixbuf_column(&treeview, "Color");
        add_text_column(&treeview, "Raw");
        add_text_column(&treeview, "R");
        add_text_column(&treeview, "G");
        add_text_column(&treeview, "B");
        treeview
    }

    fn fill_tools_notebook(&self, book: &gtk::Notebook) {
        let oam_view = self.build_oam_treeview();
        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.add(&oam_view);
        book.append_page(&scroll, Some(&gtk::Label::new(Some("OAM"))));

        let cgram_view = self.build_cgram_treeview();
        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.add(&cgram_view);
        book.append_page(&scroll, Some(&gtk::Label::new(Some("CGRAM"))));
    }

    fn build(model: Rc<RefCell<Model>>) -> RealMainView {
        let this = RealMainView {
            win: Window::new(WindowType::Toplevel),
            frame: Image::new(),
            pixbuf: RefCell::new(unsafe { Pixbuf::new(0 /* RGB */, false, 8, 1, 1).unwrap() }),
            btn_open_rom: Button::new_with_label("Open ROM"),
            btn_open_save: Button::new_with_label("Open Save State"),
            btn_step: Button::new_with_label("Emulate Frame"),
            oam: gtk::ListStore::new(&[
                gtk::Type::U8,      // #
                gtk::Type::I32,     // X
                gtk::Type::U8,      // Y
                gtk::Type::String,  // Size
                gtk::Type::String,  // Tile addr (Hex)
                gtk::Type::U8,      // Prio
                gtk::Type::U8,      // Palette
                gtk::Type::Bool,    // HFlip
                gtk::Type::Bool,    // VFlip
            ]),
            cgram: gtk::ListStore::new(&[
                gtk::Type::U8,      // #
                Pixbuf::static_type(),  // Color
                gtk::Type::String,  // Raw (Hex)
                gtk::Type::U8,      // R
                gtk::Type::U8,      // G
                gtk::Type::U8,      // B
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
        menu.add(&this.btn_step);

        let vsplit = gtk::Box::new(Orientation::Vertical, 0);
        vsplit.pack_start(&menu, false, false, 0);
        vsplit.pack_end(&hsplit, true, true, 0);

        this.win.add(&vsplit);

        this
    }
}
