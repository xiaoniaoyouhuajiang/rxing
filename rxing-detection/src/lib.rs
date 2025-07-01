pub mod detection;

// Re-export the public API for easier consumption by other crates.
pub use detection::detector::{Detector, DetectionResult, YoloQrDetector};
pub use detection::preprocess::enhance_and_decode_qr;
pub use detection::resource::get_or_download_model_path;
