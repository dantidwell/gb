pub struct Apu {
    ch1_len_and_duty: u8,
    ch1_vol_envelope: u8,
    enabled: u8,
    pan: u8,
    volume: u8,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            ch1_len_and_duty: 0,
            ch1_vol_envelope: 0,
            enabled: 0,
            pan: 0,
            volume: 0,
        }
    }

    // pub fn read(&self, address: u16) -> u8 {
    //     return 0;
    // }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF11 => self.ch1_len_and_duty = value,
            0xFF12 => self.ch1_vol_envelope = value,
            0xFF24 => self.volume = value,
            0xFF25 => self.pan = value,
            0xFF26 => self.enabled = value & 0x80,
            _ => panic!("apu: invalid write address {:#x}", address),
        }
    }
}
