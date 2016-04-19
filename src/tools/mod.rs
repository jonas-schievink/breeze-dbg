//! Tools manage tabs. They'll usually display some information about the emulator state.

mod cgram;
mod oam;

use view::RealMainView;
use data::ModelData;

use gtk::ScrolledWindow;

use std::rc::Rc;

fn cons_tool<T: Tool + 'static>() -> Box<Tool> { Box::new(T::new()) }

thread_local! {
    pub static TOOLS: [fn() -> Box<Tool>; 2] = [
        cons_tool::<oam::Oam>,
        cons_tool::<cgram::Cgram>,
    ]
}

pub trait Tool {
    fn new() -> Self where Self: Sized;
    fn get_name(&self) -> &'static str;
    fn init_tab(&mut self, win: &ScrolledWindow);
    fn connect_events(&mut self, view: Rc<RealMainView>);
    fn update_model_data(&mut self, data: &ModelData);
}
