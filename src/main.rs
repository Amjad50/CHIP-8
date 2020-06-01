mod system;
use std::env;
use std::fs::File;
use system::cpu::CPU;

fn main() {
    let args: Vec<String> = env::args().collect();

    // mostly redundant
    assert_eq!(args.len() > 0, true);

    if args.len() < 2 {
        println!("USAGE: {} <rom-file>", args[0]);
        return;
    }

    let mut cpu = CPU::new();
    cpu.read_file(&mut File::open(&args[1]).unwrap());

    cpu.run_display_application();
}
