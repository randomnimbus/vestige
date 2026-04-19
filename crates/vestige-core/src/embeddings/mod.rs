//! Semantic Embeddings Module
//!
//! Provides local embedding generation using fastembed v5.13.
//! No external API calls required — 100% local and private.
//!
//! Supports:
//! - Dual backend: Nomic Embed v1.5 (ONNX, default, 768d native → 256d
//!   Matryoshka) or Qwen3-Embedding-0.6B (Candle, `qwen3-embed` feature,
//!   1024d native, 32K context).
//! - Cosine similarity computation.
//! - Batch embedding for efficiency.
//! - Hybrid multi-model fusion (future).

mod code;
mod hybrid;
mod local;

pub(crate) use local::get_cache_dir;
pub use local::{
    BATCH_SIZE, EMBEDDING_DIMENSIONS, Embedding, EmbeddingError, EmbeddingService, MAX_TEXT_LENGTH,
    NOMIC_EMBEDDING_DIMENSIONS, QWEN3_EMBEDDING_DIMENSIONS, QWEN3_QUERY_INSTRUCTION,
    cosine_similarity, dot_product, euclidean_distance, matryoshka_truncate, qwen3_format_query,
};

pub use code::CodeEmbedding;
pub use hybrid::HybridEmbedding;
