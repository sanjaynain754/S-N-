//! S+N++ type, ownership and definite-assignment checking facade.

pub fn check(functions: &[crate::Function]) -> Result<(), String> {
    crate::check_program(functions)
}
