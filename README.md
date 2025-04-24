# get_set_macro

[![Crates.io](https://img.shields.io/crates/v/get_set_macro)](https://crates.io/crates/get_set_macro)
[![Docs.rs](https://docs.rs/get_set_macro/badge.svg)](https://docs.rs/get_set_macro)

Procedural macro to automatically generate getters and setters for struct fields in Rust, with fine-grained control over behavior.

## Features

- Generate **getters** that return either **references** or **copies**, depending on your needs.
- Automatically generate **setters** for fields.
- Customize method names for getters and setters.
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

Apply it to a struct:
```rust
#[get_set]
struct Example {
    #[gsflag(get)]
    name: String,

    #[gsflag(get_copy)]
    age: u32,

    #[gsflag(get(rename = "city_ref"), set(rename = "set_city" /* same as default */))]
    city: String,
}
```

Generates
```rust
struct Example {
    name: String,
    age: u32,
    city: String,
}

impl Example {
    get_name(&self) -> &String {
        &self.name
    }
    get_age(&self) -> u32 {
        self.age
    }
    city_ref(&self) -> &String {
        &self.city
    }
    set_city(&mut self, new_city: String) {
        self.city = new_city;
    }
}
```

## Attributes

| Attribute        | Description |
|:-----------------|:------------|
| `#[getr]`         | Generate a getter that returns a **reference**. |
| `#[get_copy]`     | Generate a getter that returns a **copy**. (Use only with `Copy` types.) |
| `#[setr]`         | Generate a setter that sets a new value. |
| `name = "..."`    | Customize the method name (e.g., `#[getr(name = "fetch_name")]`). |

> **Note:** Only structs with **named fields** are currently supported.

---

## Limitations

- Only named fields (`struct Foo { x: T }`) are supported — **tuple structs** and **unit structs** are not yet supported.
- No automatic validation that `#[get_copy]` fields are `Copy` yet (coming soon).

---

## Planned Features

- Optional `#[inline(always)]` on generated methods.
- Automatic validation for `Copy` types when using `#[get_copy]`.
- More granular control over visibility (e.g., private vs public methods).
- Builder pattern support.

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