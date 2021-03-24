use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::min;
use std::collections::HashMap;
use std::iter::FromIterator;
use unicode_segmentation::UnicodeSegmentation;

pub struct Ngram((String, u64));

impl Ngram {
    fn ngram(&self) -> &String {
        &self.0 .0
    }

    fn value(&self) -> u64 {
        self.0 .1
    }
}

impl Serialize for Ngram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.ngram())
    }
}

impl<'de> Deserialize<'de> for Ngram {
    fn deserialize<D>(deserializer: D) -> Result<Ngram, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str = Deserialize::deserialize(deserializer)?;
        Ok(Ngram((str, 0)))
    }
}

pub struct Ngrams {
    ngrams: Vec<Ngram>,
    index: HashMap<String, usize>,
}

impl<'de> Deserialize<'de> for Ngrams {
    fn deserialize<D>(deserializer: D) -> Result<Ngrams, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ngrams: Vec<Ngram> =
            Deserialize::deserialize(deserializer)?;
        Ok(Ngrams::from_vec(ngrams))
    }
}

impl Serialize for Ngrams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let l = min(400, self.ngrams.len());
        self.ngrams[0..l].serialize(serializer)
    }
}

impl Ngrams {
    /// Creates a new Ngrams structure from a given text
    /// (the ngrams length are from 2 ... length).
    pub fn new(text: &str, length: i8) -> Ngrams {
        let mut ngrams = Ngrams::parse_text(text, length as usize)
            .into_iter()
            .map(Ngram)
            .collect::<Vec<Ngram>>();

        ngrams.sort_by(|a, b| {
            if a.value() == b.value() {
                b.ngram().cmp(&a.ngram())
            } else {
                b.value().cmp(&a.value())
            }
        });

        Ngrams::from_vec(ngrams)
    }

    pub fn to_vec(&self) -> Vec<&str> {
        self.ngrams.iter().map(|w| w.0 .0.as_str()).collect()
    }

    pub fn from_vec_str(ngrams: Vec<&str>) -> Ngrams {
        let ngrams = ngrams
            .iter()
            .map(|w| Ngram((w.to_string(), 0)))
            .collect();

        Self::from_vec(ngrams)
    }

    /// Takes a vector of Ngram and builds an index, useful to locate Ngrams quickly
    /// using a hash. This should be called from the deserializer or automatically
    /// from the constructor _if_ we are parsing a new text.
    fn from_vec(ngrams: Vec<Ngram>) -> Ngrams {
        let mut index = HashMap::new();

        for (pos, ngram) in ngrams.iter().enumerate() {
            index.entry(ngram.ngram().clone()).or_insert(pos);
        }

        Ngrams { ngrams, index }
    }

    /// Creates a HashMap of text -> count
    fn parse_text(text: &str, length: usize) -> HashMap<String, u64> {
        let mut ngrams: HashMap<String, u64> = HashMap::new();
        let text: Vec<char> = text
            .to_lowercase()
            .unicode_words()
            .fold(String::new(), |a, b| a + "_" + b)
            .chars()
            .collect::<Vec<_>>();

        let text_length = text.len();

        for i in 0..text_length {
            for len in 1..(length + 1) {
                if i + len > text_length {
                    break;
                }

                if len == 1
                    && (text[i].is_numeric()
                        || text[i].is_ascii_punctuation())
                {
                    continue;
                }

                let ngram = String::from_iter(&text[i..i + len]);

                if ngram.is_empty() {
                    break;
                }

                let count = ngrams.entry(ngram).or_insert(0);
                *count += 1;
            }
        }

        ngrams
    }

    /// Very simple distance algorithm know as Out of place[1]
    ///
    /// TODO: experiment with other more sophisticated distances algorithm like PageRank (although that
    /// would require a serialization change).
    ///
    /// [1] https://www.researchgate.net/figure/Out-of-Place-Measure-Computation-adapted-from-Cavnar-and-Trenkle-1994_fig2_220746484
    pub fn distance(&self, another: &Ngrams) -> u64 {
        self.ngrams
            .iter()
            .map(|n| {
                another
                    .position(n.ngram())
                    .map_or(5000_u64, |v| v as u64)
            })
            .sum()
    }

    #[allow(dead_code)]
    pub fn get(&self, pos: usize) -> &Ngram {
        &self.ngrams[pos]
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.ngrams.len()
    }

    /// Returns the position in which a given Ngram is located
    /// in the ranking or None if it is not found.
    pub fn position(&self, ngram: &str) -> Option<usize> {
        if let Some(pos) = self.index.get(ngram) {
            return Some(*pos);
        }

        None
    }

    /// Search for an ngram and returns the Ngram struct or None.
    #[allow(dead_code)]
    fn ngram(&self, ngram: &str) -> Option<&Ngram> {
        if let Some(pos) = self.index.get(ngram) {
            return Some(&self.ngrams[*pos]);
        }

        None
    }
}

mod tests {
    #[allow(unused_imports)]
    use crate::ngram::Ngrams;

    #[test]
    fn length() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done.".to_string(),
            4,
        );

        assert_eq!(160, ngrams.len());
    }

    #[test]
    fn get_count() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done.".to_string(),
            4,
        );
        assert_eq!(10, ngrams.get(0).value());
        assert_eq!(
            2,
            ngrams.ngram(&"is".to_string()).unwrap().value()
        );
        assert_eq!(
            1,
            ngrams.ngram(&"this".to_string()).unwrap().value()
        );
    }

    #[test]
    fn get_by_position() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done.".to_string(),
            4,
        );
        assert_eq!("e", ngrams.get(0).ngram());
    }

    #[test]
    fn search() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done.".to_string(),
            4,
        );
        assert_eq!(true, ngrams.ngram(&"notf".to_string()).is_none());
        assert_eq!(true, ngrams.ngram(&"this".to_string()).is_some());
        assert_eq!(Some(5), ngrams.position(&"_t".to_string()))
    }
}
