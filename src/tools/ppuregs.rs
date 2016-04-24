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
    bgmode: ComboBoxText,
    bg_tilesizes: Vec<ComboBoxText>,
    mosaicsize: ComboBoxText,
    mosaicbgs: Vec<CheckButton>,    // Not an array because #[derive] sucks
    tm: Vec<CheckButton>,
    ts: Vec<CheckButton>,
    tmw: Vec<CheckButton>,
    tsw: Vec<CheckButton>,
    cgw_clip: ComboBoxText,
    cgw_prevent: ComboBoxText,
    cgw_subscreen: CheckButton,
    cgw_direct_color: CheckButton,
    math_add_sub: ComboBoxText,
    half_math: CheckButton,
    color_math: Vec<CheckButton>,
}

impl PpuRegs {
    fn inidisp_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2100 - INIDISP"));
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.set_border_width(5);

        let brightness_lbl = Label::new(Some("Brightness: "));

        hbox.pack_start(&self.fblank, false, true, 0);
        hbox.pack_start(&brightness_lbl, false, true, 0);
        hbox.pack_start(&self.brightness, true, true, 0);

        frame.add(&hbox);
        frame
    }

    fn obsel_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2101 - OBSEL"));
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.set_border_width(5);

        hbox.pack_start(&self.objsize, false, true, 0);

        frame.add(&hbox);
        frame
    }

    fn bgmode_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2105 - BGMODE"));
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.set_border_width(5);

        hbox.pack_start(&self.bgmode, false, true, 0);
        for i in (1..5).rev() {
            hbox.pack_end(&self.bg_tilesizes[i - 1], false, true, 0);
            hbox.pack_end(&Label::new(Some(&format!("BG {}:", i))), false, true, 0);
        }
        hbox.pack_end(&Label::new(Some("BG tile sizes:")), false, true, 0);

        frame.add(&hbox);
        frame
    }

    fn mosaic_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2106 - MOSAIC"));
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.set_border_width(5);

        hbox.pack_start(&Label::new(Some("Mosaic size:")), false, true, 0);
        hbox.pack_start(&self.mosaicsize, false, true, 0);
        for bg in self.mosaicbgs.iter().rev() {
            hbox.pack_end(bg, false, true, 0);
        }
        hbox.pack_end(&Label::new(Some("Mosaic enabled on: ")), false, true, 0);

        frame.add(&hbox);
        frame
    }

    fn tm_frame(&mut self) -> Frame {
        ppu_layers_frame("$212c - TM", "Main Screen Layers enabled:", &mut self.tm, false)
    }

    fn ts_frame(&mut self) -> Frame {
        ppu_layers_frame("$212d - TS", "Sub Screen Layers enabled:", &mut self.ts, false)
    }

    fn tmw_frame(&mut self) -> Frame {
        ppu_layers_frame("$212e - TMW", "Main Screen Windows enabled:", &mut self.tmw, false)
    }

    fn tsw_frame(&mut self) -> Frame {
        ppu_layers_frame("$212f - TSW", "Sub Screen Windows enabled:", &mut self.tsw, false)
    }

    fn cgwsel_frame(&mut self) -> Frame {
        let frame = Frame::new(Some("$2130 - CGWSEL"));
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.set_border_width(5);

        hbox.pack_start(&Label::new(Some("Clip colors:")), false, true, 0);
        hbox.pack_start(&self.cgw_clip, false, true, 0);
        hbox.pack_start(&Label::new(Some("Prevent math:")), false, true, 0);
        hbox.pack_start(&self.cgw_prevent, false, true, 0);

        hbox.pack_end(&self.cgw_direct_color, false, true, 0);
        hbox.pack_end(&self.cgw_subscreen, false, true, 0);

        frame.add(&hbox);
        frame
    }

    fn cgadsub_frame(&mut self) -> Frame {
        use gtk::Cast;

        let frame = ppu_layers_frame("$2131 - CGADSUB", "Color Math enabled:", &mut self.color_math, true);
        let hbox = frame.get_child().unwrap().downcast::<gtk::Box>().unwrap();

        hbox.pack_end(&self.math_add_sub, false, true, 0);
        hbox.pack_end(&self.half_math, false, true, 0);

        frame
    }
}

impl Tool for PpuRegs {
    fn new() -> Self {
        let mosaicsize = ComboBoxText::new();
        for i in 1..17 {
            mosaicsize.append_text(&format!("{0}x{0}", i));
        }

        let bgmode = ComboBoxText::new();
        for mode in 1..8 {
            bgmode.append_text(&format!("Mode {}", mode));
        }

        let mut bg_tilesizes = Vec::new();
        for _bg in 1..5 {
            bg_tilesizes.push(combo_box_text(&[
                "8x8",
                "16x16",
            ]));
        }

        PpuRegs {
            regs: ListStore::new(&[
                gtk::Type::String,  // Address (Hex `u16`)
                gtk::Type::String,  // Name
                gtk::Type::String,  // Raw value (Hex `u8`)
            ]),
            fblank: CheckButton::new_with_label("Forced Blank"),
            brightness: gtk::Scale::new_with_range(Orientation::Horizontal, 0.0, 15.0, 1.0),
            objsize: combo_box_text(&[
                "8x8 and 16x16 sprites",
                "8x8 and 32x32 sprites",
                "8x8 and 64x64 sprites",
                "16x16 and 32x32 sprites",
                "16x16 and 64x64 sprites",
                "32x32 and 64x64 sprites",
                "16x32 and 32x64 sprites",
                "16x32 and 32x32 sprites",
            ]),
            bgmode: bgmode,
            bg_tilesizes: bg_tilesizes,
            mosaicsize: mosaicsize,
            mosaicbgs: vec![
                CheckButton::new_with_label("BG1"),
                CheckButton::new_with_label("BG2"),
                CheckButton::new_with_label("BG3"),
                CheckButton::new_with_label("BG4"),
            ],
            tm: vec![],
            ts: vec![],
            tmw: vec![],
            tsw: vec![],
            cgw_clip: combo_box_text(&[
                "Never",
                "Outside Color Window",
                "Inside Color Window",
                "Always",
            ]),
            cgw_prevent: combo_box_text(&[
                "Never",
                "Outside Color Window",
                "Inside Color Window",
                "Always",
            ]),
            cgw_subscreen: CheckButton::new_with_label("Subscreen Math"),
            cgw_direct_color: CheckButton::new_with_label("Direct Color Mode"),
            color_math: vec![],
            math_add_sub: combo_box_text(&[
                "Add",
                "Subtract",
            ]),
            half_math: CheckButton::new_with_label("Half color math"),
        }
    }

    fn get_name(&self) -> &'static str { "PPU Regs" }

    fn init_tab(&mut self, win: &ScrolledWindow) {
        let left_column = gtk::Box::new(Orientation::Vertical, 5);
        left_column.set_border_width(5);

        left_column.pack_start(&self.inidisp_frame(), false, true, 0);
        left_column.pack_start(&self.obsel_frame(), false, true, 0);
        left_column.pack_start(&self.bgmode_frame(), false, true, 0);
        left_column.pack_start(&self.mosaic_frame(), false, true, 0);
        left_column.pack_start(&self.tm_frame(), false, true, 0);
        left_column.pack_start(&self.ts_frame(), false, true, 0);
        left_column.pack_start(&self.tmw_frame(), false, true, 0);
        left_column.pack_start(&self.tsw_frame(), false, true, 0);
        left_column.pack_start(&self.cgwsel_frame(), false, true, 0);
        left_column.pack_start(&self.cgadsub_frame(), false, true, 0);

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
        self.bgmode.set_sensitive(false);
        self.mosaicsize.set_sensitive(false);
        self.cgw_clip.set_sensitive(false);
        self.cgw_prevent.set_sensitive(false);
        self.cgw_subscreen.set_sensitive(false);
        self.cgw_direct_color.set_sensitive(false);
        self.half_math.set_sensitive(false);
        self.math_add_sub.set_sensitive(false);
        for x in &self.mosaicbgs { x.set_sensitive(false); }
        for x in &self.bg_tilesizes { x.set_sensitive(false); }
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
        self.bgmode.set_active((bgmode & 0b111) as i32);
        for bg in 1..5 {
            self.bg_tilesizes[bg - 1].set_active(if bgmode & 0x80 << bg == 0 { 0 } else { 1 });
        }

        let mosaic = data.ppu.mosaic();
        self.mosaicsize.set_active(((mosaic & 0xf0) >> 4) as i32);
        for i in 0..4 {
            self.mosaicbgs[i].set_active(mosaic & (1 << i) != 0);
        }

        let tm = data.ppu.tm();
        for i in 0..5 {
            self.tm[i].set_active(tm & (1 << i) != 0);
        }

        let ts = data.ppu.ts();
        for i in 0..5 {
            self.ts[i].set_active(ts & (1 << i) != 0);
        }

        let tmw = data.ppu.tmw();
        for i in 0..5 {
            self.tmw[i].set_active(tmw & (1 << i) != 0);
        }

        let tsw = data.ppu.tsw();
        for i in 0..5 {
            self.tsw[i].set_active(tsw & (1 << i) != 0);
        }

        let cgwsel = data.ppu.cgwsel();
        let cgw_clip = cgwsel >> 6;
        let cgw_prevent = (cgwsel >> 4) & 0b11;
        let cgw_subscreen = cgwsel & 0x02 != 0;
        let cgw_direct_color = cgwsel & 0x01 != 0;
        self.cgw_clip.set_active(cgw_clip as i32);
        self.cgw_prevent.set_active(cgw_prevent as i32);
        self.cgw_subscreen.set_active(cgw_subscreen);
        self.cgw_direct_color.set_active(cgw_direct_color);

        let cgadsub = data.ppu.cgadsub();
        let add_subtract = cgadsub >> 7;
        let half_math = (cgadsub >> 6) & 1 != 0;
        self.math_add_sub.set_active(add_subtract as i32);
        self.half_math.set_active(half_math);
        for i in 0..6 {
            self.color_math[i].set_active(cgadsub & (1 << i) != 0);
        }

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
