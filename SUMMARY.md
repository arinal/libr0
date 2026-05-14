# Summary

[Introduction](index.md)

# Part 1: Foundational Types

- [Option](docs/01-option-0-intro.md)
  - [Unwrapping & Defaults](docs/01-option-1-unwrapping.md)
  - [Transforming](docs/01-option-2-transforming.md)
  - [Borrowing & Moving](docs/01-option-3-borrowing.md)
- [Result](docs/02-result-0-intro.md)
  - [Unwrapping & Defaults](docs/02-result-1-unwrapping.md)
  - [Transforming & Chaining](docs/02-result-2-transforming.md)
  - [The ? Operator](docs/02-result-3-question-mark.md)

# Part 2: Smart Pointers & Interior Mutability

- [Box](docs/03-box-0-intro.md)
  - [Building Our Own](docs/03-box-1-building.md)
  - [Advanced Methods & Deref Coercion](docs/03-box-2-advanced.md)
- [Vec](docs/04-vec-0-intro.md)
  - [Slices & Memory Layout](docs/04-vec-1-slices.md)
  - [Operations & Performance](docs/04-vec-2-operations.md)
- [Cell](docs/05-cell-0-intro.md)
  - [Building Our Own](docs/05-cell-1-building.md)
  - [Practice & Thread Safety](docs/05-cell-2-practice.md)
- [RefCell](docs/06-refcell.md)
- [Rc](docs/07-rc-0-intro.md)
  - [Usage, Patterns & Pitfalls](docs/07-rc-1-usage.md)
  - [Building Our Own](docs/07-rc-2-building.md)
- [Rc + RefCell](docs/08-rc-refcell-0-intro.md)
  - [Patterns & Pitfalls](docs/08-rc-refcell-1-patterns.md)
  - [Implementing Weak](docs/08-rc-refcell-2-weak.md)
  - [ManuallyDrop & Anti-patterns](docs/08-rc-refcell-3-manuallydrop.md)

# Appendix

- [Closures](docs/appendix-closures-0-intro.md)
  - [The move Keyword](docs/appendix-closures-1-move.md)
  - [Size & Type Annotations](docs/appendix-closures-2-size-types.md)
- [Memory Layout](docs/appendix-memory-layout-0-intro.md)
  - [Stack, Heap & Static Data](docs/appendix-memory-layout-1-stack-heap.md)
  - [Raw Pointers & Unsafe](docs/appendix-memory-layout-2-pointers.md)
  - [Visualizing Types](docs/appendix-memory-layout-3-types.md)
- [Sized](docs/appendix-sized.md)
- [Nested Types](docs/appendix-nested-types.md)