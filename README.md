# textcat-rs

Library to extract N-Grams from texts. This is a low level library.
[Lingo](https://github.com/crodas/lingo-rs) is build on top of this library to
detect human languages on texts.

This library provides tools to train with sample texts, extracting N-Grams from
texts, create sample and train categories. The trained data can be serialized to
be used later. The library also provides tools to detect to which pretained
category a given text would be closer to.
