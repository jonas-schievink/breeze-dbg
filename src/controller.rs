use std::io;
use std::path::PathBuf;

pub struct Controller {

}

impl Controller {
    /// Called when the user selects a ROM file to load
    ///
    /// Loads the given path into the model
    pub fn load_rom(&self, path: PathBuf) -> io::Result<()> {
        Ok(())
    }
}
