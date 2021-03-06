
// GL
#[cfg(not(target_arch = "wasm32"))]
pub mod ogl;

#[cfg(not(target_arch = "wasm32"))]
pub use ogl::*;

// WEBGL
#[cfg(target_arch = "wasm32")]
pub mod wgl2;

#[cfg(target_arch = "wasm32")]
pub use wgl2::*;