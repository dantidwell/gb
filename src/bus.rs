use crate::apu;
use crate::ppu;

pub struct Bus<'a> {
    apu: &'a mut apu::Apu,
    ppu: &'a mut ppu::Ppu,
}

pub const BOOT_ROM: [u8; 48] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
    0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
    0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
];

impl<'a> Bus<'a> {
    pub fn new(apu: &'a mut apu::Apu, ppu: &'a mut ppu::Ppu) -> Self {
        Self { apu, ppu }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // TODO: allow disabling boot rom
            0x0000..=0x00FF => self.read_boot_rom(address - 0x0000),
            0x8000..=0x9FFF => self.ppu.read(address - 0x8000),
            _ => panic!("bus: invalid read access"),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => self.ppu.write(address, value),
            0xFF10..=0xFF3F => self.apu.write(address, value),
            0xFF40..=0xFF4B => self.ppu.write(address, value),
            _ => panic!("bus: invalid write access"),
        }
    }

    fn read_boot_rom(&self, address: u16) -> u8 {
        BOOT_ROM[address as usize]
    }
}
