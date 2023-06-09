#[cfg(not(feature = "derive"))]
fn main() {
    panic!(r#"feature "derive" must be enabled for this example"#);
}

#[cfg(feature = "derive")]
fn main() {
    use action_core::{self as action, Action, Parse};
    use std::collections::HashMap;

    #[derive(Action)]
    #[action = "./examples/myaction.yml"]
    struct MyAction {}

    // parse all values into a map
    let inputs: HashMap<MyActionInput, Option<String>> = MyAction::parse();
    dbg!(&inputs);

    // parse single value as type T
    let resolve_versions: Option<bool> = MyAction::resolve_versions::<bool>().unwrap();
    dbg!(&resolve_versions);

    // parse single value as type T using string name
    // this is not recommended, as changes to the action.yml file go unnoticed.
    let value: Option<String> = action::get_input::<String>("input name").unwrap();
    dbg!(&value);
}
