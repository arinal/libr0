# Appendix: Nested Types - Understanding Order and Flexibility

When combining smart pointers like `Rc`, interior mutability types like `RefCell`, and containers like `Option`, the order matters! This appendix explores the different combinations and what they mean.

> **Note:** This is a placeholder for future completion. The content below covers basic patterns, but will be expanded with more cases and exercises.

## The Order Matters

Each wrapper in a nested type determines what operations are possible on what's inside. **Read from outside-in** to understand capabilities:
- `RefCell<...>` - Can mutate what's inside
- `Rc<...>` - Can share (multiple owners)
- `Option<...>` - Might not be present (fixed at construction)

## Common Patterns

### `RefCell<Option<Rc<T>>>` - Can add/remove/change the link

```rust
struct Node {
    next: RefCell<Option<Rc<Node>>>,
}

let node = Rc::new(Node { next: RefCell::new(None) });

// ✅ Can add a link (None → Some)
*node.next.borrow_mut() = Some(Rc::clone(&other_node));

// ✅ Can change which node (Some(a) → Some(b))
*node.next.borrow_mut() = Some(Rc::clone(&another_node));

// ✅ Can remove the link (Some → None)
*node.next.borrow_mut() = None;
```

**Use when:** You need full flexibility - add, remove, or change links.

### `Option<RefCell<Rc<T>>>` - Can change the link, but can't add/remove

```rust
struct Node {
    next: Option<RefCell<Rc<Node>>>,
}

let node_with_link = Rc::new(Node {
    next: Some(RefCell::new(Rc::clone(&other_node))),
});

let node_without_link = Rc::new(Node { next: None });

// ✅ If Some, can change which node
if let Some(ref cell) = &node_with_link.next {
    *cell.borrow_mut() = Rc::clone(&another_node);
}

// ❌ Can't add a link to node_without_link (it's None forever)
// ❌ Can't remove the link from node_with_link (it's Some forever)
```

**Use when:** The presence/absence of the link is determined at construction and never changes, but you need to change which `Rc` it points to.

### `Rc<RefCell<Option<T>>>` - Multiple owners of a changeable optional value

```rust
let cache: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
let cache1 = Rc::clone(&cache);
let cache2 = Rc::clone(&cache);

// Multiple owners can all change the value
*cache1.borrow_mut() = Some(String::from("cached"));
*cache2.borrow_mut() = None;  // Clear the cache
```

**Use when:** Multiple owners need to mutate the same optional value (shared cache, shared state).

### `Option<Rc<RefCell<T>>>` - Optional shared mutable value

```rust
struct Config {
    logger: Option<Rc<RefCell<Logger>>>,
}

let logger = Rc::new(RefCell::new(Logger::new()));
let config = Config {
    logger: Some(Rc::clone(&logger)),
};

// ✅ If Some, multiple owners can mutate the logger
if let Some(ref logger_rc) = config.logger {
    logger_rc.borrow_mut().log("message");
}

// ❌ Can't change from Some to None (fixed at construction)
```

**Use when:** The optional value is determined at construction, but if present, it needs to be shared and mutable.

## TODO: Future Sections

- [ ] More combinations with `Box`, `Vec`, `Arc`, `Mutex`
- [ ] Decision flowchart for choosing the right pattern
- [ ] Performance implications of different orderings
- [ ] Exercises for each pattern
- [ ] Common mistakes and how to fix them
- [ ] Real-world examples from popular crates
