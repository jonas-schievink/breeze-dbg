use view::View;

use breeze_core::rom::Rom;
use breeze_core::snes::Snes;
use breeze_core::save::SaveStateFormat;
use breeze_frontends::frontend::dummy::{DummyRenderer, DummySink};

use std::io::{self, Read};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::fs::File;

pub struct Model {
    savestate: Option<Vec<u8>>,
    rom: Option<Rom>,

    view: Option<Weak<View>>,
}

impl Model {
    /// Creates a new, uninitialized and empty model
    ///
    /// `Model::set_view` must be called before attempting to use it.
    pub fn new() -> Self {
        Model {
            savestate: None,
            rom: None,
            view: None,
        }
    }

    /// Set the reference to the view
    ///
    /// Initialization function, to be called once after model and view are created.
    pub fn set_view(&mut self, view: Weak<View>) {
        self.view = Some(view);
    }

    /// Load a ROM file from the given path
    pub fn load_rom(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = try!(File::open(path));
        let mut content = vec![];
        try!(file.read_to_end(&mut content));
        self.rom = Some(try!(Rom::from_bytes(&content)));

        self.update_frame()
    }

    pub fn load_save_state(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = try!(File::open(path));
        let mut content = vec![];
        try!(file.read_to_end(&mut content));
        self.savestate = Some(content);

        self.update_frame()
    }

    fn view(&self) -> Rc<View> {
        self.view.as_ref().expect("view reference unset").upgrade().expect("view was dropped")
    }

    /// Emulates one frame and renders the result on the view
    ///
    /// Does nothing if ROM is unset
    fn update_frame(&self) -> io::Result<()> {
        if let Some(ref rom) = self.rom {
            let sprites;
            let mut r = DummyRenderer::default();
            {
                let mut snes = Snes::new(rom.clone(), &mut r, Box::new(DummySink));
                if let Some(ref state) = self.savestate {
                    let mut reader = state as &[u8];
                    try!(snes.restore_save_state(SaveStateFormat::Custom, &mut reader));
                }
                snes.render_frame();

                // Collect sprites
                sprites = (0..128).map(|id| snes.peripherals().ppu.oam.get_sprite(id))
                                  .collect::<Vec<_>>();
            }

            self.view().update_frame_data(r.last_frame());
            self.view().update_oam(&sprites);
        }

        Ok(())
    }
}
