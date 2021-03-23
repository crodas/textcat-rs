use std::env;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tera::{Context, Tera};
use textcat::storage::learn_from_directory;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _p = learn_from_directory(&args[1]).unwrap();
    let mut tera = Tera::default();

    let code = "
    use crate::storage::FileContent;
    use std::str::FromStr;

    pub enum Language {
        {% for lang in languages %}
            {{lang|capitalize}},{% endfor %}
    }

    impl FromStr for Language {
        type Err = String;

        fn from_str(name: &str) -> Result<Language, String> {
            match name.to_lowercase().as_str() {
            {% for lang in languages %}
                \"{{lang}}\" => Ok(Self::{{lang|capitalize}}),{% endfor %}
            _ => Err(\"Invalid argument\".to_string()),
            }
        }
    }

    pub struct TextCat {
        built_in: FileContent,
    }

    impl TextCat {
        pub fn new() -> Self {
            TextCat {
                built_in: Self::get_embed_languages(),
            }
        }
        
        pub fn get_language(&self, sample: &str) -> Option<Language> {
            self.built_in
                .get_category(sample)
                .map(|r| Language::from_str(r.as_str()).unwrap())
        }

        pub fn get_embed_languages() -> FileContent {
            FileContent::from_vec(vec![
            {% for c in ngrams %}
                (
                    \"{{c.0}}\",
                    vec![
                    {% for ngram in c.1|slice(end=400) %}
                        \"{{ngram}}\",{% endfor %}
                    ]
                ),{% endfor %}
            ]
            )
        }
    }

    ///
    /// Load default categories
    /// We should never pay the price of decoding a JSON which is already compiled.
    ///
    /// Future versions will generate code that will not rely on serde for embedded deserialization.
    pub fn languages() -> FileContent {
        FileContent::new()
    }
    ";

    tera.add_raw_template("embed", code).unwrap();

    let mut context = Context::new();
    context.insert("ngrams", &_p.to_vec());
    context.insert("languages", &_p.categories());
    context.insert("version", &env!("CARGO_PKG_VERSION").to_string());

    File::create("./src/default.rs")
        .unwrap()
        .write_all(tera.render("embed", &context).unwrap().as_bytes())
        .unwrap();

    Command::new("cargo").arg("fmt").output().unwrap();
}
