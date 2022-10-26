mod apu;
mod bus;
mod cart;
mod cpu;
mod debug;
mod ppu;

fn main() {
    let mut apu = apu::Apu::new();
    let mut ppu = ppu::Ppu::new();
    let mut bus = bus::Bus::new(&mut apu, &mut ppu);
    let mut cpu = cpu::Cpu::new(&mut bus);

    debug::disassemble(&mut bus::BOOT_ROM[..]);

    loop {
        cpu.tick()
    }
}
