//! View PPU register values (broken down to the individual bit)

use super::Tool;
use view::RealMainView;
use util::*;
use data::ModelData;

use breeze_core::ppu::Ppu;

use gtk::{self, TreeView, ListStore, ScrolledWindow, Orientation, Frame, CheckButton, ComboBoxText,
    Label};
use gtk::prelude::*;

use std::rc::Rc;

#[derive(Clone)]    //:
pub struct PpuRegs {
    regs: ListStore,
    fblank: CheckButton,
    brightness: gtk::Scale,
    objsize: ComboBoxText,
    bgmode: Label,
    bg_tilesizes: Label,
}

impl PpuRegs {
    fn inidisp_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2100 - INIDISP"));
        let inidisp = gtk::Box::new(Orientation::Horizontal, 5);
        inidisp.set_border_width(5);

        let brightness_lbl = Label::new(Some("Brightness: "));

        inidisp.pack_start(&self.fblank, false, true, 0);
        inidisp.pack_start(&brightness_lbl, false, true, 0);
        inidisp.pack_start(&self.brightness, true, true, 0);

        frame.add(&inidisp);
        frame
    }

    fn obsel_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2101 - OBSEL"));
        let obsel = gtk::Box::new(Orientation::Horizontal, 5);
        obsel.set_border_width(5);

        obsel.pack_start(&self.objsize, false, true, 0);

        frame.add(&obsel);
        frame
    }

    fn bgmode_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2105 - BGMODE"));
        let bgmode = gtk::Box::new(Orientation::Horizontal, 5);
        bgmode.set_border_width(5);

        bgmode.pack_start(&self.bgmode, false, true, 0);
        bgmode.pack_start(&self.bg_tilesizes, false, true, 0);

        frame.add(&bgmode);
        frame
    }
}

impl Tool for PpuRegs {
    fn new() -> Self {
        let objsize = ComboBoxText::new();
        objsize.append_text("8x8 and 16x16 sprites");
        objsize.append_text("8x8 and 32x32 sprites");
        objsize.append_text("8x8 and 64x64 sprites");
        objsize.append_text("16x16 and 32x32 sprites");
        objsize.append_text("16x16 and 64x64 sprites");
        objsize.append_text("32x32 and 64x64 sprites");
        objsize.append_text("16x32 and 32x64 sprites");
        objsize.append_text("16x32 and 32x32 sprites");

        PpuRegs {
            regs: ListStore::new(&[
                gtk::Type::String,  // Address (Hex `u16`)
                gtk::Type::String,  // Name
                gtk::Type::String,  // Raw value (Hex `u8`)
            ]),
            fblank: CheckButton::new_with_label("F-Blank"),
            brightness: gtk::Scale::new_with_range(Orientation::Horizontal, 0.0, 15.0, 1.0),
            objsize: objsize,
            bgmode: Label::new(None),
            bg_tilesizes: Label::new(None),
        }
    }

    fn get_name(&self) -> &'static str { "PPU Regs" }

    fn init_tab(&mut self, win: &ScrolledWindow) {
        let left_column = gtk::Box::new(Orientation::Vertical, 5);
        left_column.set_border_width(5);

        left_column.pack_start(&self.inidisp_frame(), false, true, 0);
        left_column.pack_start(&self.obsel_frame(), false, true, 0);
        left_column.pack_start(&self.bgmode_frame(), false, true, 0);

        let treeview = TreeView::new_with_model(&self.regs);
        add_text_column(&treeview, "Addr");
        add_text_column(&treeview, "Name");
        add_text_column(&treeview, "Raw");

        let hbox = gtk::Paned::new(Orientation::Horizontal);
        hbox.pack1(&left_column, true, true);
        hbox.pack2(&treeview, false, true);
        win.add(&hbox);
    }

    fn connect_events(&mut self, _view: Rc<RealMainView>) {
        // FIXME Changing state through this tool isn't yet supported, so disable the controls
        self.fblank.set_sensitive(false);
        self.brightness.set_sensitive(false);
        self.objsize.set_sensitive(false);
    }

    fn update_model_data(&mut self, data: &ModelData) {
        let inidisp = data.ppu.inidisp();
        let fblank = inidisp & 0x80 != 0;
        let brightness = inidisp & 0x0f;
        self.fblank.set_active(fblank);
        self.brightness.set_value(brightness as f64);

        let obsel = data.ppu.obsel();
        self.objsize.set_active(((obsel & 0b11100000) >> 5) as i32);

        let bgmode = data.ppu.bgmode();
        self.bgmode.set_label(&format!("BG Mode {}.  ", bgmode & 0b111));
        let bgtiles = (1..5).map(|bg| {
            let tilesize = if bgmode & 0x80 << bg == 0 { 8 } else { 16 };
            format!("BG {} tiles: {}x{}", bg, tilesize, tilesize)
        }).collect::<Vec<_>>().join("; ");
        self.bg_tilesizes.set_label(&bgtiles);

        // Update raw register values on the right
        static RAW_REGS: &'static [(u16, &'static str, fn(&Ppu) -> u8)] = &[
            (0x2100, "INIDISP", Ppu::inidisp),
            (0x2101, "OBSEL", Ppu::obsel),
            (0x2105, "BGMODE", Ppu::bgmode),
            (0x2106, "MOSAIC", Ppu::mosaic),
            (0x211a, "M7SEL", Ppu::m7sel),
            (0x2123, "W12SEL", Ppu::w12sel),
            (0x2124, "W34SEL", Ppu::w34sel),
            (0x2125, "WOBJSEL", Ppu::wobjsel),
            (0x212a, "WBGLOG", Ppu::wbglog),
            (0x212b, "WOBJLOG", Ppu::wobjlog),
            (0x212c, "TM", Ppu::tm),
            (0x212d, "TS", Ppu::ts),
            (0x212e, "TMW", Ppu::tmw),
            (0x212f, "TSW", Ppu::tsw),
            (0x2130, "CGWSEL", Ppu::cgwsel),
            (0x2131, "CGADSUB", Ppu::cgadsub),
            (0x2133, "SETINI", Ppu::setini),
        ];

        let entry_count = self.regs.iter_n_children(None) as usize;
        for _ in entry_count..RAW_REGS.len() {
            self.regs.append();
        }

        let mut child = self.regs.iter_children(None).unwrap();
        for &(addr, name, fun) in RAW_REGS {
            let value = fun(&data.ppu);
            self.regs.set(&child, &[0, 1, 2], &[
                &format!("${:04X}", addr),
                &name,
                &format!("${:02X}", value),
            ]);

            self.regs.iter_next(&mut child);
        }
    }
}
