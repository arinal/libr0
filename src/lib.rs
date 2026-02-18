//! # libr0
//!
//! Building Rust's standard library from scratch - a hands-on learning guide.
//!
//! This library provides educational reimplementations of Rust's core types
//! to help understand how they work under the hood.

pub mod option;
pub mod result;
pub mod r#box;
pub mod vec;
pub mod cell;
pub mod refcell;
pub mod rc;

// Re-export main types for convenience
pub use option::Option0;
pub use result::Result0;
pub use r#box::Box0;
pub use vec::{Vec0, IntoIter};
pub use cell::Cell0;
pub use refcell::{RefCell0, Ref, RefMut, BorrowError, BorrowMutError};
pub use rc::{Rc0, Weak0};