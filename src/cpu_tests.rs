
use std::io::{Write, Seek, SeekFrom};
use crate::cpu::*;

// Random mini-program
static HELLO_WORLD: [u8; 20] = [
    0x62, 0x78, 0xA5, 0x00, 0x63,
    0x01, 0x64, 0x01, 0xF1, 0x0A,
    0x00, 0xE0, 0xF2, 0x18, 0xF1,
    0x29, 0xD3, 0x45, 0x12, 0x00
];

fn load_hello_world(cpu: &mut CPU) {
    let mut tmp_file = tempfile::tempfile().unwrap();
    let _ = tmp_file.write(&HELLO_WORLD).unwrap();
    tmp_file.flush().unwrap();
    tmp_file.seek(SeekFrom::Start(0)).unwrap();
    cpu.load_game(&mut tmp_file);
}

#[test]
fn fontset_is_loaded_correctly() {
    let cpu = CPU::new();
    for i in 0..FONT_SET.len() {
        assert_eq!(cpu.ram[0x50+i], FONT_SET[i]);
    }
}

#[test]
fn game_is_loaded_correctly() {
    let mut cpu = CPU::new();
    load_hello_world(&mut cpu);
    for i in 0..HELLO_WORLD.len() {
        assert_eq!(cpu.ram[0x200+i], HELLO_WORLD[i]);
    }
    assert_eq!(cpu.pc, 0x200);
}

#[test]
fn opcode_is_read_correctly() {
    let mut cpu = CPU::new();
    load_hello_world(&mut cpu);
    assert_eq!(0x6278, cpu.get_opcode());
}

#[test]
fn op_00e0() {
    let mut cpu = CPU::new();
    cpu.vram[0][0] = 1;
    cpu.exec_opcode(0x00E0);
    assert_eq!(cpu.vram[0][0], 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_00ee() {
    let mut cpu = CPU::new();
    // Simulate CALL to 0x300 (0x2300)
    cpu.stack[cpu.sp] = cpu.pc;
    cpu.sp += 1;
    cpu.pc = 0x300;
    cpu.exec_opcode(0x00EE);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_1nnn() {
    let mut cpu = CPU::new();
    cpu.exec_opcode(0x1333);
    assert_eq!(cpu.pc, 0x333);
}

#[test]
fn op_2nnn() {
    let mut cpu = CPU::new();
    cpu.exec_opcode(0x2444);
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack[cpu.sp-1], 0x200);
    assert_eq!(cpu.pc, 0x444);
}

#[test]
fn op_3xkk() {
    let mut cpu = CPU::new();
    cpu.v[0x2] = 0x3;
    cpu.exec_opcode(0x3203);
    assert_eq!(cpu.pc, 0x204);
    cpu.exec_opcode(0x3204);
    assert_eq!(cpu.pc, 0x206);

}

#[test]
fn op_4xkk() {
    let mut cpu = CPU::new();
    cpu.v[0xA] = 0x5;
    cpu.exec_opcode(0x4A07);
    assert_eq!(cpu.pc, 0x204);
    cpu.exec_opcode(0x4A05);
    assert_eq!(cpu.pc, 0x206);

}

#[test]
fn op_5xy0() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 1;
    cpu.v[0x1] = 1;
    cpu.v[0x2] = 2;
    cpu.exec_opcode(0x5010);
    assert_eq!(cpu.pc, 0x204);
    cpu.exec_opcode(0x5020);
    assert_eq!(cpu.pc, 0x206);
}

#[test]
fn op_6xkk() {
    let mut cpu = CPU::new();
    cpu.exec_opcode(0x6A10);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0xA], 0x10);
}

#[test]
fn op_7xkk() {
    let mut cpu = CPU::new();
    cpu.v[0x5] = 0x9;
    cpu.exec_opcode(0x7501);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x5], 0xA);
}

#[test]
fn op_8xy0() {
    let mut cpu = CPU::new();
    cpu.v[0x5] = 0x9;
    cpu.v[0xA] = 0xFF;
    cpu.exec_opcode(0x85A0);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x5], 0xFF);
    assert_eq!(cpu.v[0xA], 0xFF);
}

#[test]
fn op_8xy1() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x1;
    cpu.v[0x1] = 0x2;
    cpu.exec_opcode(0x8011);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0x3);
    assert_eq!(cpu.v[0x1], 0x2);
}

#[test]
fn op_8xy2() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x1;
    cpu.v[0x1] = 0x2;
    cpu.exec_opcode(0x8012);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0x0);
    assert_eq!(cpu.v[0x1], 0x2);
}

#[test]
fn op_8xy3() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x1;
    cpu.v[0x1] = 0x2;
    cpu.exec_opcode(0x8013);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0x3);
    assert_eq!(cpu.v[0x1], 0x2);
}

#[test]
fn op_8xy4() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x10;
    cpu.v[0x1] = 0xFF;
    cpu.exec_opcode(0x8014);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], ((0x10 + 0xFF) as u16) as u8);
    assert_eq!(cpu.v[0x1], 0xFF);
    assert_eq!(cpu.v[0xF], 0x1);
    cpu.v[0x0] = 0x05;
    cpu.v[0x1] = 0xF0;
    cpu.v[0xF] = 0;
    cpu.exec_opcode(0x8014);
    assert_eq!(cpu.pc, 0x204);
    assert_eq!(cpu.v[0x0], 0xF5);
    assert_eq!(cpu.v[0x1], 0xF0);
    assert_eq!(cpu.v[0xF], 0x0);
}

#[test]
fn op_8xy5() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0xF;
    cpu.v[0x1] = 0x5;
    cpu.exec_opcode(0x8015);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0xA);
    assert_eq!(cpu.v[0xF], 0x1);
    cpu.v[0x0] = 0x5;
    let old_v0 = 0x5 as u8;
    cpu.v[0x1] = 0xF;
    cpu.v[0xF] = 0;
    cpu.exec_opcode(0x8015);
    assert_eq!(cpu.pc, 0x204);
    assert_eq!(cpu.v[0x0], old_v0.wrapping_sub(cpu.v[0x1]));
    assert_eq!(cpu.v[0xF], 0x0);
}

#[test]
fn op_8xy6() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0xA;
    cpu.exec_opcode(0x80F6);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0x5);
    assert_eq!(cpu.v[0xF], 0x0);
    cpu.v[0x0] = 0xFF;
    cpu.exec_opcode(0x80D6);
    assert_eq!(cpu.pc, 0x204);
    assert_eq!(cpu.v[0x0], 0x7F);
    assert_eq!(cpu.v[0xF], 0x1);
}

#[test]
fn op_8xy7() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x0;
    cpu.v[0x1] = 0xA;
    cpu.exec_opcode(0x8017);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0xA);
    assert_eq!(cpu.v[0xF], 0x1);
    cpu.v[0x0] = 0x1;
    cpu.v[0x1] = 0x0;
    cpu.v[0xF] = 0x0;
    cpu.exec_opcode(0x8017);
    assert_eq!(cpu.pc, 0x204);
    assert_eq!(cpu.v[0x0], 0xFF);
    assert_eq!(cpu.v[0xF], 0x0);
}

#[test]
fn op_8xye() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0xA;
    cpu.exec_opcode(0x80EE);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.v[0x0], 0x14);
    assert_eq!(cpu.v[0xF], 0x0);
    cpu.v[0x0] = 0xFF;
    cpu.v[0xF] = 0x0;
    cpu.exec_opcode(0x80FE);
    assert_eq!(cpu.pc, 0x204);
    assert_eq!(cpu.v[0x0], 0xFE);
    assert_eq!(cpu.v[0xF], 0x1);
}

#[test]
fn op_9xy0() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x0;
    cpu.v[0x1] = 0x1;
    cpu.exec_opcode(0x9010);
    assert_eq!(cpu.pc, 0x204);
    cpu.v[0x0] = 0x1;
    cpu.exec_opcode(0x9010);
    assert_eq!(cpu.pc, 0x206);
}

#[test]
fn op_annn() {
    let mut cpu = CPU::new();
    cpu.exec_opcode(0xA123);
    assert_eq!(cpu.i, 0x123);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_bnnn() {
    let mut cpu = CPU::new();
    cpu.v[0] = 5;
    cpu.exec_opcode(0xB123);
    assert_eq!(cpu.pc, 0x128);
}

#[test]
fn op_dxyn() {
    // http://www.emulator101.com/chip-8-sprites.html
    let mut cpu = CPU::new();
    // Load space invader sprite at 0x500
    let space_invader_sprite: [u8; 6] = [
        /* X.XXX.X. */ 0b10111010,
        /* .XXXXX.. */ 0b01111100,
        /* XX.X.XX. */ 0b11010110,
        /* XXXXXXX. */ 0b11111110,
        /* .X.X.X.. */ 0b01010100,
        /* X.X.X.X. */ 0b10101010
    ];
    for i in 0..space_invader_sprite.len() {
        cpu.ram[0x500+i] = space_invader_sprite[i];
    }

    assert_eq!(cpu.ram[0x500], 0xBA);
    assert_eq!(cpu.ram[0x501], 0x7C);
    assert_eq!(cpu.ram[0x502], 0xD6);
    assert_eq!(cpu.ram[0x503], 0xFE);
    assert_eq!(cpu.ram[0x504], 0x54);
    assert_eq!(cpu.ram[0x505], 0xAA);

    cpu.i = 0x500;
    cpu.v[0x0] = 0;
    cpu.v[0x1] = 0;

    cpu.exec_opcode(0xD016);

    assert_eq!(cpu.vram[0][0], 1);
    assert_eq!(cpu.vram[0][1], 0);
    assert_eq!(cpu.vram[0][2], 1);
    assert_eq!(cpu.vram[0][3], 1);
    assert_eq!(cpu.vram[0][4], 1);
    assert_eq!(cpu.vram[0][5], 0);
    assert_eq!(cpu.vram[0][6], 1);
    assert_eq!(cpu.vram[0][7], 0);

    assert_eq!(cpu.vram[1][0], 0);
    assert_eq!(cpu.vram[1][1], 1);
    assert_eq!(cpu.vram[1][2], 1);
    assert_eq!(cpu.vram[1][3], 1);
    assert_eq!(cpu.vram[1][4], 1);
    assert_eq!(cpu.vram[1][5], 1);
    assert_eq!(cpu.vram[1][6], 0);
    assert_eq!(cpu.vram[1][7], 0);

    assert_eq!(cpu.vram[2][0], 1);
    assert_eq!(cpu.vram[2][1], 1);
    assert_eq!(cpu.vram[2][2], 0);
    assert_eq!(cpu.vram[2][3], 1);
    assert_eq!(cpu.vram[2][4], 0);
    assert_eq!(cpu.vram[2][5], 1);
    assert_eq!(cpu.vram[2][6], 1);
    assert_eq!(cpu.vram[2][7], 0);

    assert_eq!(cpu.vram[3][0], 1);
    assert_eq!(cpu.vram[3][1], 1);
    assert_eq!(cpu.vram[3][2], 1);
    assert_eq!(cpu.vram[3][3], 1);
    assert_eq!(cpu.vram[3][4], 1);
    assert_eq!(cpu.vram[3][5], 1);
    assert_eq!(cpu.vram[3][6], 1);
    assert_eq!(cpu.vram[3][7], 0);

    assert_eq!(cpu.vram[4][0], 0);
    assert_eq!(cpu.vram[4][1], 1);
    assert_eq!(cpu.vram[4][2], 0);
    assert_eq!(cpu.vram[4][3], 1);
    assert_eq!(cpu.vram[4][4], 0);
    assert_eq!(cpu.vram[4][5], 1);
    assert_eq!(cpu.vram[4][6], 0);
    assert_eq!(cpu.vram[4][7], 0);

    assert_eq!(cpu.vram[5][0], 1);
    assert_eq!(cpu.vram[5][1], 0);
    assert_eq!(cpu.vram[5][2], 1);
    assert_eq!(cpu.vram[5][3], 0);
    assert_eq!(cpu.vram[5][4], 1);
    assert_eq!(cpu.vram[5][5], 0);
    assert_eq!(cpu.vram[5][6], 1);
    assert_eq!(cpu.vram[5][7], 0);

    assert_eq!(cpu.v[0xF], 0x0);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_ex9e() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x0;
    cpu.keypad[0x0] = true;
    cpu.exec_opcode(0xE09E);
    assert_eq!(cpu.pc, 0x204);
    cpu.keypad[0x0] = false;
    cpu.exec_opcode(0xE09E);
    assert_eq!(cpu.pc, 0x206);
}

#[test]
fn op_exa1() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x0;
    cpu.keypad[0x0] = false;
    cpu.exec_opcode(0xE0A1);
    assert_eq!(cpu.pc, 0x204);
    cpu.keypad[0x0] = true;
    cpu.exec_opcode(0xE0A1);
    assert_eq!(cpu.pc, 0x206);
}

#[test]
fn op_fx07() {
    let mut cpu = CPU::new();
    cpu.delay_timer = 0x5;
    cpu.exec_opcode(0xF007);
    assert_eq!(cpu.v[0x0], 0x5);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx0a() {
    let mut cpu = CPU::new();
    cpu.exec_opcode(0xF00A);
    assert_eq!(cpu.pc, 0x202);
    assert_eq!(cpu.waiting_keypad, true);
    assert_eq!(cpu.waiting_keypad_register, 0x0);
}

#[test]
fn op_fx15() {
    let mut cpu = CPU::new();
    cpu.v[0xA] = 0x5;
    cpu.exec_opcode(0xFA15);
    assert_eq!(cpu.delay_timer, 0x5);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx18() {
    let mut cpu = CPU::new();
    cpu.v[0xA] = 0x5;
    cpu.exec_opcode(0xFA18);
    assert_eq!(cpu.sound_timer, 0x5);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx1e() {
    let mut cpu = CPU::new();
    cpu.i = 0x1;
    cpu.v[0x0] = 0x5;
    cpu.exec_opcode(0xF01E);
    assert_eq!(cpu.i, 0x6);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx29() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0x2;
    cpu.exec_opcode(0xF029);
    assert_eq!(cpu.i, 0xA);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx33() {
    let mut cpu = CPU::new();
    cpu.v[0x0] = 0xFF;
    cpu.i = 0x300;
    cpu.exec_opcode(0xF033);
    assert_eq!(cpu.ram[cpu.i], 0x2);
    assert_eq!(cpu.ram[cpu.i+1], 0x5);
    assert_eq!(cpu.ram[cpu.i+2], 0x5);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx55() {
    let mut cpu = CPU::new();
    cpu.i = 0x300;
    cpu.v[0x0] = 0x1;
    cpu.v[0x1] = 0x2;
    cpu.v[0x2] = 0x4;
    cpu.exec_opcode(0xF255);
    assert_eq!(cpu.ram[cpu.i], 0x1);
    assert_eq!(cpu.ram[cpu.i+1], 0x2);
    assert_eq!(cpu.ram[cpu.i+2], 0x4);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn op_fx65() {
    let mut cpu = CPU::new();
    cpu.i = 0x300;
    cpu.ram[0x300] = 0x1;
    cpu.ram[0x301] = 0x2;
    cpu.ram[0x302] = 0x4;
    cpu.exec_opcode(0xF265);
    assert_eq!(cpu.v[0x0], 0x1);
    assert_eq!(cpu.v[0x1], 0x2);
    assert_eq!(cpu.v[0x2], 0x4);
    assert_eq!(cpu.pc, 0x202);
}

#[test]
fn timers() {
    let mut cpu = CPU::new();
    load_hello_world(&mut cpu);
    cpu.sound_timer = 10;
    cpu.delay_timer = 20;
    cpu.tick([false; 16]);
    assert_eq!(cpu.sound_timer, 9);
    assert_eq!(cpu.delay_timer, 19);
}
