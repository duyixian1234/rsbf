use std::io::{Read, Write};

const MEMORY_SIZE: usize = 30000;

#[derive(Debug)]
enum Instruction {
    Increment,
    Decrement,
    MoveRight,
    MoveLeft,
    Read,
    Write,
    LoopStart(usize),
    LoopEnd(usize),
}

struct VirtualMachine<R: Read, W: Write> {
    memory: [u8; MEMORY_SIZE],
    pointer: usize,
    instructions: Vec<Instruction>,
    input: R,
    output: W,
}

impl<R: Read, W: Write> VirtualMachine<R, W> {
    fn new(input: R, output: W) -> VirtualMachine<R, W> {
        VirtualMachine {
            memory: [0; MEMORY_SIZE],
            pointer: 0,
            instructions: Vec::new(),
            input,
            output,
        }
    }

    fn reset(&mut self) {
        self.memory = [0; MEMORY_SIZE];
        self.pointer = 0;
    }

    fn clear(&mut self) {
        self.reset();
        self.instructions.clear();
    }

    fn compile(&mut self, code: &str) {
        self.clear();
        let mut left: Vec<usize> = Vec::new();
        for (_, ch) in code.chars().enumerate() {
            match ch {
                '>' => self.instructions.push(Instruction::MoveRight),
                '<' => self.instructions.push(Instruction::MoveLeft),
                '+' => self.instructions.push(Instruction::Increment),
                '-' => self.instructions.push(Instruction::Decrement),
                '.' => self.instructions.push(Instruction::Write),
                ',' => self.instructions.push(Instruction::Read),
                '[' => {
                    left.push(self.instructions.len());
                    self.instructions.push(Instruction::LoopStart(0));
                }
                ']' => {
                    let l = left.pop().unwrap();
                    self.instructions[l] = Instruction::LoopStart(self.instructions.len());
                    self.instructions.push(Instruction::LoopEnd(l));
                }
                _ => {}
            }
        }
    }

    fn run(&mut self) {
        let mut index = 0;
        let size = self.instructions.len();
        while index < size {
            let mut next = index + 1;
            match self.instructions[index] {
                Instruction::Increment => {
                    self.memory[self.pointer] = self.memory[self.pointer].wrapping_add(1);
                }
                Instruction::Decrement => {
                    self.memory[self.pointer] = self.memory[self.pointer].wrapping_sub(1);
                }
                Instruction::MoveRight => {
                    self.pointer = (self.pointer + 1) % MEMORY_SIZE;
                }
                Instruction::MoveLeft => {
                    self.pointer = (self.pointer + MEMORY_SIZE - 1) % MEMORY_SIZE;
                }
                Instruction::Read => {
                    let mut buf = [0; 1];
                    self.input.read_exact(&mut buf).unwrap();
                    self.memory[self.pointer] = buf[0];
                }
                Instruction::Write => {
                    let buf = [self.memory[self.pointer]];
                    self.output.write_all(&buf).unwrap();
                }
                Instruction::LoopStart(jump_to) => {
                    if self.memory[self.pointer] == 0 {
                        next = jump_to;
                    }
                }
                Instruction::LoopEnd(jump_to) => {
                    if self.memory[self.pointer] != 0 {
                        next = jump_to;
                    }
                }
            }
            index = next;
        }
    }
}

pub fn execute<R: Read, W: Write>(code: &str, input: R, output: W) {
    let mut vm = VirtualMachine::new(input, output);
    vm.compile(code);
    vm.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn test_output() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute(".", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![0u8]);
    }

    #[test]
    fn test_input() {
        let mut input = Cursor::new("A".as_bytes().to_vec());
        let mut output = Cursor::new(vec![0u8; 1]);
        execute(",.", &mut input, &mut output);
        assert_eq!(input.get_ref(), &"A".as_bytes().to_vec());
    }

    #[test]
    fn test_move_right() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute(">.", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![0u8]);
    }

    #[test]
    fn test_move_left() {
        let mut buffer = Cursor::new(vec![0u8; 2]);
        execute("+><.", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![1u8, 0u8]);
    }

    #[test]
    fn test_increment() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute("+.", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![1u8]);
    }

    #[test]
    fn test_decrement() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute("+-.", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![0u8]);
    }

    #[test]
    fn test_loop() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute("++[>+<-]>.", &mut io::empty(), &mut buffer);
        assert_eq!(buffer.get_ref(), &vec![2u8]);
    }

    #[test]
    fn test_full() {
        let mut buffer = Cursor::new(vec![0u8; 1]);
        execute(
            "++++++ [ > ++++++++++ < - ] > +++++ .",
            &mut io::empty(),
            &mut buffer,
        );
        assert_eq!(buffer.get_ref(), &b"A"[..]);
    }

    #[test]
    fn test_add() {
        let mut input = Cursor::new(vec![30u8, 35u8]);
        let mut output = Cursor::new(vec![0u8; 1]);
        let code = ",>,<[- >+ <]>.";
        execute(code, &mut input, &mut output);
        assert_eq!(output.get_ref(), &b"A"[..]);
    }
}
