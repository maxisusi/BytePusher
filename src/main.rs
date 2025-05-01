use raylib::prelude::*;
use std::{fs, ops::BitOr, path::Path, time::Duration};

// * The memory of the Bytepusher. 16 MiB (0x1000008 bytes).
//  * The memory map is located at the beginning of memory and
//  * looks like this:
//  *
//  * memory (byte) | description
//  * --------------|----------------
//  * 0, 1          | Keyboard state, if key X is
//  *               | pressed then bit X is on
//  *               |
//  * 2, 3, 4       | The program counter starts here
//  *               |
//  * 5             | Graphics block location. A value
//  *               | of ZZ means color of pixel at coordinate (XX, YY)
//  *               | is at ZZYYXX
//  *               |
//  * 6, 7          | Sound block location. A value of XXYY
//  *               | means audio sample ZZ is at address XXYY
//  * -------------------------------
//  * The byte ordering used by Bytepusher is big-endian.

const MEM_SIZE: usize = 0x1000008;
const MEM_PADDING: usize = 0x8;
const INSTR_STEP: usize = 65536;

const COLOR_INTENSITY: usize = 0x33;

fn main() {
    let path = Path::new("/home/max/Documents/dev/rust/bpp/roms/Keyboard Test.bytepusher");
    let program = fs::read(path).expect("Couldn't read program");

    let mut cpu = Cpu::new(program);

    let palette: &mut [Color; 256] = &mut [Default::default(); 256];
    let mut idx = 0;

    for r in (0..=255).step_by(COLOR_INTENSITY) {
        for g in (0..=255).step_by(COLOR_INTENSITY) {
            for b in (0..=255).step_by(COLOR_INTENSITY) {
                palette[idx] = Color { r, g, b, a: 255 };
                idx += 1;
            }
        }
    }

    let _ = palette.iter_mut().take(217).map(|_| Color::BLACK);

    let (mut rl, thread) = raylib::init().size(256, 256).title("BytePusher VM").build();
    while !rl.window_should_close() {
        cpu.reset();

        // While loop are more perfomant that for..in loops in Rust
        let mut step = 0;
        while step < INSTR_STEP {
            cpu.step();
            step += 1;
        }

        // Check if any key is pressed
        if let Some(key) = rl.get_key_pressed() {
            println!("{:?}", key);
            match key {
                KeyboardKey::KEY_ONE => cpu.step_key(1),
                KeyboardKey::KEY_TWO => cpu.step_key(2),
                KeyboardKey::KEY_THREE => cpu.step_key(3),
                KeyboardKey::KEY_FOUR => cpu.step_key(12),
                KeyboardKey::KEY_Q => cpu.step_key(4),
                KeyboardKey::KEY_W => cpu.step_key(5),
                KeyboardKey::KEY_E => cpu.step_key(6),
                KeyboardKey::KEY_R => cpu.step_key(13),
                KeyboardKey::KEY_A => cpu.step_key(7),
                KeyboardKey::KEY_S => cpu.step_key(8),
                KeyboardKey::KEY_D => cpu.step_key(9),
                KeyboardKey::KEY_F => cpu.step_key(14),
                KeyboardKey::KEY_Z => cpu.step_key(10),
                KeyboardKey::KEY_X => cpu.step_key(0),
                KeyboardKey::KEY_C => cpu.step_key(11),
                KeyboardKey::KEY_V => cpu.step_key(15),
                _ => {}
            }
        } else {
            // Unset all keys
            let mut keyb_ptr = cpu.memory.as_ptr() as *mut u16;
            let mut value = unsafe { u16::from_le(*keyb_ptr) };
            for i in 0..16 {
                if value & (1 << i) != 0 {
                    value &= !(1 << i);
                }
            }
            unsafe { *keyb_ptr = value }
        }

        let mut d = rl.begin_drawing(&thread);

        for y in 0..256 {
            for x in 0..256 {
                let mem_color_idx =
                    (((cpu.memory[5]) as usize) << 16) | ((y as usize) << 8) | x as usize;
                let pal_color_idx = cpu.memory[mem_color_idx] as usize;
                d.draw_pixel(x, y, palette[pal_color_idx]);
            }
        }
    }
}

struct Cpu {
    memory: Vec<u8>,
    pc: *const u8,
}

impl Cpu {
    fn new(rom: Vec<u8>) -> Self {
        let mut memory = rom.clone();
        memory.resize(MEM_SIZE + MEM_PADDING, 0);
        let pc = memory.as_ptr();
        Self { memory, pc }
    }

    fn reset(&mut self) {
        unsafe {
            // Unsafe as well, but... 3 FPS improvement
            let pc_addr = (((*(self.memory.get_unchecked(2)) as u32) << 16)
                | ((*(self.memory.get_unchecked(3)) as u32) << 8)
                | *self.memory.get_unchecked(4) as u32) as usize;
            self.pc = self.memory.as_ptr().add(pc_addr);
        }
    }

    fn step(&mut self) {
        unsafe {
            let source = ((*(self.pc) as usize) << 16)
                | ((*(self.pc.add(1)) as usize) << 8)
                | *(self.pc.add(2)) as usize;

            let destination = ((*(self.pc.add(3)) as usize) << 16)
                | ((*(self.pc.add(4)) as usize) << 8)
                | *(self.pc.add(5)) as usize;

            let jump = ((*(self.pc.add(6)) as usize) << 16)
                | ((*(self.pc.add(7)) as usize) << 8)
                | *(self.pc.add(8)) as usize;

            let mem_ptr = self.memory.as_mut_ptr();
            // -> Not safe but 11 FPS Improvement... (As long as the ROM is good, trust me bro)
            std::ptr::copy(mem_ptr.add(source), mem_ptr.add(destination), 1);
            self.pc = self.memory.as_ptr().add(jump);
        }
    }

    fn step_key(&mut self, byte: u8) {
        println!("Recieved key: {byte}");
        let keyb_ptr = self.memory.as_ptr() as *mut u16;
        let mut value = unsafe { u16::from_le(*keyb_ptr) };

        if byte < 8 {
            value |= 1 << (byte + 8);
        } else {
            value |= 1 << (byte - 8);
        }
        unsafe { *keyb_ptr = value }
        println!("Keyb state: {value}");
    }
}

#[test]
fn cpu_step_test() {
    let program = vec![
        0, // 0
        0, // 1
        0, // 2
        0, // 3
        8, // 4 -> PC starts here
        0, // 5
        0, // 6
        0, // 7
        //
        // SOURCE
        //
        0,  // 8
        0,  // 9
        20, // 10
        //
        // DESTINATION
        //
        0,  // 11
        0,  // 12
        25, // 13
        //
        // JUMP
        //
        0,  // 14
        0,  // 15
        17, // 16
        //
        //
        17, // 17
        0,  // 18
        0,  // 19
        1,  // 20
        0,  // 21
        0,  // 22
        0,  // 23
        0,  // 24
        0,  // 25 -> This will become 1
        0,  // 26
    ];
    // Put value of 99 to index 21

    let mut cpu = Cpu::new(program);
    cpu.reset();
    cpu.step();

    unsafe { assert_eq!(cpu.pc, cpu.memory.as_ptr().add(17)) }
    assert_eq!(cpu.memory[25], 1)
}
