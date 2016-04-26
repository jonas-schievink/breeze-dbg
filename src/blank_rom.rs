//! Provides a default ROM for when none is loaded

use breeze_core::rom::Rom;

use std::iter;

/// Loads the default ROM
pub fn load_blank_rom() -> Rom {
    let code = [
        // Enter endless loop
        0xA9, 0x00,         // lda #0
        0xF0, 0xFE,         // beq -2 (self)
    ];

    // Build the header
    let mut header = Vec::with_capacity(32);

    // First 21 Bytes: Title (ASCII)
    let name = b"BLANK";
    header.extend(name.into_iter()
                      .chain(iter::repeat(&b' '))
                      .take(21));

    header.push(0);     // ROM makeup Byte - LoROM, no FastROM
    header.push(0);     // Chipset (none/don't care)
    header.push(6);     // ROM size - $400<<6 = 64K bytes
    header.push(0);     // Cart. RAM size - $400 bytes
    header.push(0);     // Vendor code
    header.push(0);
    header.push(0);     // Version
    header.push(0x55);  // Checksum (invalid)
    header.push(0x55);
    header.push(0xAA);  // Checksum complement
    header.push(0xAA);
    // Extended header (ignored)

    assert_eq!(header.len(), 32);
    assert!(code.len() < 0x8000 - 64, "code size too high");

    // Now we can put the image together
    // The header is located (for LoROM) at `0x8000 - 64`, preceded by code that will be mapped to
    // 0x8000+, followed by the extended header, the interrupt vectors, and the data section(s)
    // (in our case)
    let mut rom = code.iter()
                      .cloned()
                      .chain(iter::repeat(0))
                      .take(0x8000 - 64)
                      .chain(header.into_iter())
                      .chain(iter::repeat(0))
                      .take(0x8000 * 2)
                      .collect::<Vec<_>>();

    // Set the correct vectors (emulation mode)
    // RESET @ 0x8000
    rom[0x7ffc] = 0x00;
    rom[0x7ffd] = 0x80;
    // This should now be a valid, runnable 64K ROM image (minus the checksum)

    Rom::from_bytes(&rom).unwrap()
}
