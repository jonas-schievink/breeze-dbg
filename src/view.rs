//! Main Window View

use model::Model;
use data::ModelData;
use tools::{Tool, TOOLS};

use gdk_pixbuf::{Pixbuf, InterpType};

use gtk::prelude::*;
use gtk::{self, Window, WindowType, Image, Orientation, ToolButton};

use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub trait View {
    fn update_model_data(&self, data: &ModelData);
    fn update_frame(&self, frame: &[u8]);
    fn error(&self, msg: &str);
}

pub struct MainView(Rc<RealMainView>);

pub struct RealMainView {
    win: Window,
    frame: Image,
    pixbuf: RefCell<Pixbuf>,
    btn_open_rom: ToolButton,
    btn_open_save: ToolButton,
    btn_step_frame: ToolButton,

    tools: RefCell<Vec<Box<Tool>>>,

    pub model: Rc<RefCell<Model>>,
}

impl View for RealMainView {
    fn update_model_data(&self, data: &ModelData) {
        // Let tools update themselves
        for tool in &mut *self.tools.borrow_mut() {
            tool.update_model_data(data);
        }
    }

    fn update_frame(&self, frame: &[u8]) {
        const W: i32 = 256;
        const H: i32 = 224;
        const SCALE: i32 = 2;
        let pixbuf = Pixbuf::new_from_vec(Vec::from(frame), 0, false, 8, W, H, W * 3);
        *self.pixbuf.borrow_mut() = pixbuf.scale_simple(W * SCALE, H * SCALE, InterpType::Nearest).unwrap();
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
            let file_chooser = gtk::FileChooserDialog::new(
                Some("Open SNES ROM"),
                Some(&this.win),
                gtk::FileChooserAction::Open);
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
            let file_chooser = gtk::FileChooserDialog::new(
                Some("Open Save State"),
                Some(&this.win),
                gtk::FileChooserAction::Open);
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
        self.0.btn_step_frame.connect_clicked(move |_| {
            match this.model.borrow_mut().step() {
                Ok(_) => {},
                Err(e) => this.error(&format!("Error: {}", e)),
            }
        });

        let this = self.0.clone();
        for tool in &mut *self.0.tools.borrow_mut() {
            tool.connect_events(this.clone());
        }
    }
}

impl RealMainView {
    fn fill_tools_notebook(&mut self, book: &gtk::Notebook) {
        TOOLS.with(|tools| {
            let mut tool_store = self.tools.borrow_mut();
            for &cons_tool in tools {
                let mut tool = cons_tool();
                let scroll = gtk::ScrolledWindow::new(None, None);
                tool.init_tab(&scroll);
                book.append_page(&scroll, Some(&gtk::Label::new(Some(tool.get_name()))));

                tool_store.push(tool);
            }
        });
    }

    fn build(model: Rc<RefCell<Model>>) -> RealMainView {
        let mut this = RealMainView {
            win: Window::new(WindowType::Toplevel),
            frame: Image::new(),
            pixbuf: RefCell::new(unsafe { Pixbuf::new(0 /* RGB */, false, 8, 1, 1).unwrap() }),
            // FIXME The required generics are really ugly (and uncessary) here
            btn_open_rom: ToolButton::new(None::<&gtk::Box>, Some("Open ROM")),
            btn_open_save: ToolButton::new(None::<&gtk::Box>, Some("Open Save State")),
            btn_step_frame: ToolButton::new(None::<&gtk::Box>, Some("Emulate Frame")),
            tools: RefCell::new(Vec::new()),

            model: model,
        };

        this.win.set_title("Breeze Emulator Tool");
        this.win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        let tools = gtk::Notebook::new();
        tools.set_border_width(5);
        this.fill_tools_notebook(&tools);

        let scroll = gtk::ScrolledWindow::new(None, None);
        scroll.set_border_width(5);
        scroll.set_shadow_type(gtk::ShadowType::In);
        scroll.set_size_request(150, 0);
        scroll.add(&this.frame);

        let hsplit = gtk::Paned::new(gtk::Orientation::Horizontal);
        //hsplit.set_wide_handle(true); // FIXME Depends on GTK 3.16
        hsplit.pack1(&scroll, true, true);
        hsplit.pack2(&tools, true, true);

        let menu = gtk::Toolbar::new();
        menu.set_border_width(5);
        menu.add(&this.btn_open_rom);
        menu.add(&this.btn_open_save);
        menu.add(&this.btn_step_frame);

        let vsplit = gtk::Box::new(Orientation::Vertical, 0);
        vsplit.pack_start(&menu, false, false, 0);
        vsplit.pack_end(&hsplit, true, true, 0);

        this.win.add(&vsplit);

        this
    }
}
