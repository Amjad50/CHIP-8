mod system;
use system::cpu::CPU;

fn main() {
    let mut cpu = CPU::new();
    // CLS
    cpu.run_instruction(0x00E0);

    // LD VB, 0x05
    cpu.run_instruction(0x6B05);
    // LD VC, 0x05
    cpu.run_instruction(0x6C05);

    // LD VA, 0x0A
    cpu.run_instruction(0x6A0A);
    cpu.run_instruction(0xFA29);
    // ADD VB, 6
    cpu.run_instruction(0x7B06);
    // DRW VB, VC
    cpu.run_instruction(0xDBC5);

    // LD VA, 0x0B
    cpu.run_instruction(0x6A0B);
    cpu.run_instruction(0xFA29);
    // ADD VB, 6
    cpu.run_instruction(0x7B06);
    // DRW VB, VC
    cpu.run_instruction(0xDBC5);

    // LD VA, 0x0C
    cpu.run_instruction(0x6A0C);
    cpu.run_instruction(0xFA29);
    // ADD VB, 6
    cpu.run_instruction(0x7B06);
    // DRW VB, VC
    cpu.run_instruction(0xDBC5);

    // must run at the end
    cpu.run_display_application();
}
