use blank_rom::load_blank_rom;
use view::View;
use data::*;

use breeze_core::rom::Rom;
use breeze_core::snes::Snes;
use breeze_core::save::SaveStateFormat;
use breeze_core::ppu::FrameBuf;

use std::io::{self, Read};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::fs::File;

pub struct Model {
    snes: Snes,
    view: Option<Weak<View>>,
}

impl Model {
    /// Creates a new, uninitialized and empty model
    ///
    /// `Model::set_view` must be called before attempting to use it.
    pub fn new() -> Self {
        Model {
            snes: Snes::new(load_blank_rom()),
            view: None,
        }
    }

    /// Set the reference to the view
    ///
    /// Initialization function, to be called once after model and view are created.
    pub fn set_view(&mut self, view: Weak<View>) {
        self.view = Some(view);
        self.update_frame();
    }

    /// Load a ROM file from the given path
    pub fn load_rom(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = try!(File::open(path));
        let mut content = vec![];
        try!(file.read_to_end(&mut content));
        self.snes = Snes::new(try!(Rom::from_bytes(&content)));

        self.update_frame();
        Ok(())
    }

    pub fn load_save_state(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = try!(File::open(path));
        let mut content = vec![];
        try!(file.read_to_end(&mut content));
        let mut reader = &*content;
        try!(self.snes.restore_save_state(SaveStateFormat::Custom, &mut reader));

        self.update_frame();
        Ok(())
    }

    /// Advance by a frame
    ///
    /// More accurately, this will run emulation until the last pixel of the frame is rendered.
    pub fn step(&mut self) {
        self.snes.render_frame(|_| None);
        self.update_frame();
    }

    /// Set a color value in CGRAM to a different raw value
    pub fn set_cgram(&mut self, index: u8, raw: u16) {
        self.snes.peripherals_mut().ppu.cgram.set_color_raw(index, raw);
        self.update_frame();
    }

    /// Unwraps the reference to the `View`
    fn view(&self) -> Rc<View> {
        self.view.as_ref().expect("view reference unset").upgrade().expect("view was dropped")
    }

    fn create_save_state(&self) -> Vec<u8> {
        let mut save = vec![];
        self.snes.create_save_state(SaveStateFormat::Custom, &mut save).unwrap();   // can't fail
        save
    }

    /// Emulates one frame and renders the result on the view
    ///
    /// Does nothing if ROM is unset
    fn update_frame(&mut self) {
        // Create a save state, render frame, restore save state
        let save = self.create_save_state();

        let mut framebuf = FrameBuf::default();
        self.snes.render_frame(|fb| {
            framebuf = fb.clone();
            None
        });
        let mut reader = &*save;

        // Collect sprites
        let sprites = (0..128).map(|id| self.snes.peripherals().ppu.oam.get_sprite(id))
                              .map(|entry| Sprite::new(&self.snes.peripherals().ppu, &entry))
                              .collect::<Vec<_>>();

        // Update everything, then roll back
        self.view().update_model_data(&ModelData {
            sprites: &sprites,
            ppu: &self.snes.peripherals().ppu,
        });
        self.view().update_frame(&*framebuf);
        self.update_info();

        self.snes.restore_save_state(SaveStateFormat::Custom, &mut reader).unwrap();
    }

    fn update_info(&self) {
        let rom_name = self.snes.peripherals().rom.get_title().unwrap_or("<none>");
        let info = format!("\
            ROM name: {}\n\
            H position: {}\n\
            V position: {}",
            rom_name, self.snes.peripherals().ppu.h_counter(), self.snes.peripherals().ppu.v_counter());

        self.view().update_info(&info);
    }
}
