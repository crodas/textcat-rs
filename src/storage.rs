use crate::ngram::Ngrams;
use glob::{glob, Paths};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Write};

pub type IoResult<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
struct Category {
    name: String,
    ngrams: Ngrams,
}

impl Category {
    pub fn distance(&self, ngrams: &Ngrams) -> u64 {
        self.ngrams.distance(ngrams)
    }

    /// Creates a struct from a vector (the output of self.to_vec())
    pub fn from_vec(name: &str, ngrams: Vec<&str>) -> Self {
        Category {
            name: name.to_string(),
            ngrams: Ngrams::from_vec_str(ngrams),
        }
    }

    /// Exports the current structure as a vector
    pub fn to_vec(&self) -> Vec<&str> {
        self.ngrams.to_vec()
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileContent {
    /// Version of the file format. Not used at the moment but it will allow the program
    /// to refuse to work older file formats.
    version: String,

    /// List of categories with their features/n-grams
    categories: Vec<Category>,

    /// Runtime configuration.
    ///
    /// Minimun threshold to (0.00-0.99) to consider a match close enough from each
    /// other. This setting can be updated with update set_threshold(0.01), the default
    /// value is 3% (0.03)
    #[serde(
        skip_deserializing,
        skip_serializing,
        default = "FileContent::default_threshold"
    )]
    threshold: f32,
}

#[allow(clippy::new_without_default)]
impl FileContent {
    pub fn new() -> FileContent {
        FileContent {
            categories: Vec::new(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            threshold: FileContent::default_threshold(),
        }
    }

    /// Converts an vector into a struct (the output of self.to_vec())
    pub fn from_vec(data: Vec<(&str, Vec<&str>)>) -> FileContent {
        let categories = data
            .iter()
            .map(|(name, ngrams)| Category::from_vec(name, ngrams.to_vec()))
            .collect();

        FileContent {
            categories,
            version: env!("CARGO_PKG_VERSION").to_string(),
            threshold: FileContent::default_threshold(),
        }
    }

    /// Converts the current structure into a vector (language, [ngrams])
    pub fn to_vec(&self) -> Vec<(&str, Vec<&str>)> {
        self.categories
            .iter()
            .map(|category| (category.name.as_str(), category.to_vec()))
            .collect()
    }

    pub fn set_threshold(&mut self, threshold: f32) -> Result<(), &str> {
        if threshold <= 0.0 && 1.0 <= threshold {
            return Err("The value has to between 0 and 1");
        }

        self.threshold = threshold;

        Ok(())
    }

    pub fn default_threshold() -> f32 {
        0.03
    }

    /// Returns a single category for a given text. If two categories or more categories
    /// that are close together None will be returned.
    pub fn get_category(&self, sample: &str) -> Option<String> {
        if let Some(categories) = self.get_categories(sample) {
            if categories.len() == 1 {
                return Some(categories[0].0.to_owned());
            }
        }

        None
    }

    /// Returns a sorted list of categories which are candidates and their score (the lower the better)
    pub fn get_categories(&self, sample: &str) -> Option<Vec<(String, u64)>> {
        let ngrams = Ngrams::new(sample, 5);

        let mut categories = self
            .categories
            .iter()
            .map(|category| (category.distance(&ngrams), category))
            .collect::<Vec<(u64, &Category)>>();

        categories.sort_by(|a, b| a.0.cmp(&b.0));

        let best_candidate = categories.first()?;
        let threshold: u64 = ((1.0+self.threshold) * best_candidate.0 as f32) as u64;

        Some(
            categories
                .iter()
                .filter(|p| threshold > p.0)
                .map(|p| (p.1.name.clone(), p.0))
                .collect(),
        )
    }

    /// Stores the categories in a JSON file.
    pub fn persist(&self, output: &str) -> IoResult<()> {
        let j = serde_json::to_string(&self)?;
        File::create(output)?.write_all(j.as_bytes())?;
        Ok(())
    }

    /// Add sample text to learn a new category.
    pub fn add_category(&mut self, name: String, sample: &str) {
        self.categories.push(Category {
            name,
            ngrams: Ngrams::new(&<&str>::clone(&sample), 5),
        });
    }

    pub fn categories(&self) -> Vec<String> {
        self.categories.iter().map(|r| r.name.clone()).collect()
    }
}

/// Loads categories stored from a file.
pub fn load(path: &str) -> IoResult<FileContent> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

/// Learn categories from a given directory. In the directory all the files
/// should have a 'sample' extensions.
pub fn learn_from_directory(path: &str) -> IoResult<FileContent> {
    let files = get_files_from_directory(path)?;
    let mut content = FileContent::new();

    for p in files {
        let mut buf: Vec<u8> = Vec::new();

        let p = p.map_err(|_e| {
            Error::new(ErrorKind::InvalidData, "failed reading glob path")
        })?;

        let _bytes = File::open(p.as_path())?.read_to_end(&mut buf)?;
        let name = p.as_path().file_stem().unwrap().to_str().unwrap();

        let str = String::from_utf8_lossy(&buf).to_string();

        content.add_category(name.to_string(), &str);
    }

    Ok(content)
}

/// Returns all sample files in a given directory
fn get_files_from_directory(path: &str) -> IoResult<Paths> {
    glob(format!("{}/*.sample", path).as_str())
        .map_err(|_p| Error::new(ErrorKind::InvalidData, "invalid data"))
}

mod test {
    #[allow(unused_imports)]
    use crate::storage::{get_files_from_directory, learn_from_directory};

    #[test]
    fn test_files_listing_in_path() {
        let r: Vec<String> = get_files_from_directory(&"tests")
            .expect("Some went wrong")
            .map(|p| p.unwrap())
            .map(|p| p.to_str().clone().unwrap().to_string())
            .collect();

        assert_eq!(vec!["tests/english.sample", "tests/spanish.sample",], r);
    }

    #[test]
    fn test_learn_from_directory() {
        learn_from_directory("tests").expect("failed to read file");
    }
}
