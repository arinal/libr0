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
pub use option::MyOption;
pub use result::MyResult;
pub use r#box::MyBox;
pub use vec::{MyVec, MyVecIntoIter};
pub use cell::MyCell;
pub use refcell::{MyRefCell, Ref, RefMut, BorrowError, BorrowMutError};
pub use rc::{MyRc, MyWeak};