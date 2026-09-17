#![allow(dead_code)]
//! Core classification, extension normalization, and rule matching engine.

pub mod classifier;
pub mod config;
pub mod normalizer;
pub mod sniffer;
pub mod taxonomy;

#[allow(unused_imports)]
pub use classifier::{ClassificationResult, Classifier, IgnoreReason};
#[allow(unused_imports)]
pub use config::{CompiledRules, Config, Settings};
#[allow(unused_imports)]
pub use normalizer::{format_collision_name, parse_file_name, ExtInfo, COMPOUND_EXTENSIONS};
#[allow(unused_imports)]
pub use sniffer::{sniff_bytes, sniff_file, SniffedType};
#[allow(unused_imports)]
pub use taxonomy::{default_categories, default_ignore_patterns};
