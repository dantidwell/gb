pub struct Ppu {
    control: u8,
    ram: [u8; 0x2000],
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            control: 0,
            ram: [0; 0x2000],
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.ram[address as usize];
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.control = value,
            0x8000..=0x9FFF => self.ram[(address - 0x8000) as usize] = value,
            _ => panic!("ppu: invalid write access {:#x}", address),
        }
    }
}
