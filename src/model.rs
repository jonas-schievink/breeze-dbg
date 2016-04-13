use view::View;

use breeze_core::rom::Rom;
use breeze_core::snes::Snes;
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
    pub fn new() -> Self {
        Model {
            savestate: None,
            rom: None,
            view: None,
        }
    }

    pub fn set_view(&mut self, view: Weak<View>) {
        self.view = Some(view);
    }

    pub fn load_rom(&mut self, path: PathBuf) -> io::Result<()> {
        let mut file = try!(File::open(path));
        let mut content = vec![];
        try!(file.read_to_end(&mut content));
        self.rom = match Rom::from_bytes(&content) {
            Ok(rom) => Some(rom),
            Err(()) => {
                self.view().error("Error loading ROM (invalid ROM?)");
                None
            }
        };

        self.update_frame();
        Ok(())
    }

    fn view(&self) -> Rc<View> {
        self.view.as_ref().unwrap().upgrade().unwrap()
    }

    /// Emulates one frame and renders the result on the view
    ///
    /// Does nothing if ROM is unset
    fn update_frame(&self) {
        if let Some(ref rom) = self.rom {
            // TODO Restore save state
            let mut r = DummyRenderer::default();
            {
                let mut snes = Snes::new(rom.clone(), &mut r, Box::new(DummySink));
                snes.render_frame();
            }

            self.view().update_frame_data(r.last_frame());
        }
    }
}
