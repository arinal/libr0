//! MyOption - Educational reimplementation of Option<T>

#[derive(Debug, Clone, PartialEq)]
pub enum MyOption<T> {
    Some(T),
    None,
}

pub use MyOption::{None, Some};

impl<T> MyOption<T> {
    /// Returns `true` if the option is a [`Some`] value.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert!(Some(42).is_some());
    /// assert!(!None::<i32>.is_some());
    /// ```
    pub fn is_some(&self) -> bool {
        matches!(self, Some(_))
    }

    /// Returns `true` if the option is a [`None`] value.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert!(!Some(42).is_none());
    /// assert!(None::<i32>.is_none());
    /// ```
    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns the contained value, panicking if [`None`].
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(42).unwrap(), 42);
    /// ```
    pub fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called unwrap on a None value"),
        }
    }

    /// Returns the contained value or a default.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(42).unwrap_or(0), 42);
    /// assert_eq!(None.unwrap_or(0), 0);
    /// ```
    pub fn unwrap_or(self, or: T) -> T {
        match self {
            Some(val) => val,
            None => or,
        }
    }

    /// Returns the contained value or computes it from a closure.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(42).unwrap_or_else(|| 0), 42);
    /// assert_eq!(None.unwrap_or_else(|| 100), 100);
    /// ```
    pub fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Some(val) => val,
            None => f(),
        }
    }

    /// Maps a [`MyOption<T>`] to [`MyOption<U>`] by applying a function.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(5).map(|x| x * 2), Some(10));
    /// assert_eq!(None.map(|x: i32| x * 2), None);
    /// ```
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => MyOption::Some(f(x)),
            None => MyOption::None,
        }
    }

    /// Applies a function that returns a [`MyOption`].
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// let sq = |x: i32| Some(x * x);
    /// assert_eq!(Some(2).and_then(sq), Some(4));
    /// ```
    pub fn and_then<U, F: FnOnce(T) -> MyOption<U>>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => f(x),
            None => MyOption::None,
        }
    }

    /// Returns [`None`] if predicate returns `false`.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(4).filter(|x| x % 2 == 0), Some(4));
    /// assert_eq!(Some(3).filter(|x| x % 2 == 0), None);
    /// ```
    pub fn filter<P: FnOnce(&T) -> bool>(self, predicate: P) -> MyOption<T> {
        match self {
            Some(x) if predicate(&x) => Some(x),
            _ => None,
        }
    }

    /// Converts from `&MyOption<T>` to `MyOption<&T>`.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// let x = Some(String::from("hello"));
    /// assert_eq!(x.as_ref().map(|s| s.len()), Some(5));
    /// ```
    pub fn as_ref(&self) -> MyOption<&T> {
        match self {
            Some(x) => MyOption::Some(x),
            None => MyOption::None,
        }
    }

    /// Takes the value out, leaving [`None`] in its place.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// let mut x = Some(42);
    /// assert_eq!(x.take(), Some(42));
    /// // x is now None
    /// ```
    pub fn take(&mut self) -> MyOption<T> {
        std::mem::replace(self, None)
    }

    /// Returns the option if [`Some`], otherwise returns `other`.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(1).or(Some(2)), Some(1));
    /// assert_eq!(None.or(Some(2)), Some(2));
    /// ```
    pub fn or(self, other: MyOption<T>) -> MyOption<T> {
        match self {
            Some(x) => Some(x),
            None => other,
        }
    }

    /// Returns the option if [`Some`], otherwise calls `f`.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// assert_eq!(Some(1).or_else(|| Some(2)), Some(1));
    /// assert_eq!(None.or_else(|| Some(2)), Some(2));
    /// ```
    pub fn or_else<F: FnOnce() -> MyOption<T>>(self, f: F) -> MyOption<T> {
        match self {
            Some(x) => Some(x),
            None => f(),
        }
    }
}

impl<T, U> MyOption<(T, U)> {
    /// Unzips an option containing a tuple into a tuple of options.
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// Some((1, "hello")).unzip(); // (Some(1), Some("hello"))
    /// None::<(i32, &str)>.unzip(); // (None, None)
    /// ```
    pub fn unzip(self) -> (MyOption<T>, MyOption<U>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

impl<T> MyOption<MyOption<T>> {
    /// Converts from [`MyOption<MyOption<T>>`] to [`MyOption<T>`].
    /// ```
    /// use rustlib::option::{MyOption, Some, None};
    /// Some(Some(42)).flatten(); // Some(42)
    /// Some(None::<i32>).flatten(); // None
    /// ```
    pub fn flatten(self) -> MyOption<T> {
        match self {
            Some(inner) => inner,
            None => None,
        }
    }
}

/// Zips two options together into a tuple.
/// Returns [`None`] if either option is [`None`].
/// ```
/// use rustlib::option::{MyOption, Some, None, zip};
/// zip(Some(1), Some("hello")); // Some((1, "hello"))
/// zip(Some(1), None::<&str>); // None
/// ```
pub fn zip<T, U>(a: MyOption<T>, b: MyOption<U>) -> MyOption<(T, U)> {
    match (a, b) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_some_is_none() {
        let x: MyOption<i32> = Some(42);
        assert!(x.is_some());
        assert!(!x.is_none());

        let y: MyOption<i32> = None;
        assert!(!y.is_some());
        assert!(y.is_none());
    }

    #[test]
    fn test_unwrap() {
        assert_eq!(Some(42).unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "called unwrap on a None value")]
    fn test_unwrap_none_panics() {
        let x: MyOption<i32> = None;
        x.unwrap();
    }

    #[test]
    fn test_unwrap_or() {
        assert_eq!(Some(42).unwrap_or(0), 42);
        assert_eq!(None.unwrap_or(0), 0);
    }

    #[test]
    fn test_unwrap_or_else() {
        assert_eq!(Some(42).unwrap_or_else(|| 0), 42);
        assert_eq!(None.unwrap_or_else(|| 100), 100);
    }

    #[test]
    fn test_map() {
        assert_eq!(Some(10).map(|x| x * 2), Some(20));
        assert_eq!(None.map(|x: i32| x * 2), None);
    }

    #[test]
    fn test_map_chain() {
        let result = Some(5).map(|x| x * 2).map(|x| x + 3);
        assert_eq!(result, Some(13));
    }

    #[test]
    fn test_and_then() {
        let parse = |x: &str| {
            if x.parse::<i32>().is_ok() {
                Some(x.parse::<i32>().unwrap())
            } else {
                None
            }
        };

        assert_eq!(Some("42").and_then(parse), Some(42));
        assert_eq!(Some("abc").and_then(parse), None);
        assert_eq!(None.and_then(parse), None);
    }

    #[test]
    fn test_filter() {
        assert_eq!(Some(4).filter(|x| x % 2 == 0), Some(4));
        assert_eq!(Some(3).filter(|x| x % 2 == 0), None);
        assert_eq!(None.filter(|x: &i32| x % 2 == 0), None);
    }

    #[test]
    fn test_as_ref() {
        let x = Some(String::from("hello"));
        let len = x.as_ref().map(|s| s.len());
        assert_eq!(len, Some(5));
        assert_eq!(x, Some(String::from("hello"))); // x still valid
    }

    #[test]
    fn test_take() {
        let mut x = Some(42);
        assert_eq!(x.take(), Some(42));
        assert_eq!(x, None);

        let mut y: MyOption<i32> = None;
        assert_eq!(y.take(), None);
    }

    #[test]
    fn test_or() {
        assert_eq!(Some(1).or(Some(2)), Some(1));
        assert_eq!(None.or(Some(2)), Some(2));
        assert_eq!(Some(1).or(None), Some(1));
        let none1: MyOption<i32> = None;
        let none2: MyOption<i32> = None;
        assert_eq!(none1.or(none2), None);
    }

    #[test]
    fn test_or_else() {
        assert_eq!(Some(1).or_else(|| Some(2)), Some(1));
        assert_eq!(None.or_else(|| Some(2)), Some(2));
    }

    #[test]
    fn test_zip() {
        assert_eq!(zip(Some(1), Some("hello")), Some((1, "hello")));
        let none_str: MyOption<&str> = None;
        assert_eq!(zip(Some(1), none_str), None);
        let none_int: MyOption<i32> = None;
        assert_eq!(zip(none_int, Some("hello")), None);
        let none1: MyOption<i32> = None;
        let none2: MyOption<&str> = None;
        assert_eq!(zip(none1, none2), None);
    }

    #[test]
    fn test_unzip() {
        let x: MyOption<(i32, &str)> = Some((1, "hello"));
        assert_eq!(x.unzip(), (Some(1), Some("hello")));

        let y: MyOption<(i32, &str)> = None;
        assert_eq!(y.unzip(), (None, None));
    }

    #[test]
    fn test_flatten() {
        assert_eq!(Some(Some(42)).flatten(), Some(42));
        let none_inner: MyOption<i32> = None;
        assert_eq!(Some(none_inner).flatten(), None);
        let none_outer: MyOption<MyOption<i32>> = None;
        assert_eq!(none_outer.flatten(), None);
    }

    #[test]
    fn test_clone() {
        let x = Some(42);
        let y = x.clone();
        assert_eq!(x, y);
    }

    #[test]
    fn test_debug() {
        let x = Some(42);
        assert_eq!(format!("{:?}", x), "Some(42)");

        let y: MyOption<i32> = None;
        assert_eq!(format!("{:?}", y), "None");
    }
}
