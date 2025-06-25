use anyhow::{anyhow, Context, Result};
use directories::ProjectDirs;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const PRETRAINED_MODEL_URL: &str = "https://github.com/xiaoniaoyouhuajiang/rxing/releases/download/v0.1.1/qrdet-s.onnx";
const MODEL_NAME: &str = "qrdet-s.onnx";
const MODEL_CHECK_SUM: &str = "8e222ebdacd50dd0c6f3a8bb2b22ead6c2017a2e33627e974fc413b442a1eafd";

pub fn get_asset_path(asset_name: &str) -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut path = std::path::PathBuf::from(manifest_dir);
    path.push("assets");
    path.push(asset_name);
    path
}

pub fn get_or_download_model_path() -> Result<PathBuf> {
    // 1. 获取跨平台的项目缓存目录
    // "rs.your_org.your_project" 是一个推荐的命名方式，确保唯一性
    if let Some(proj_dirs) = ProjectDirs::from("rs", "github", "rxing-detection") {
        let cache_dir = proj_dirs.cache_dir();
        fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;

        let model_path = cache_dir.join(MODEL_NAME);

        // 2. 检查文件是否存在
        if model_path.exists() {
            println!("Model file found at: {:?}", model_path);
            // 3. 如果存在，校验SHA256
            if validate_checksum(&model_path, MODEL_CHECK_SUM)? {
                println!("Checksum is valid. Using cached model.");
                return Ok(model_path);
            } else {
                println!("Checksum mismatch. Re-downloading the model...");
                fs::remove_file(&model_path).context("Failed to remove corrupted model file")?;
            }
        }

        // 4. 如果文件不存在或校验失败，则下载
        println!("Downloading model from {}...", PRETRAINED_MODEL_URL);
        download_file(PRETRAINED_MODEL_URL, &model_path)?;
        println!("Download complete.");

        // 5. 再次校验下载好的文件
        if validate_checksum(&model_path, MODEL_CHECK_SUM)? {
            println!("Downloaded file checksum is valid.");
            Ok(model_path)
        } else {
            Err(anyhow!("Downloaded file is corrupted. Checksum mismatch."))
        }
    } else {
        Err(anyhow!("Could not determine a valid cache directory."))
    }
}

/// 从URL下载文件到指定路径
fn download_file(url: &str, path: &Path) -> Result<()> {
    let response = reqwest::blocking::get(url)
        .context("Failed to send download request")?
        .error_for_status()
        .context("Server returned an error status")?;

    let bytes = response.bytes()?.to_vec();
    fs::write(path, &bytes).context(format!("Failed to write downloaded data to {:?}", path))?;
    Ok(())
}

/// 验证文件的SHA256哈希值
fn validate_checksum(path: &Path, expected_hash_hex: &str) -> Result<bool> {
    let mut file = fs::File::open(path).context("Failed to open model file for validation")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let calculated_hash = hasher.finalize();
    let calculated_hash_hex = hex::encode(calculated_hash);

    Ok(calculated_hash_hex.eq_ignore_ascii_case(expected_hash_hex))
}