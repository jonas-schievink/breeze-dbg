//! Main Window View

use model::Model;

use gdk_pixbuf::Pixbuf;

use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Image, Orientation, Button};

use std::rc::Rc;
use std::cell::{RefCell, RefMut};

pub trait View {
    fn update_frame_data(&self, data: &[u8]);
    fn error(&self, msg: &str);
}

pub struct MainView {
    win: Window,
    frame: Image,
    pixbuf: Pixbuf,

    model: Rc<RefCell<Model>>,
}

impl View for MainView {
    fn update_frame_data(&self, data: &[u8]) {
        // TODO: Recreate and scale Pixbuf with new data

        self.frame.set_from_pixbuf(Some(&self.pixbuf));       // Display Updates
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
    fn model_mut(&mut self) -> RefMut<Model> {
        self.model.borrow_mut()
    }

    pub fn new(model: Rc<RefCell<Model>>) -> MainView {
        let win = Window::new(WindowType::Toplevel);
        win.set_title("Breeze Test Case Reducer");
        win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        const W: i32 = 128;
        const H: i32 = 128;
        let pixbuf = Pixbuf::new_from_vec(vec![255u8; W as usize*H as usize*3], 0, false, 8, W, H, W * 3);
        let frame = Image::new_from_pixbuf(Some(&pixbuf));

        let testbtn = Button::new_with_label("Test Button");
        let btn2 = Button::new_with_label("Button2");

        let tools = gtk::Box::new(Orientation::Vertical, 10);
        tools.add(&testbtn);
        tools.add(&btn2);

        let hsplit = gtk::Box::new(Orientation::Horizontal, 10);
        hsplit.add(&frame);
        hsplit.pack_end(&tools, false, false, 0);

        let menu = gtk::Box::new(Orientation::Horizontal, 10);
        menu.add(&menu_button("Open ROM", clone!(frame, win, pixbuf, model => move |_| {
            pixbuf.put_pixel(0, 0, 255, 0, 0, 255);     // Change
            frame.set_from_pixbuf(Some(&pixbuf));       // Display Updates


            let file_chooser = gtk::FileChooserDialog::new(Some("Open SNES ROM"), Some(&win), gtk::FileChooserAction::Open);
            file_chooser.add_buttons(&[
                ("Open", gtk::ResponseType::Ok as i32),
                ("Cancel", gtk::ResponseType::Cancel as i32),
            ]);

            let result = file_chooser.run();
            let filename = file_chooser.get_filename();
            file_chooser.destroy();
            drop(file_chooser);

            if result == gtk::ResponseType::Ok as i32 {
                // FIXME Make Rom::from_bytes return an io::Result and propagate it to here
                model.borrow_mut().load_rom(filename.unwrap()).unwrap();
            }
        })));
        menu.add(&menu_button("Open Save State", |_| ()));

        let vsplit = gtk::Box::new(Orientation::Vertical, 10);
        vsplit.add(&menu);
        vsplit.add(&hsplit);

        win.add(&vsplit);

        MainView {
            win: win,
            frame: frame,
            pixbuf: pixbuf,
            model: model,
        }
    }

    pub fn main_loop(&self) {
        self.win.show_all();
        gtk::main();
    }
}

fn menu_button<F>(label: &str, action: F) -> Button where F: Fn(&Button) + 'static {
    let btn = Button::new_with_label(label);
    btn.connect_clicked(action);
    btn
}
