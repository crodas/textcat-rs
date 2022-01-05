//! # NGram
//!
//! NGram module. This module is responsible for parsing and sorting ngrams from texts.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::min;
use std::collections::HashMap;
use std::iter::FromIterator;
use unicode_segmentation::UnicodeSegmentation;

/// Ngram structure
///
/// An ngram is a tuple the ngram (string) and its score
#[derive(Debug, Clone)]
pub struct Ngram((String, u64));

impl Ngram {
    /// Returns a reference to the ngram
    pub fn ngram(&self) -> &String {
        &self.0 .0
    }

    /// Returns the score of the ngram
    pub fn score(&self) -> u64 {
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

/// Set of ngrams
#[derive(Debug, Clone)]
pub struct Ngrams {
    ngrams: Vec<Ngram>,
    index: HashMap<String, usize>,
}

impl From<Vec<&str>> for Ngrams {
    fn from(value: Vec<&str>) -> Self {
        value
            .iter()
            .map(|w| Ngram((w.to_string(), 0)))
            .collect::<Vec<Ngram>>()
            .into()
    }
}

impl From<Vec<Ngram>> for Ngrams {
    fn from(ngrams: Vec<Ngram>) -> Self {
        let mut index = HashMap::new();

        for (pos, ngram) in ngrams.iter().enumerate() {
            index.entry(ngram.ngram().clone()).or_insert(pos);
        }

        Ngrams { ngrams, index }
    }
}

impl<'de> Deserialize<'de> for Ngrams {
    fn deserialize<D>(deserializer: D) -> Result<Ngrams, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ngrams: Vec<Ngram> = Deserialize::deserialize(deserializer)?;
        Ok(ngrams.into())
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
    pub fn new(text: &str, length: u8) -> Ngrams {
        let mut ngrams = Ngrams::parse_text(text, length as usize)
            .into_iter()
            .map(Ngram)
            .collect::<Vec<Ngram>>();

        ngrams.sort_by(|a, b| {
            if a.score() == b.score() {
                b.ngram().cmp(&a.ngram())
            } else {
                b.score().cmp(&a.score())
            }
        });

        ngrams.into()
    }

    /// Returns a vector of strings of ngrams sorted by the rank
    pub fn to_vec(&self) -> Vec<&str> {
        self.ngrams.iter().map(|w| w.0 .0.as_str()).collect()
    }

    /// Splits the texts from ngrams, from start to end length. NGrams are in their own
    /// vector grouped by length.
    pub fn split_and_group_by_ngrams(
        text: &str,
        start: usize,
        end: usize,
    ) -> Vec<Vec<String>> {
        let text: Vec<char> = text
            .to_lowercase()
            .unicode_words()
            .fold(String::new(), |a, b| a + "_" + b)
            .chars()
            .collect::<Vec<_>>();

        let mut ngrams_set = Vec::new();

        let text_length = text.len();

        for len in start..end {
            let mut ngrams = Vec::new();

            for i in 0..text_length {
                if i + len > text_length {
                    break;
                }

                if len == 1
                    && (text[i].is_numeric() || text[i].is_ascii_punctuation())
                {
                    continue;
                }
                let ngram = String::from_iter(&text[i..i + len]);
                if ngram.is_empty() {
                    break;
                }

                ngrams.push(ngram);
            }

            ngrams_set.push(ngrams);
        }

        ngrams_set
    }

    /// Splits a given text into ngrams
    pub fn split(text: &str, start: usize, end: usize) -> Vec<String> {
        Self::split_and_group_by_ngrams(text, start, end)
            .into_iter()
            .flatten()
            .collect()
    }

    /// Creates a HashMap of ngram -> count
    pub fn parse_text(text: &str, length: usize) -> HashMap<String, u64> {
        let mut ngrams: HashMap<String, u64> = HashMap::new();

        Self::split(&text, 1, length)
            .iter()
            .map(|ngram| {
                let count = ngrams.entry((&ngram).to_string()).or_insert(0);
                *count += 1;
            })
            .for_each(drop);

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
            .map(|n| another.position(n.ngram()).map_or(5000_u64, |v| v as u64))
            .sum()
    }

    /// Gets an ngram by their position
    pub fn get_by_position(&self, pos: usize) -> Option<&Ngram> {
        self.ngrams.get(pos)
    }

    /// Returns the a length of ngarms
    pub fn len(&self) -> usize {
        self.ngrams.len()
    }

    /// Returns true if the ngrams has a length of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the position in which a given Ngram is located
    /// in the ranking or None if it is not found.
    pub fn position(&self, ngram: &str) -> Option<usize> {
        self.index.get(ngram).map(|pos| *pos)
    }

    /// Search for an ngram and returns the Ngram struct or None.
    fn ngram(&self, ngram: &str) -> Option<&Ngram> {
        self.index.get(ngram).map(|pos| &self.ngrams[*pos])
    }
}

#[cfg(test)]
mod tests {
    use crate::ngram::Ngrams;

    #[test]
    fn length() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done."
                .to_string(),
            5,
        );

        assert_eq!(160, ngrams.len());
    }

    #[test]
    fn get_count() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done."
                .to_string(),
            5,
        );
        assert_eq!(10, ngrams.get_by_position(0).expect("first ngram").score());
        assert_eq!(
            2,
            ngrams.ngram(&"is".to_string()).expect("find ngram").score()
        );
        assert_eq!(
            1,
            ngrams
                .ngram(&"this".to_string())
                .expect("find ngram")
                .score()
        );
    }

    #[test]
    fn get_by_position() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done."
                .to_string(),
            5,
        );
        assert_eq!(
            "e",
            ngrams
                .get_by_position(0)
                .expect("find ngram by position")
                .ngram()
        );
    }

    #[test]
    fn search() {
        let ngrams = Ngrams::new(
            &"hi there, this is a test. Something else needs to be done."
                .to_string(),
            5,
        );
        assert_eq!(true, ngrams.ngram(&"notf".to_string()).is_none());
        assert_eq!(true, ngrams.ngram(&"this".to_string()).is_some());
        assert_eq!(Some(5), ngrams.position(&"_t".to_string()))
    }
}
