# What `#[derive(Default)]` generates

## Regular structs

For regular structs almost the same code is generated as for tuple structs
except that it assigns the fields differently.

```rust
# use derive_more::Default;
#
#[derive(Default)]
struct Point2D {
    // #[default("1")]
    pub x: i32,
    pub y: i32,
}

// assert_eq!(1, Point2D::default().x);
// assert_eq!(1, Point2D::X_INIT);

```
