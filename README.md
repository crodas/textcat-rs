# textcat-rs
N-Gram-Based Text Categorization


## Usage

```rust
use textcat::embed::TextCat;

fn main() {
    let textcat  = TextCat::new();
    let text     = "Hi there, this is a simple text written in what language?";
    let language = textcat.get_language(text).unwrap();

    println!("\"{}\" is written in \"{}\"", text, language);
}
```
