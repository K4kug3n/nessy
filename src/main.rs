use nessy::nes::Nes;
use nessy::cartridge::Cartridge;

use std::io::prelude::*;
use std::fs::File;

fn main() {
    let mut file = File::open("rom/snake.nes").expect("Could not read the file {}");
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("Could not read bytes");

    let cartridge = Cartridge::from_ines(&buffer);

    let mut nes = Nes::new(&cartridge);

    nes.run();
}
