# get_set_macro

[![Crates.io](https://img.shields.io/crates/v/get_set_macro)](https://crates.io/crates/get_set_macro)
[![Docs.rs](https://docs.rs/get_set_macro/badge.svg)](https://docs.rs/get_set_macro)

Procedural macro to automatically generate getters and setters for struct fields in Rust, with fine-grained control over behavior.

## Features

- Generate **getters** that return either **references** or **copies**, depending on your needs.
- Automatically generate **setters** for fields.
- Customize method names for getters and setters.
- Choose to have a getter or setter be not inlined with `noinline`
- Return proper compiler errors instead of panicking.
- Lightweight and minimal dependencies.

---

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
get_set_macro = "0.1"
```

---

## Usage

Import the macro:

```rust
use get_set_macro::get_set;
```

Apply it to a struct: (see [`tests/ui/ok_readme.rs`](./tests/ui/ok_readme.rs))
```rust
#[get_set]
struct Example {
    #[gsflag(get)]
    name: String,

    #[gsflag(get_copy)]
    age: u32,

    #[gsflag(get(inline_always, rename = "city_ref"), set(rename = "set_city"))]
    city: String,
}

// Has functionality
fn main() {
    let mut example = Example {
        name: "ExampleName".to_string(),
        age: 55,
        city: "ExampleCity".to_string(),
    };

    assert_eq!("ExampleName", example.get_name().as_str());
    assert_eq!(55, example.get_age());
    assert_eq!("ExampleCity", example.city_ref().as_str());

    example.set_city("NewCity".to_string());

    assert_eq!("NewCity", example.city_ref().as_str());
}
```

## Attributes

| Attribute | Description |
|:-|:-|
| `#[get]` | Generate a getter that returns a **reference**. |
| `#[get_copy]` | Generate a getter that returns a **copy**. (Use only with `Copy` types.) |
| `#[set]` | Generate a setter that sets a new value. |
| `rename = "..."` | Customize the method name (e.g., `#[gsflags(get(rename = "fetch_name"))]`). |
| `inline(\|_always\|_never)` | Choose to have a getter or setter inlined and how (e.g. `#[gsflags(get(inline_always, rename = "always_inlined_get"))]`). |

> **Note:** Only structs with **named fields** are currently supported.

---

## Limitations

- Only named fields (`struct Foo { x: T }`) are supported — **tuple structs** and **unit structs** are not yet supported.

---

## Planned Features



---

## Contributing

Pull requests, issues, and suggestions are welcome!

If you find a bug or would like to request a feature, feel free to open an issue.

---

## License

This project is licensed under the [MIT License](LICENSE).

---

## Acknowledgments

I did this.