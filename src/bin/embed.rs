use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use textcat::storage::learn_from_directory;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _p = learn_from_directory(&args[1]).unwrap();

    let code = format!(
        "use crate::storage::FileContent;

    /// Load default categories
    /// We should never pay the price of decoding a JSON which is already compiled.
    ///
    /// Future versions will generate code that will not rely on serde for embedded deserialization.
    pub fn languages() -> FileContent {{
        serde_json::from_str(r#\"
            {}
        \"#).unwrap()
    }}",
        serde_json::to_string_pretty(&_p).unwrap()
    );

    File::create("./src/default.rs")
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap();

    Command::new("cargo").arg("fmt").output().unwrap();
}
