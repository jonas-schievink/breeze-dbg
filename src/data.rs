//! Sanitized data extracted from the emulator, ready for display in the view
//!
//! Note that the values are collected after the fact, which means they can technically sometimes
//! change mid-frame and become inconsistent.

use breeze_core::ppu::Ppu;
use breeze_core::ppu::oam::OamEntry;
use breeze_core::ppu::cgram::Cgram;

/// Created from `OamEntry`s and PPU state
pub struct Sprite {
    /// Start address of tile data in VRAM. Calculated from `tile` and `name_table` fields of
    /// `OamEntry` and PPU registers.
    pub tile_addr: u16,
    pub x: i16,
    pub y: u8,
    /// 0-3
    pub priority: u8,
    /// First CGRAM color index usable by this sprite. `128+ppp*16`.
    /// (or not due to transparency)
    pub color_start: u8,
    pub hflip: bool,
    pub vflip: bool,
    /// The result of `obj_size()` for this sprite, given the PPU state
    pub size: (u8, u8),
}

impl Sprite {
    pub fn new(ppu: &Ppu, sprite: &OamEntry) -> Self {
        // FIXME Share this calculation with Breeze
        let name_base: u16 = (ppu.obsel() as u16 & 0b111) << 13;
        let name_select: u16 = (ppu.obsel() as u16 >> 3) & 0b11;
        let tile_start_word_addr =
            (name_base |
            ((sprite.tile as u16) << 4) |
            (sprite.name_table as u16 * ((name_select + 1) << 12))) & 0x7fff;

        Sprite {
            tile_addr: tile_start_word_addr << 1,
            x: sprite.x,
            y: sprite.y,
            priority: sprite.priority,
            color_start: 128 + sprite.palette * 16,
            hflip: sprite.hflip,
            vflip: sprite.vflip,
            size: ppu.obj_size(sprite.size_toggle),
        }
    }
}

/// Data reported from the model to the view when the model is updated
pub struct ModelData<'a> {
    /// 128 sprites (OAM entries)
    pub sprites: &'a [Sprite],
    pub cgram: &'a Cgram,
    pub frame: &'a [u8],
}
