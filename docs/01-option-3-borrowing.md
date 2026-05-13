# Borrowing & Moving

## as_ref - Borrow the Inner Value

Convert `&Option0<T>` to `Option0<&T>`:

```rust
impl<T> Option0<T> {
    fn as_ref(&self) -> Option0<&T> {
        match self {
            Some(x) => Some(x),
            None => None,
        }
    }
}
```

**Why do we need this?** Because `map` takes `self` by value - it consumes the Option.

**Examples:**

```rust
// Problem: map consumes the Option
let maybe_name: Option0<String> = Some(String::from("Alice"));
let len = maybe_name.map(|s| s.len());
// println!("{:?}", maybe_name);  // ERROR: maybe_name was moved!

// Solution: Use as_ref() to borrow
let maybe_name: Option0<String> = Some(String::from("Alice"));
let len = maybe_name.as_ref().map(|s| s.len());  // s is &String
len  // Some(5)
println!("{:?}", maybe_name);  // Works! maybe_name still valid

// Multiple operations on the same Option
let data = Some(String::from("hello world"));

let len = data.as_ref().map(|s| s.len());
let uppercase = data.as_ref().map(|s| s.to_uppercase());
let contains = data.as_ref().map(|s| s.contains("world"));

len  // Some(11)
uppercase  // Some("HELLO WORLD")
contains  // Some(true)
// data is still usable!
data  // Some("hello world")

// as_ref with None
let nothing: Option0<String> = None;
let result = nothing.as_ref().map(|s| s.len());
result  // None

// Real-world: Validating without consuming
struct Config {
    api_key: Option0<String>,
}

impl Config {
    fn validate(&self) -> bool {
        // Use as_ref to check without consuming api_key
        self.api_key
            .as_ref()
            .map(|key| key.len() > 10)
            .unwrap_or(false)
    }

    fn get_key(&self) -> Option0<&str> {
        // Convert Option0<String> to Option0<&str>
        self.api_key.as_ref().map(|s| s.as_str())
    }
}

let config = Config {
    api_key: Some(String::from("secret_key_12345")),
};

config.validate()  // true (borrows api_key)
config.validate()  // true (can validate again!)
config.get_key()  // Some("secret_key_12345")

// Chaining with as_ref
let text = Some(String::from("  hello  "));
let trimmed_len = text
    .as_ref()
    .map(|s| s.trim())
    .map(|s| s.len());
trimmed_len  // Some(5)
text  // Some("  hello  ") - original unchanged
```

The key insight: `as_ref()` converts `&Option0<T>` to `Option0<&T>`. Now when `map` consumes the Option, it's consuming an Option of _references_, not the original data.

## take - Extract and Replace with None

Useful for moving values out of mutable references:

```rust
impl<T> Option0<T> {
    fn take(&mut self) -> Option0<T> {
        std::mem::replace(self, None)
    }
}
```

**Examples:**

```rust
// Basic usage: Move value out, leave None
let mut slot: Option0<String> = Some(String::from("hello"));
let taken = slot.take();

taken  // Some("hello")
slot  // None (slot is now None)

// Taking from None returns None
let mut empty: Option0<i32> = None;
let result = empty.take();
result  // None
empty  // None

// Use case: Moving from struct fields
struct Cache {
    data: Option0<String>,
}

impl Cache {
    fn flush(&mut self) -> Option0<String> {
        // Take the data, leaving cache empty
        self.data.take()
    }

    fn get(&self) -> Option0<&str> {
        // Use as_ref for non-destructive access
        self.data.as_ref().map(|s| s.as_str())
    }
}

let mut cache = Cache {
    data: Some(String::from("cached_value")),
};

cache.get()  // Some("cached_value")

let flushed = cache.flush();
flushed  // Some("cached_value")
cache.get()  // None (cache is now empty)

// Taking in a loop
let mut items = vec![
    Some(1),
    Some(2),
    Some(3),
];

let extracted: Vec<i32> = items
    .iter_mut()
    .filter_map(|opt| opt.take())
    .collect();

extracted  // [1, 2, 3]
// All items are now None
items.iter().all(|opt| opt.is_none())  // true

// Conditional take
struct Player {
    weapon: Option0<String>,
}

impl Player {
    fn drop_weapon_if(&mut self, condition: bool) -> Option0<String> {
        if condition {
            self.weapon.take()
        } else {
            None
        }
    }
}

let mut player = Player {
    weapon: Some(String::from("sword")),
};

// Don't drop
let result = player.drop_weapon_if(false);
result  // None
player.weapon  // Some("sword")

// Do drop
let result = player.drop_weapon_if(true);
result  // Some("sword")
player.weapon  // None
```

## The Complete Implementation

See the full code in [`src/option.rs`](src/option.rs) for the complete implementation of `Option0` with all methods.
Also, see the exercises in [01_option.rs](./examples/01_option.rs)

## Key Takeaways

1. **Option is just an enum** - No magic, just two variants
2. **The compiler enforces handling** - Can't ignore the `None` case
3. **map transforms, and_then chains** - Functional programming patterns
4. **unwrap is a code smell** - Prefer `unwrap_or`, `unwrap_or_else`, or pattern matching