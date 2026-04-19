//! Local Semantic Embeddings
//!
//! Uses fastembed v5.13 for local inference.
//!
//! ## Models
//!
//! - **Default (nomic)**: Nomic Embed Text v1.5 (ONNX via `TextEmbedding`, 768d → 256d
//!   Matryoshka, 8192 token context). Single binary, no GPU required.
//! - **Optional (qwen3)**: Qwen3 Embedding 0.6B (Candle via `Qwen3TextEmbedding`, 1024d,
//!   32K token context). Enable with `qwen3-embed` feature flag; combine with `metal`
//!   for Apple Silicon GPU acceleration. Asymmetric: queries MUST use the Instruct
//!   prefix via [`qwen3_format_query`], documents get no prefix.
//!
//! ## Dual-backend architecture
//!
//! The `Backend` enum routes to either fastembed's ONNX `TextEmbedding` path
//! (nomic, default) or the standalone Candle `Qwen3TextEmbedding` path
//! (Qwen3, feature-gated). Both are held behind the same global `OnceLock<Mutex<...>>`
//! so the rest of Vestige calls `EmbeddingService::embed()` unchanged.

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::{Mutex, OnceLock};

// ============================================================================
// CONSTANTS
// ============================================================================

/// Nomic Embed Text v1.5 output dimensions after Matryoshka truncation.
/// Truncated from 768 → 256 for 3x storage savings with only ~2% quality loss
/// (Matryoshka Representation Learning — the first N dims ARE the N-dim representation).
pub const NOMIC_EMBEDDING_DIMENSIONS: usize = 256;

/// Qwen3-Embedding-0.6B native output dimensions.
/// Supports Matryoshka truncation to 32-1024; we keep the full 1024 by default
/// so the ~+8-9 point MTEB retrieval lift over the nomic baseline isn't eroded
/// by truncation (Qwen3-Embedding-0.6B scores 61.83 on MTEB-Eng-v2 retrieval
/// per its HF model card; nomic-v1.5 is around 52-53 on the comparable bench —
/// cross-version so delta is ballpark, not precision).
/// Index storage cost at int8 = 1 KB/vec vs nomic's 256 B/vec (4x).
pub const QWEN3_EMBEDDING_DIMENSIONS: usize = 1024;

/// Back-compat alias: default embedding dimensions used by downstream code that
/// predates the dual-backend split. Always returns the active backend's native
/// dimension count. Callers that need a specific backend's dim should use the
/// explicit constants above.
pub const EMBEDDING_DIMENSIONS: usize = {
    #[cfg(feature = "qwen3-embed")]
    {
        QWEN3_EMBEDDING_DIMENSIONS
    }
    #[cfg(not(feature = "qwen3-embed"))]
    {
        NOMIC_EMBEDDING_DIMENSIONS
    }
};

/// Maximum text length for embedding (truncated if longer).
/// Nomic caps at 8K; Qwen3 allows 32K. Use the active backend's limit.
pub const MAX_TEXT_LENGTH: usize = {
    #[cfg(feature = "qwen3-embed")]
    {
        32_768
    }
    #[cfg(not(feature = "qwen3-embed"))]
    {
        8192
    }
};

/// Batch size for efficient embedding generation
pub const BATCH_SIZE: usize = 32;

/// Qwen3 instruct prefix template for retrieval queries.
/// Qwen3-Embedding is asymmetric: queries get the instruct-wrapped format,
/// documents are embedded raw. Missing this drops retrieval NDCG by ~3 points
/// per the Qwen3 model card.
pub const QWEN3_QUERY_INSTRUCTION: &str =
    "Given a web search query, retrieve relevant passages that answer the query";

/// Format a query string with the Qwen3 instruct prefix. No-op under the nomic
/// backend (nomic is symmetric — query and document embeddings share an embedding
/// function). Call this on QUERY text at search time, never on document text
/// at ingest time.
///
/// The exact template (no space after `Query:`) matches the canonical
/// `get_detailed_instruct` function in the Qwen3-Embedding-0.6B model card —
/// the TEI docs happen to include a space but the Python reference function
/// does not, so we match the Python reference.
#[inline]
pub fn qwen3_format_query(query: &str) -> String {
    #[cfg(feature = "qwen3-embed")]
    {
        format!(
            "Instruct: {instruction}\nQuery:{query}",
            instruction = QWEN3_QUERY_INSTRUCTION,
            query = query,
        )
    }
    #[cfg(not(feature = "qwen3-embed"))]
    {
        query.to_string()
    }
}

// ============================================================================
// BACKEND ENUM
// ============================================================================

/// Internal embedding backend. Held inside a `Mutex` to serialise access
/// across callers — the Nomic ONNX path needs `&mut self` for `embed`, and
/// the Qwen3 Candle path takes `&self` but the `Mutex` still keeps the API
/// uniform and gives us a single mutation-safe story under both cfgs without
/// a `Backend`-specific lock type.
///
/// The enum is private — callers go through `EmbeddingService` which hides
/// the branch. This lets us add new backends (e.g. ONNX Qwen3 for lower memory,
/// binary-quantized variants) without breaking downstream code.
enum Backend {
    /// fastembed ONNX path — Nomic Embed Text v1.5 (768d → 256d Matryoshka).
    ///
    /// This variant is constructed only under the default (non-Qwen3) build.
    /// When `qwen3-embed` is enabled `init_backend` selects [`Self::Qwen3`]
    /// exclusively, so the Nomic variant is dead code under that cfg. The
    /// match arms on this enum still handle both variants so the codebase
    /// can be audited feature-agnostically without `#[cfg]` noise.
    #[cfg_attr(feature = "qwen3-embed", allow(dead_code))]
    Nomic(TextEmbedding),
    /// fastembed Candle path — Qwen3-Embedding-0.6B (1024d, 32K context).
    #[cfg(feature = "qwen3-embed")]
    Qwen3(fastembed::Qwen3TextEmbedding),
}

impl Backend {
    /// Embed a batch of texts. The Nomic path truncates to Matryoshka dims
    /// internally; the Qwen3 path returns full-dim L2-normalized vectors.
    fn embed_batch(&mut self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        match self {
            Self::Nomic(model) => model
                .embed(texts, None)
                .map_err(|e| EmbeddingError::EmbeddingFailed(e.to_string())),
            #[cfg(feature = "qwen3-embed")]
            Self::Qwen3(model) => model
                .embed(&texts)
                .map_err(|e| EmbeddingError::EmbeddingFailed(e.to_string())),
        }
    }

    /// Post-process a raw embedding before handing it back to callers.
    /// Nomic: Matryoshka-truncate to 256d + L2-normalize.
    /// Qwen3: pass through (already last-token pooled and L2-normalized by the model).
    #[inline]
    fn post_process(&self, raw: Vec<f32>) -> Vec<f32> {
        match self {
            Self::Nomic(_) => matryoshka_truncate(raw),
            #[cfg(feature = "qwen3-embed")]
            Self::Qwen3(_) => raw,
        }
    }

    /// HuggingFace repo ID for this backend's model. Written to the
    /// `embedding_model` column on every knowledge-node row so dual-index
    /// search can route queries to the matching USearch index at retrieval time.
    fn model_name(&self) -> &'static str {
        match self {
            Self::Nomic(_) => "nomic-ai/nomic-embed-text-v1.5",
            #[cfg(feature = "qwen3-embed")]
            Self::Qwen3(_) => "Qwen/Qwen3-Embedding-0.6B",
        }
    }

    /// Output vector dimensions after post-processing.
    fn dimensions(&self) -> usize {
        match self {
            Self::Nomic(_) => NOMIC_EMBEDDING_DIMENSIONS,
            #[cfg(feature = "qwen3-embed")]
            Self::Qwen3(_) => QWEN3_EMBEDDING_DIMENSIONS,
        }
    }
}

// ============================================================================
// GLOBAL MODEL (with Mutex for fastembed's &mut self API)
// ============================================================================

/// Global backend, initialised on first use. Held as a `Mutex` because both
/// underlying embedding models require exclusive access for `embed()`.
static EMBEDDING_BACKEND: OnceLock<Result<Mutex<Backend>, String>> = OnceLock::new();

/// Get the default cache directory for fastembed models.
///
/// Resolution order:
/// 1. `FASTEMBED_CACHE_PATH` env var (explicit override)
/// 2. Platform cache dir via `directories::ProjectDirs`
///    - Linux:   `$XDG_CACHE_HOME/vestige/fastembed` (typically `~/.cache/vestige/fastembed`)
///    - macOS:   `~/Library/Caches/vestige/fastembed`
///    - Windows: `%LOCALAPPDATA%\vestige\cache\fastembed`
/// 3. `~/.cache/vestige/fastembed` (home-dir fallback)
/// 4. `.fastembed_cache` relative to CWD (absolute last resort, should never trigger)
pub(crate) fn get_cache_dir() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("FASTEMBED_CACHE_PATH") {
        return std::path::PathBuf::from(path);
    }

    // qualifier="" produces a clean app-name-only path on Linux/Windows;
    // on macOS the qualifier is used for the bundle ID so we keep it empty
    // to get ~/Library/Caches/vestige/fastembed rather than
    // ~/Library/Caches/com.vestige.vestige/fastembed.
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "vestige", "vestige") {
        return proj_dirs.cache_dir().join("fastembed");
    }

    // Fallback to home directory
    if let Some(base_dirs) = directories::BaseDirs::new() {
        return base_dirs.home_dir().join(".cache/vestige/fastembed");
    }

    // Last resort fallback (shouldn't happen in practice)
    std::path::PathBuf::from(".fastembed_cache")
}

/// Initialise the Nomic ONNX backend. Downloads the model on first use.
///
/// Called by [`init_backend`] only when `qwen3-embed` is NOT enabled.
/// Kept compiled under both cfgs so that a future runtime-selectable backend
/// can reuse it without a cross-feature refactor; silenced as dead code when
/// the Qwen3 feature is on.
#[cfg_attr(feature = "qwen3-embed", allow(dead_code))]
fn init_nomic(cache_dir: std::path::PathBuf) -> Result<Backend, String> {
    // nomic-embed-text-v1.5: 768 dimensions, 8192 token context, Matryoshka
    let options = InitOptions::new(EmbeddingModel::NomicEmbedTextV15)
        .with_show_download_progress(true)
        .with_cache_dir(cache_dir);

    TextEmbedding::try_new(options).map(Backend::Nomic).map_err(|e| {
        format!(
            "Failed to initialize nomic-embed-text-v1.5 embedding model: {}. \
             Ensure ONNX runtime is available and model files can be downloaded.",
            e
        )
    })
}

/// Initialise the Qwen3 Candle backend. Downloads ~1.2 GB model weights on first
/// use (same cache dir as the ONNX path). Uses Metal GPU on Apple Silicon when
/// `metal` feature is on; CPU otherwise. CUDA auto-selection is a Day-3+ follow
/// (candle-core 0.10 exposes `Device::new_cuda(0)` but we ship the CPU fallback
/// first to keep Linux users working out of the box).
#[cfg(feature = "qwen3-embed")]
fn init_qwen3(_cache_dir: std::path::PathBuf) -> Result<Backend, String> {
    // Device selection is caller-side in candle-core 0.10: fastembed does NOT
    // auto-select from its own `metal` / `cuda` feature. We gate on vestige-core's
    // `metal` feature and fall back to CPU if Metal init fails (e.g. x86 macOS
    // or a broken Apple Silicon Metal stack) so the feature flag is always safe
    // to combine with qwen3-embed.
    #[cfg(feature = "metal")]
    let device = candle_core::Device::new_metal(0).unwrap_or_else(|e| {
        tracing::warn!("Metal device init failed ({}); falling back to CPU", e);
        candle_core::Device::Cpu
    });
    #[cfg(not(feature = "metal"))]
    let device = candle_core::Device::Cpu;

    let dtype = candle_core::DType::F32;

    fastembed::Qwen3TextEmbedding::from_hf(
        "Qwen/Qwen3-Embedding-0.6B",
        &device,
        dtype,
        MAX_TEXT_LENGTH,
    )
    .map(Backend::Qwen3)
    .map_err(|e| {
        format!(
            "Failed to initialize Qwen3-Embedding-0.6B: {}. \
             First-run requires ~1.2 GB download to ~/.cache/vestige/fastembed; \
             subsequent runs load from cache.",
            e
        )
    })
}

/// Initialise the active backend based on compiled features. Qwen3 wins when
/// both features are enabled (it's strictly newer and more capable).
fn init_backend() -> Result<Backend, String> {
    let cache_dir = get_cache_dir();

    // Create cache directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        tracing::warn!("Failed to create cache directory {:?}: {}", cache_dir, e);
    }

    #[cfg(feature = "qwen3-embed")]
    {
        init_qwen3(cache_dir)
    }
    #[cfg(not(feature = "qwen3-embed"))]
    {
        init_nomic(cache_dir)
    }
}

/// Lock and return the global embedding backend. Initialises on first call.
fn get_backend() -> Result<std::sync::MutexGuard<'static, Backend>, EmbeddingError> {
    let result = EMBEDDING_BACKEND.get_or_init(|| init_backend().map(Mutex::new));

    match result {
        Ok(backend) => backend
            .lock()
            .map_err(|e| EmbeddingError::ModelInit(format!("Lock poisoned: {}", e))),
        Err(err) => Err(EmbeddingError::ModelInit(err.clone())),
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Embedding error types
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EmbeddingError {
    /// Failed to initialize the embedding model
    ModelInit(String),
    /// Failed to generate embedding
    EmbeddingFailed(String),
    /// Invalid input (empty, too long, etc.)
    InvalidInput(String),
}

impl std::fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingError::ModelInit(e) => write!(f, "Model initialization failed: {}", e),
            EmbeddingError::EmbeddingFailed(e) => write!(f, "Embedding generation failed: {}", e),
            EmbeddingError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl std::error::Error for EmbeddingError {}

// ============================================================================
// EMBEDDING TYPE
// ============================================================================

/// A semantic embedding vector
#[derive(Debug, Clone)]
pub struct Embedding {
    /// The embedding vector
    pub vector: Vec<f32>,
    /// Dimensions of the vector
    pub dimensions: usize,
}

impl Embedding {
    /// Create a new embedding from a vector
    pub fn new(vector: Vec<f32>) -> Self {
        let dimensions = vector.len();
        Self { vector, dimensions }
    }

    /// Compute cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.dimensions != other.dimensions {
            return 0.0;
        }
        cosine_similarity(&self.vector, &other.vector)
    }

    /// Compute Euclidean distance with another embedding
    pub fn euclidean_distance(&self, other: &Embedding) -> f32 {
        if self.dimensions != other.dimensions {
            return f32::MAX;
        }
        euclidean_distance(&self.vector, &other.vector)
    }

    /// Normalize the embedding vector to unit length
    pub fn normalize(&mut self) {
        let norm = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut self.vector {
                *x /= norm;
            }
        }
    }

    /// Check if the embedding is normalized (unit length)
    pub fn is_normalized(&self) -> bool {
        let norm = self.vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        (norm - 1.0).abs() < 0.001
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        self.vector.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if !bytes.len().is_multiple_of(4) {
            return None;
        }
        let vector: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        Some(Self::new(vector))
    }
}

// ============================================================================
// EMBEDDING SERVICE
// ============================================================================

/// Service for generating and managing embeddings
pub struct EmbeddingService {
    _unused: (),
}

impl Default for EmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new() -> Self {
        Self { _unused: () }
    }

    /// Check if the model is ready
    pub fn is_ready(&self) -> bool {
        match get_backend() {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Embedding model not ready: {}", e);
                false
            }
        }
    }

    /// Check if the model is ready and return the error if not
    pub fn check_ready(&self) -> Result<(), EmbeddingError> {
        get_backend().map(|_| ())
    }

    /// Initialize the model (downloads if necessary)
    pub fn init(&self) -> Result<(), EmbeddingError> {
        let _model = get_backend()?; // Ensures model is loaded and returns any init errors
        Ok(())
    }

    /// HuggingFace repo ID of the active backend. Used by storage to tag every
    /// embedded row with its source model so dual-index search can route at
    /// retrieval time without re-embedding the query against every index.
    pub fn model_name(&self) -> &'static str {
        // Acquire the lock only to read a const — cheap, and avoids duplicating
        // the cfg branch at the call site.
        match get_backend() {
            Ok(guard) => guard.model_name(),
            #[cfg(feature = "qwen3-embed")]
            Err(_) => "Qwen/Qwen3-Embedding-0.6B",
            #[cfg(not(feature = "qwen3-embed"))]
            Err(_) => "nomic-ai/nomic-embed-text-v1.5",
        }
    }

    /// Output vector dimensions for the active backend.
    pub fn dimensions(&self) -> usize {
        match get_backend() {
            Ok(guard) => guard.dimensions(),
            Err(_) => EMBEDDING_DIMENSIONS,
        }
    }

    /// Generate embedding for a single text.
    ///
    /// Documents go in raw. For QUERY text under the Qwen3 backend, the caller
    /// is responsible for wrapping with [`qwen3_format_query`] before calling
    /// this method — the asymmetric query/document format is intentional and
    /// handled at the search layer, not the embedding layer.
    pub fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput(
                "Text cannot be empty".to_string(),
            ));
        }

        let mut backend = get_backend()?;

        // Truncate if too long (char-boundary safe)
        let text = if text.len() > MAX_TEXT_LENGTH {
            let mut end = MAX_TEXT_LENGTH;
            while !text.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            &text[..end]
        } else {
            text
        };

        let raw = backend.embed_batch(vec![text])?;

        // Shape contract: both backends must return at least one vector of
        // non-zero length for a non-empty input. An empty outer or inner vec
        // means the backend misbehaved (e.g. fastembed regression, malformed
        // ONNX output). Guard both so a silent zero-dim vector never lands in
        // the USearch index where it would later blow up with an opaque
        // InvalidDimensions error deep in the search path.
        let first = raw.into_iter().next().ok_or_else(|| {
            EmbeddingError::EmbeddingFailed("No embedding generated".to_string())
        })?;
        if first.is_empty() {
            return Err(EmbeddingError::EmbeddingFailed(
                "Backend returned an empty embedding vector".to_string(),
            ));
        }

        Ok(Embedding::new(backend.post_process(first)))
    }

    /// Generate embeddings for multiple texts (batch processing).
    ///
    /// As with [`Self::embed`], query/document asymmetry is the caller's
    /// responsibility: wrap query texts with [`qwen3_format_query`] upstream.
    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let mut backend = get_backend()?;
        let mut all_embeddings = Vec::with_capacity(texts.len());

        // Process in batches for efficiency
        for chunk in texts.chunks(BATCH_SIZE) {
            let truncated: Vec<&str> = chunk
                .iter()
                .map(|t| {
                    if t.len() > MAX_TEXT_LENGTH {
                        let mut end = MAX_TEXT_LENGTH;
                        while !t.is_char_boundary(end) && end > 0 {
                            end -= 1;
                        }
                        &t[..end]
                    } else {
                        *t
                    }
                })
                .collect();

            let raw = backend.embed_batch(truncated)?;

            for emb in raw {
                all_embeddings.push(Embedding::new(backend.post_process(emb)));
            }
        }

        Ok(all_embeddings)
    }

    /// Find most similar embeddings to a query
    pub fn find_similar(
        &self,
        query_embedding: &Embedding,
        candidate_embeddings: &[Embedding],
        top_k: usize,
    ) -> Vec<(usize, f32)> {
        let mut similarities: Vec<(usize, f32)> = candidate_embeddings
            .iter()
            .enumerate()
            .map(|(i, emb)| (i, query_embedding.cosine_similarity(emb)))
            .collect();

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        similarities.into_iter().take(top_k).collect()
    }
}

// ============================================================================
// SIMILARITY FUNCTIONS
// ============================================================================

/// Apply Matryoshka truncation: truncate to [`NOMIC_EMBEDDING_DIMENSIONS`] and L2-normalize.
///
/// Nomic Embed v1.5 supports Matryoshka Representation Learning,
/// meaning the first N dimensions of the 768-dim output ARE a valid
/// N-dimensional embedding with minimal quality loss (~2% on MTEB for 256-dim).
///
/// Not applied to the Qwen3 backend — Qwen3 output is already last-token pooled
/// and L2-normalized by the Candle model internals, and we keep full 1024-dim
/// by default so the retrieval quality gain over nomic isn't Matryoshka-capped.
#[inline]
pub fn matryoshka_truncate(mut vector: Vec<f32>) -> Vec<f32> {
    if vector.len() > NOMIC_EMBEDDING_DIMENSIONS {
        vector.truncate(NOMIC_EMBEDDING_DIMENSIONS);
    }
    // L2-normalize the truncated vector
    let norm = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut vector {
            *x /= norm;
        }
    }
    vector
}

/// Compute cosine similarity between two vectors
#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let mut dot_product = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot_product += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denominator = (norm_a * norm_b).sqrt();
    if denominator > 0.0 {
        dot_product / denominator
    } else {
        0.0
    }
}

/// Compute Euclidean distance between two vectors
#[inline]
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

/// Compute dot product between two vectors
#[inline]
pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 0.0001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_euclidean_distance_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let dist = euclidean_distance(&a, &b);
        assert!(dist.abs() < 0.0001);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let dist = euclidean_distance(&a, &b);
        assert!((dist - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_embedding_to_from_bytes() {
        let original = Embedding::new(vec![1.5, 2.5, 3.5, 4.5]);
        let bytes = original.to_bytes();
        let restored = Embedding::from_bytes(&bytes).unwrap();

        assert_eq!(original.vector.len(), restored.vector.len());
        for (a, b) in original.vector.iter().zip(restored.vector.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[test]
    fn test_embedding_normalize() {
        let mut emb = Embedding::new(vec![3.0, 4.0]);
        emb.normalize();

        // Should be unit length
        assert!(emb.is_normalized());

        // Components should be 0.6 and 0.8 (3/5 and 4/5)
        assert!((emb.vector[0] - 0.6).abs() < 0.0001);
        assert!((emb.vector[1] - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_find_similar() {
        let service = EmbeddingService::new();

        let query = Embedding::new(vec![1.0, 0.0, 0.0]);
        let candidates = vec![
            Embedding::new(vec![1.0, 0.0, 0.0]),  // Most similar
            Embedding::new(vec![0.7, 0.7, 0.0]),  // Somewhat similar
            Embedding::new(vec![0.0, 1.0, 0.0]),  // Orthogonal
            Embedding::new(vec![-1.0, 0.0, 0.0]), // Opposite
        ];

        let results = service.find_similar(&query, &candidates, 2);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // First candidate should be most similar
        assert!((results[0].1 - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_qwen3_format_query_feature_gated() {
        let wrapped = qwen3_format_query("cats are cute");

        #[cfg(feature = "qwen3-embed")]
        {
            // With Qwen3 active, queries get wrapped in the Instruct template.
            // No space between `Query:` and the user text — this matches the
            // canonical `get_detailed_instruct` function in the Qwen3 model
            // card's Python example, even though the TEI curl example has a
            // space. We follow the Python reference.
            assert!(wrapped.starts_with("Instruct: "));
            assert!(wrapped.ends_with("\nQuery:cats are cute"));
        }
        #[cfg(not(feature = "qwen3-embed"))]
        {
            // Under the nomic backend the wrapper is a no-op.
            assert_eq!(wrapped, "cats are cute");
        }
    }

    #[test]
    fn test_backend_dimensions_match_feature_flag() {
        #[cfg(feature = "qwen3-embed")]
        assert_eq!(EMBEDDING_DIMENSIONS, QWEN3_EMBEDDING_DIMENSIONS);
        #[cfg(not(feature = "qwen3-embed"))]
        assert_eq!(EMBEDDING_DIMENSIONS, NOMIC_EMBEDDING_DIMENSIONS);
    }

    /// Integration: load the Qwen3 backend and verify it produces a 1024-dim
    /// L2-normalized vector on CPU. Ignored by default because it downloads
    /// ~1.2 GB of model weights on first run.
    ///
    /// Run with: `cargo test --features qwen3-embed -- --ignored test_qwen3_embed_live`
    #[cfg(feature = "qwen3-embed")]
    #[test]
    #[ignore]
    fn test_qwen3_embed_live() {
        let service = EmbeddingService::new();
        service.init().expect("Qwen3 backend init");

        let emb = service.embed("hello world").expect("embed succeeds");
        assert_eq!(emb.dimensions, QWEN3_EMBEDDING_DIMENSIONS);
        assert!(emb.is_normalized(), "Qwen3 output must be L2-normalized");
    }
}
