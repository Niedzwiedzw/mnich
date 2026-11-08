pub mod global_context;
pub mod surface;

#[macro_export]
macro_rules! line_label {
    () => {
        Some(concat!(env!("CARGO_PKG_NAME"), " ", file!(), ":", line!()))
    };
}
