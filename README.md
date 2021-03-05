# textcat-rs
N-Gram-Based Text Categorization


## Usage

```rust
use textcat::default::languages;

fn main() {
    let textcat  = languages();
    let text     = "Hi there, this is a simple text written in what language?";
    let language = textcat.get_category(text).unwrap();

    println!("\"{}\" is written in \"{}\"", text, language);
}
```
