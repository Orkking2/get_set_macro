use get_set_macro::get_set;

#[get_set(get)]
struct Example {
    // Will not have any getters or setters generated
    #[gsflags(skip)]
    skipped: f32,
}

fn main() {
    let mut example = Example {
        skipped: 1.0,
    };

    // Should be no method named `get_skipped` generated for `Example`.
    example.get_skipped();
}