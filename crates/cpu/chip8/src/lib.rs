pub mod cpu;

#[cfg(test)]
#[path = "tests/cpu.rs"]
mod cpu_tests;

pub mod instruction;

#[cfg(test)]
#[path = "tests/instruction.rs"]
mod instruction_tests;
