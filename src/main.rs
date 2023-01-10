mod constants;

use std::{thread, time, env, fs::File, io::Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    assert_eq!(2, args.len(), "Ensuring exactly 1 argument is provided, the name of the file to run");


    // Initialize all emulator elements
    let mut mem : [u8; 4096]= [0; 4096]; // memory array, default initialize to all zeroes
    let mut display_mem: [[bool; 64]; 32] = [[false; 64]; 32]; // display memory, basically an array of pixels
    let mut program_counter: usize = 0x200; // code is initialized at 0x200 by convention
    let mut index_reg: u16 = 0;
    // let mut return_stack: Vec<u16> = Vec::new(); // emulated return stack of infinite size
    // let mut delay_timer: u8 = 0;
    // let mut sound_timer: u8 = 0;
    // let mut keypad: [bool; 16] = [false; 16];
    let mut reg_file: [u8; 16] = [0; 16]; // register file

    // load in code
    let mut file = File::open(&args[1]).unwrap();
    file.read(&mut mem[0x200..]).unwrap(); // read in code starting at 0x200

    println!("Read contents of file {}", &args[1]);

    let cycle_delay = time::Duration::from_millis(10); // TODO: add a command line arg for this

    // load in fonts
    // mem[0x050..0x0A0].clone_from_slice(&constants::FONT_TABLE); // TODO: Make this also a command-line option, or initialize code at a later address


    // Main loop of fetch, decode, execute

    loop {
        // fetch
        let insnbits: u16 = ((mem[program_counter] as u16) << 8) | (mem[program_counter + 1] as u16); // fetch instr
        program_counter += 2; // increment PC


        // decode
        let nibble_0: u8 = ((insnbits & 0xF000) >> 12) as u8;
        let nibble_1: u8 = ((insnbits & 0x0F00) >> 8) as u8;
        let nibble_2: u8 = ((insnbits & 0x00F0) >> 4) as u8;
        let nibble_3: u8 = (insnbits & 0x000F) as u8;
        let insn_nibbles = (nibble_0, nibble_1, nibble_2, nibble_3);

        // execute
        match insn_nibbles {
            (0x0, 0x0, 0xE, 0x0) => { // clear screen instruction
                display_mem.iter_mut().for_each(|row| row.iter_mut().for_each(|elem| *elem = false));
                display(&display_mem);
            },
            (0x1, n1, n2, n3) => { // jump
                let branch_target: u16 = ((n1 as u16) << 8) | (n2 << 4) as u16 | n3 as u16;
                program_counter = branch_target as usize;
            },
            (0x6, x, n1, n2) => { // set register vx
                let imm_val: u8 = (n1 << 4) | n2;
                reg_file[x as usize] = imm_val;
            },
            (0xA, n1, n2, n3) => { // set register I
                let imm_val: u16 = ((n1 as u16) << 8) | (n2 << 4) as u16 | n3 as u16;
                index_reg = imm_val;
            },
            (0x7, x, n1, n2) => { // Add value to register vx
                let imm_val: u8 = (n1 << 4) | n2;
                reg_file[x as usize] += imm_val;
            }
            (0xD, x, y, n) => { // Draw instruction
                let horizontal_start = (reg_file[x as usize] % 64) as usize;
                let vertical_start = (reg_file[y as usize] % 32) as usize;

                reg_file[0xF] = 0;
                let mut y_offset: usize = 0;

                while y_offset + vertical_start < 32 && y_offset < n.into() {
                    let row: u8 = mem[index_reg as usize + y_offset]; // read a byte
                    let pixels = u8_to_bool_array(row);
                    let mut x_offset = 0;
                    while x_offset + horizontal_start < 64 && x_offset < 8 {
                        display_mem[y_offset + vertical_start][x_offset + horizontal_start] ^= pixels[x_offset];
                        if !display_mem[y_offset + vertical_start][x_offset + horizontal_start] { // set flag if a pixel was cleared
                            reg_file[0xF] = 1;
                        }
                        x_offset += 1;
                    }
                    y_offset += 1;
                }
                display(&display_mem);
            }

            _ => panic!("Error: undefined instruction {:#04X} at address 0x{:#04X}", insnbits, program_counter),
        }



        // delay till next cycle
        thread::sleep(cycle_delay);
    }



}

fn display(display_mem: &[[bool; 64]; 32]) {
    // TODO: This is garbage, make it better later
    println!("==================================================================");
    for row in display_mem {
        print!("‖");
        for elem in row {
            print!("{}", if *elem {'@'} else {' '});
        }
        println!("‖");
    }
    println!("==================================================================");
}

// Takes in a u8 and splits it into an array of bits (boolean).
// For example, an input of 0x93 gives an output of:
// [true, false, false, true, false, false, true, true]
// Corresponding to the binary of 10010011
fn u8_to_bool_array(byte: u8) -> [bool; 8] {
    let mut ret = [false; 8];
    for i in 0..8 {
        ret[i] = ((byte << i) >> 7) == 1;
    }
    return ret;
}
