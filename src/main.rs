mod bus;
mod cpu;
mod mmu;
mod ppu;
mod utils;

fn main() {
    println!("Hello, world!");
    let cpu = cpu::GameBoyCPU::new();
}
