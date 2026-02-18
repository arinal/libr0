//! Result0 - Educational reimplementation of Result<T, E>

//! ```
//! use rustlib::result::Result0;
//! ```

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Result0<T, E> {
    Ok(T),
    Err(E),
}

pub use Result0::{Err, Ok};

impl<T, E> Result0<T, E> {
    /// Returns `true` if the result is an [`Ok`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert!(Ok::<i32, &str>(42).is_ok());
    /// assert!(!Err::<i32, &str>("error").is_ok());
    /// ```
    pub fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }

    /// Returns `true` if the result is an [`Err`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert!(!Ok::<i32, &str>(42).is_err());
    /// assert!(Err::<i32, &str>("error").is_err());
    /// ```
    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /// Converts from [`Result0<T, E>`] to `Option<T>`.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).ok(), Some(42));
    /// assert_eq!(Err::<i32, &str>("error").ok(), None);
    /// ```
    pub fn ok(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    /// Converts from [`Result0<T, E>`] to `Option<E>`.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).err(), None);
    /// assert_eq!(Err::<i32, &str>("error").err(), Some("error"));
    /// ```
    pub fn err(self) -> Option<E> {
        match self {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    /// Returns the contained value or a default.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).unwrap_or(0), 42);
    /// assert_eq!(Err::<i32, &str>("error").unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(val) => val,
            Err(_) => default,
        }
    }

    /// Returns the contained value or computes it from a closure.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).unwrap_or_else(|_| 0), 42);
    /// assert_eq!(Err::<i32, &str>("error").unwrap_or_else(|_| 100), 100);
    /// ```
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T {
        match self {
            Ok(val) => val,
            Err(e) => f(e),
        }
    }

    /// Maps a [`Result0<T, E>`] to [`Result0<U, E>`] by applying a function to the [`Ok`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(5).map(|x| x * 2), Ok(10));
    /// assert_eq!(Err::<i32, &str>("error").map(|x: i32| x * 2), Err("error"));
    /// ```
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result0<U, E> {
        match self {
            Ok(x) => Result0::Ok(f(x)),
            Err(e) => Result0::Err(e),
        }
    }

    /// Maps a [`Result0<T, E>`] to [`Result0<T, F>`] by applying a function to the [`Err`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).map_err(|e: &str| e.len()), Ok(42));
    /// assert_eq!(Err::<i32, &str>("error").map_err(|e| e.len()), Err(5));
    /// ```
    pub fn map_err<F2, O: FnOnce(E) -> F2>(self, op: O) -> Result0<T, F2> {
        match self {
            Ok(x) => Result0::Ok(x),
            Err(e) => Result0::Err(op(e)),
        }
    }

    /// Applies a function that returns a [`Result0`] to the [`Ok`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(2).and_then(|x| Ok(x * x)), Ok(4));
    /// assert_eq!(Err::<i32, &str>("error").and_then(|x: i32| Ok(x * x)), Err("error"));
    /// ```
    pub fn and_then<U, F: FnOnce(T) -> Result0<U, E>>(self, f: F) -> Result0<U, E> {
        match self {
            Ok(x) => f(x),
            Err(e) => Result0::Err(e),
        }
    }

    /// Converts from `&Result0<T, E>` to `Result0<&T, &E>`.
    /// ```
    /// use rustlib::result::{Result0, Ok};
    /// let x: Result0<String, &str> = Ok(String::from("hello"));
    /// assert_eq!(x.as_ref().map(|s| s.len()), Ok(5));
    /// ```
    pub fn as_ref(&self) -> Result0<&T, &E> {
        match self {
            Ok(x) => Result0::Ok(x),
            Err(e) => Result0::Err(e),
        }
    }

    /// Returns the result if [`Ok`], otherwise returns `other`.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(1).or(Ok(2)), Ok(1));
    /// assert_eq!(Err::<i32, &str>("error").or(Ok(2)), Ok(2));
    /// ```
    pub fn or(self, other: Result0<T, E>) -> Result0<T, E> {
        match self {
            Ok(x) => Ok(x),
            Err(_) => other,
        }
    }

    /// Returns the result if [`Ok`], otherwise calls `f`.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(1).or_else(|_| Ok(2)), Ok(1));
    /// assert_eq!(Err::<i32, &str>("error").or_else(|_| Ok(2)), Ok(2));
    /// ```
    pub fn or_else<F: FnOnce(E) -> Result0<T, E>>(self, f: F) -> Result0<T, E> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => f(e),
        }
    }

    /// Returns `other` if the result is [`Ok`], otherwise returns the [`Err`] value.
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(1).and(Ok("two")), Ok("two"));
    /// assert_eq!(Err::<i32, &str>("error").and(Ok("two")), Err("error"));
    /// ```
    pub fn and<U>(self, other: Result0<U, E>) -> Result0<U, E> {
        match self {
            Ok(_) => other,
            Err(e) => Err(e),
        }
    }
}

impl<T, E> Result0<Result0<T, E>, E> {
    /// Converts from [`Result0<Result0<T, E>, E>`] to [`Result0<T, E>`].
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<Result0<i32, &str>, &str>(Ok(42)).flatten(), Ok(42));
    /// assert_eq!(Ok::<Result0<i32, &str>, &str>(Err("error")).flatten(), Err("error"));
    /// assert_eq!(Err::<Result0<i32, &str>, &str>("error").flatten(), Err("error"));
    /// ```
    pub fn flatten(self) -> Result0<T, E> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}

impl<T, E: fmt::Debug> Result0<T, E> {
    /// Returns the contained [`Ok`] value, panicking if [`Err`].
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).unwrap(), 42);
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("called unwrap on Err: {:?}", e),
        }
    }

    /// Returns the contained [`Ok`] value, panicking with a custom message if [`Err`].
    /// ```
    /// use rustlib::result::{Result0, Ok, Err};
    /// assert_eq!(Ok::<i32, &str>(42).expect("should be ok"), 42);
    /// ```
    pub fn expect(self, msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("{}: {:?}", msg, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ok_is_err() {
        let ok: Result0<i32, &str> = Ok(42);
        assert!(ok.is_ok());
        assert!(!ok.is_err());

        let err: Result0<i32, &str> = Err("error");
        assert!(!err.is_ok());
        assert!(err.is_err());
    }

    #[test]
    fn test_ok() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.ok(), Some(42));

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.ok(), None);
    }

    #[test]
    fn test_err() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.err(), None);

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.err(), Some("error"));
    }

    #[test]
    fn test_unwrap() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "called unwrap on Err")]
    fn test_unwrap_err_panics() {
        let err: Result0<i32, &str> = Err("error");
        err.unwrap();
    }

    #[test]
    fn test_expect() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.expect("should be ok"), 42);
    }

    #[test]
    #[should_panic(expected = "custom message")]
    fn test_expect_err_panics() {
        let err: Result0<i32, &str> = Err("error");
        err.expect("custom message");
    }

    #[test]
    fn test_unwrap_or() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.unwrap_or(0), 42);

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.unwrap_or(0), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.unwrap_or_else(|_| 0), 42);

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.unwrap_or_else(|e| e.len() as i32), 5);
    }

    #[test]
    fn test_map() {
        let ok: Result0<i32, &str> = Ok(10);
        assert_eq!(ok.map(|x| x * 2), Ok(20));

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.map(|x| x * 2), Err("error"));
    }

    #[test]
    fn test_map_err() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(ok.map_err(|e| e.len()), Ok(42));

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.map_err(|e| e.len()), Err(5));
    }

    #[test]
    fn test_and_then() {
        let ok: Result0<i32, &str> = Ok(10);
        let result = ok.and_then(|x| Ok(x * 2));
        assert_eq!(result, Ok(20));

        let ok2: Result0<i32, &str> = Ok(10);
        let result2: Result0<i32, &str> = ok2.and_then(|_| Err("error"));
        assert_eq!(result2, Err("error"));

        let err: Result0<i32, &str> = Err("error");
        let result3 = err.and_then(|x| Ok(x * 2));
        assert_eq!(result3, Err("error"));
    }

    #[test]
    fn test_as_ref() {
        let ok: Result0<String, &str> = Ok(String::from("hello"));
        let as_ref = ok.as_ref();
        assert_eq!(as_ref, Result0::Ok(&String::from("hello")));
        // ok still valid
        assert_eq!(ok, Ok(String::from("hello")));
    }

    #[test]
    fn test_or() {
        let ok1: Result0<i32, &str> = Ok(1);
        let ok2: Result0<i32, &str> = Ok(2);
        assert_eq!(ok1.or(ok2), Ok(1));

        let err1: Result0<i32, &str> = Err("error1");
        let ok3: Result0<i32, &str> = Ok(3);
        assert_eq!(err1.or(ok3), Ok(3));

        let err2: Result0<i32, &str> = Err("error1");
        let err3: Result0<i32, &str> = Err("error2");
        assert_eq!(err2.or(err3), Err("error2"));
    }

    #[test]
    fn test_or_else() {
        let ok: Result0<i32, &str> = Ok(1);
        assert_eq!(ok.or_else(|_| Ok(2)), Ok(1));

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(err.or_else(|_| Ok(100)), Ok(100));
    }

    #[test]
    fn test_and() {
        let ok1: Result0<i32, &str> = Ok(1);
        let ok2: Result0<&str, &str> = Ok("two");
        assert_eq!(ok1.and(ok2), Ok("two"));

        let ok3: Result0<i32, &str> = Ok(1);
        let err: Result0<&str, &str> = Err("error");
        assert_eq!(ok3.and(err), Err("error"));

        let err2: Result0<i32, &str> = Err("error1");
        let ok4: Result0<&str, &str> = Ok("two");
        assert_eq!(err2.and(ok4), Err("error1"));
    }

    #[test]
    fn test_flatten() {
        let ok_ok: Result0<Result0<i32, &str>, &str> = Ok(Ok(42));
        assert_eq!(ok_ok.flatten(), Ok(42));

        let ok_err: Result0<Result0<i32, &str>, &str> = Ok(Err("inner error"));
        assert_eq!(ok_err.flatten(), Err("inner error"));

        let err: Result0<Result0<i32, &str>, &str> = Err("outer error");
        assert_eq!(err.flatten(), Err("outer error"));
    }

    #[test]
    fn test_clone() {
        let ok: Result0<i32, &str> = Ok(42);
        let cloned = ok.clone();
        assert_eq!(ok, cloned);
    }

    #[test]
    fn test_debug() {
        let ok: Result0<i32, &str> = Ok(42);
        assert_eq!(format!("{:?}", ok), "Ok(42)");

        let err: Result0<i32, &str> = Err("error");
        assert_eq!(format!("{:?}", err), "Err(\"error\")");
    }
}
