//! Main Window View

use gdk_pixbuf::Pixbuf;

use gtk;
use gtk::prelude::*;
use gtk::{Window, WindowType, Image, Orientation, Button};

pub struct MainView {
    win: Window,
    frame: Image,
    pixbuf: Box<Pixbuf>,
}

impl MainView {
    pub fn new() -> MainView {
        let win = Window::new(WindowType::Toplevel);
        win.set_title("Breeze Test Case Reducer");
        win.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });

        const W: i32 = 128;
        const H: i32 = 128;
        let pixbuf = Box::new(Pixbuf::new_from_vec(vec![255u8; W as usize*H as usize*3], 0, false, 8, W, H, W * 3));
        let img = Image::new_from_pixbuf(Some(&*pixbuf));

        let testbtn = Button::new_with_label("Test Button");
        let btn2 = Button::new_with_label("Button2");

        let tools = gtk::Box::new(Orientation::Vertical, 10);
        tools.add(&testbtn);
        tools.add(&btn2);

        let hsplit = gtk::Box::new(Orientation::Horizontal, 10);
        hsplit.add(&img);
        hsplit.pack_end(&tools, false, false, 0);

        let menu = gtk::Box::new(Orientation::Horizontal, 10);
        let pixbuf2 = pixbuf.clone();
        let img2 = img.clone();
        menu.add(&menu_button("Open ROM", move |_| {
            pixbuf2.put_pixel(0, 0, 255, 0, 0, 255);    // Change
            img2.set_from_pixbuf(Some(&pixbuf2));       // Display Updates


            let file_chooser = gtk::FileChooserDialog::new(Some("Open SNES ROM"), None, gtk::FileChooserAction::Open);
            file_chooser.add_buttons(&[
                ("Open", gtk::ResponseType::Ok as i32),
                ("Cancel", gtk::ResponseType::Cancel as i32),
            ]);

            if file_chooser.run() == gtk::ResponseType::Ok as i32 {
                let filename = file_chooser.get_filename().unwrap();
                // TODO Tell controller
            }
        }));
        menu.add(&menu_button("Open Save State", |_| ()));

        let vsplit = gtk::Box::new(Orientation::Vertical, 10);
        vsplit.add(&menu);
        vsplit.add(&hsplit);

        win.add(&vsplit);

        MainView {
            win: win,
            frame: img,
            pixbuf: pixbuf,
        }
    }

    pub fn main_loop(&mut self) {
        self.win.show_all();
        gtk::main();
    }
}

fn menu_button<F>(label: &str, action: F) -> Button where F: Fn(&Button) + 'static {
    let btn = Button::new_with_label(label);
    btn.connect_clicked(action);
    btn
}
