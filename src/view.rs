//! Main Window View

use model::Model;

use gdk_pixbuf::{Pixbuf, InterpType};

use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Image, Orientation, Button};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub trait View {
    fn update_frame_data(&self, data: &[u8]);
    fn error(&self, msg: &str);
}

pub struct MainView(Rc<RealMainView>);

struct RealMainView {
    win: Window,
    frame: Image,
    pixbuf: RefCell<Pixbuf>,
    btn_open_rom: Button,
    btn_open_save: Button,

    model: Rc<RefCell<Model>>,
}

impl View for RealMainView {
    fn update_frame_data(&self, data: &[u8]) {
        const W: i32 = 256;
        const H: i32 = 224;
        let pixbuf = Pixbuf::new_from_vec(Vec::from(data), 0, false, 8, W, H, W * 3);
        *self.pixbuf.borrow_mut() = pixbuf.scale_simple(W * 3, H * 3, InterpType::Nearest).unwrap();
        self.frame.set_from_pixbuf(Some(&self.pixbuf.borrow()));       // Display Updates
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

impl RealMainView {
    fn build(model: Rc<RefCell<Model>>) -> RealMainView {
        let this = RealMainView {
            win: Window::new(WindowType::Toplevel),
            frame: Image::new(),
            pixbuf: RefCell::new(unsafe { Pixbuf::new(0 /* RGB */, false, 8, 1, 1).unwrap() }),
            btn_open_rom: Button::new_with_label("Open ROM"),
            btn_open_save: Button::new_with_label("Open Save State"),

            model: model,
        };

        this.win.set_title("Breeze Emulator Tool");
        this.win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let tools = gtk::Notebook::new();
        tools.append_page(&gtk::TreeView::new(), Some(&gtk::Label::new(Some("OAM"))));

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
