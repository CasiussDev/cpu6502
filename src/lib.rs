mod alu;
mod cpu;
mod instructions;
mod pinout;
mod registers;

#[macro_use]
extern crate enum_primitive_derive;
extern crate lazy_static;
extern crate num_traits;

pub use cpu::Cpu;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
