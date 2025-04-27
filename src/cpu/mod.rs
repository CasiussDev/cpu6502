#[cfg(not(feature = "new_instr_impl"))]
mod cpu_impl;

#[cfg(feature = "new_instr_impl")]
mod cpu_impl2;

#[cfg(feature = "logging")]
mod logging_memory;

#[cfg(not(feature = "new_instr_impl"))]
pub use cpu_impl::*;

#[cfg(feature = "new_instr_impl")]
pub use cpu_impl2::*;
