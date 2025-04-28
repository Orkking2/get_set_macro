use get_set_macro::get_set;

#[get_set(default(inline_always), get)]
struct Example {
    // The function that this generates will still be #[inline(always)]
    #[gsflags(get(rename = "renamed_get_name"))]
    name: String,

    // Has noinline set, will override the global inline
    // Equalivalent to 
    // #[gsflags(get(noinline))]
    #[gsflags(default(noinline), get)]
    age: u32,

    // Inherits the following from global settings: 
    // #[gsflags(default(inline_always), get)]
    unflagged: i64,

    #[gsflags(skip)]
    skipped: f32,

    // Removes the `inline_always` default, get (`city_ref`) is #[inline(never)] while `set_city` has no inline attribute (because of the new default)
    #[gsflags(default(noinline), get(inline_never, rename = "city_ref"), set(rename = "set_city" /* same as default */))]
    city: String,
}

fn main() {
    let mut example = Example {
        name: "ExampleName".to_string(),
        age: 55,
        unflagged: -128,
        skipped: 12.32,
        city: "ExampleCity".to_string(),
    };

    assert_eq!("ExampleName", example.get_name().as_str());
    assert_eq!(example.renamed_get_name(), example.get_name());
    assert_eq!(55, *example.get_age());
    assert_eq!("ExampleCity", example.city_ref().as_str());

    example.set_city("NewCity".to_string());

    assert_eq!("NewCity", example.city_ref().as_str());

    assert_eq!(-128, *example.get_unflagged());

    // The following would produce an error, see fail_allsetting.rs
    // example.get_skipped();
}