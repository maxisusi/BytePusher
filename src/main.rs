use raylib::prelude::*;
use std::{fs, path::Path, time::Duration};

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

const MEMORY_SIZE: usize = 0x1000008;
const INSTRUCTION_STEP: usize = 65536;
const _COLOR_INTENSITY: usize = 0x33;

//
// [0] {red: 000, blue: 000, green: 000}
// [1] {red: 033, blue: 000, green: 000}
// [2] {red: 066, blue: 000, green: 000}
// [3] {red: 099, blue: 000, green: 000}

fn main() {
    let (mut rl, thread) = raylib::init().size(640, 480).title("Hello, World").build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
    let path = Path::new("/home/max/Documents/dev/rust/bpp/roms/Palette Test.BytePusher");
    let program = fs::read(path).expect("Couldn't read program");

    let mut cpu = Cpu::new(program.clone());

    loop {
        cpu.reset();
        let mut step = 0;
        while step < INSTRUCTION_STEP {
            cpu.step();
            step += 1;
        }

        for (idx, byte) in (cpu.memory[0..20]).iter().enumerate() {
            // if *byte != program[idx] {
            //     println!("{}:\t{} \t{}:\t{}", idx, byte, idx, program[idx]);
            // }
            println!("{}:\t{}", idx, byte);
        }
        println!("---------------------------");
        std::thread::sleep(Duration::new(0, 0));
    }
}

struct Cpu {
    memory: Vec<u8>,
    pc: *const u8,
}

impl Cpu {
    fn new(mem: Vec<u8>) -> Self {
        let mut memory = mem.clone();
        memory.resize(MEMORY_SIZE, 0);
        let pc = memory.as_ptr();

        Self { memory: mem, pc }
    }

    fn reset(&mut self) {
        let pc_addr = ((((self.memory[2]) as u32) << 16)
            | (((self.memory[3]) as u32) << 8)
            | self.memory[4] as u32) as usize;
        unsafe {
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

            if destination > self.memory.len() {
                // Print values
                println!(
                    "Source: {} Destination: {} Jump: {}",
                    source, destination, jump
                );
                // Print destination PC values
                //
                println!("PC+0: {}", *self.pc.add(0) as usize);
                println!("PC+1: {}", *self.pc.add(1) as usize);
                println!("PC+2: {}", *self.pc.add(2) as usize);
                println!("PC+3: {}", *self.pc.add(3) as usize);
                println!("PC+4: {}", *self.pc.add(4) as usize);
                println!("PC+5: {}", *self.pc.add(5) as usize);
                println!("\n");
                panic!("Destination out of bounds");
            }
            self.memory[destination] = self.memory[source];
            self.pc = self.memory.as_ptr().add(jump);
        }
    }
}

#[test]
fn cpu_step_test() {
    let mut program = vec![
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
