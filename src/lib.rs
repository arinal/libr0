//! # libr0
//!
//! Building Rust's standard library from scratch - a hands-on learning guide.
//!
//! This library provides educational reimplementations of Rust's core types
//! to help understand how they work under the hood.

pub mod r#box;
pub mod cell;
pub mod option;
pub mod rc;
pub mod refcell;
pub mod result;
pub mod vec;

// Re-export main types for convenience
pub use cell::Cell0;
pub use option::Option0;
pub use r#box::Box0;
pub use rc::{Rc0, Weak0};
pub use refcell::{BorrowError, BorrowMutError, Ref, RefCell0, RefMut};
pub use result::Result0;
pub use vec::{IntoIter, Vec0};
