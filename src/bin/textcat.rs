use std::env;
use std::io;
use textcat::default::languages;
use textcat::storage::load;

fn main() {
    let args: Vec<String> = env::args().collect();

    let db = if args.len() == 1 {
        languages()
    } else {
        load(&args[1]).unwrap()
    };

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("failed to read from pipe");

    println!("Languages: {}", db.categories().join(", "));
    println!(
        "Language: {}",
        db.get_category(&input)
            .unwrap_or_else(|| "Unknown".to_string())
    );
    println!("Input text: {}", input);
}

mod test {
    use serde::{Deserialize, Serialize};

    #[allow(unused_imports)]
    use std::fs::File;
    #[allow(unused_imports)]
    use std::io::BufReader;
    #[allow(unused_imports)]
    use textcat::default::languages;

    #[derive(Deserialize, Serialize)]
    struct Samples {
        category: String,
        samples: Vec<String>,
    }

    #[test]
    fn test_list_of_samples() {
        let file = File::open("tests/samples.json").unwrap();
        let reader = BufReader::new(file);
        let samples: Vec<Samples> =
            serde_json::from_reader(reader).unwrap();

        let textcat = languages();

        samples
            .iter()
            .map(|sample| {
                sample
                    .samples
                    .iter()
                    .map(move |t| (sample.category.clone(), t))
            })
            .flatten()
            .map(|t| {
                assert_eq!(t.0, textcat.get_category(t.1).unwrap());
                true
            })
            .for_each(drop);
    }
}
