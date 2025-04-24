use get_set_macro::get_set;

#[get_set]
struct Example {
    #[gsflags(get)]
    name: String,

    #[gsflags(get_copy)]
    age: u32,

    #[gsflags(get(inline_always, rename = "city_ref"), set(rename = "set_city" /* same as default */))]
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