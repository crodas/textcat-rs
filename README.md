# textcat-rs
N-Gram-Based Text Categorization with built-in support for natural language detection.


## Usage - Natural language.

```rust
use textcat::embed::TextCat;

fn main() {
    let textcat  = TextCat::new();
    let text     = "Hi there, this is a simple text written in what language?";
    let language = textcat.get_language(text).unwrap();

    println!("\"{}\" is written in \"{}\"", text, language);
}
```

### Adding support for languages.

Adding support to new languages is quite trivial. Add a new sample file in `embed/samples` and run `cargo run` in the `embed` sub-project. This step will [embed the new features for the new language](https://github.com/crodas/textcat-rs/blob/develop/src/embed.rs).
