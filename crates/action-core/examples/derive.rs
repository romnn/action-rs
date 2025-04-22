#[cfg(not(feature = "derive"))]
fn main() {
    panic!(r#"feature "derive" must be enabled for this example"#);
}

#[cfg(feature = "derive")]
fn main() {
    use action_core::{Action, Parse, env};
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

    {
        use action_core::input::ParseInput;
        // parse single value as type T using string name
        // this is not recommended, as changes to the action.yml file go unnoticed.
        let env = env::OsEnv;
        let value: Option<String> = env.parse_input::<String>("input name").unwrap();
        dbg!(&value);
    }
}
