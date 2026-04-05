// Seed-Intelligence Library
// Hebbian learning based embodied intelligence system

pub mod brain;
pub mod crawler;
pub mod inference;
pub mod learner;
pub mod nlp;
pub mod lm;
pub mod file_reader;
pub mod response_generator;

// Re-export main types for convenience
pub use brain::{Brain, Concept, Relation};
pub use learner::{IncrementalLearner, LearnResult, Stats};
pub use inference::{query, ask, Answer};
pub use nlp::{TokenizerWrapper, simple_tokenize, filter_stop_words};
pub use lm::{CausalLM, Trainer, Vocab};
pub use file_reader::{read_file, stream_read_file, is_ebook_format, is_pdf_format};
pub use response_generator::{
    generate_single_concept_answer,
    generate_multi_concept_answer,
    infer_relation_type,
    relation_to_sentence,
};
