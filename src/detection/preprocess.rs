use anyhow::{Result, anyhow};
use image::{DynamicImage, GenericImageView, GrayImage, Luma, Rgb, RgbImage};
use imageproc::{
    contrast::{self, ThresholdType}, filter,
    geometric_transformations::{warp, Interpolation, Projection},
};
use std::convert::TryInto;


pub fn enhance_and_decode_qr(
    full_image: &DynamicImage,
    detection: &super::detector::DetectionResult,
    decoder: impl Fn(&DynamicImage) -> Option<String>,
) -> Option<String> {
    let [x_min, y_min, x_max, y_max] = detection.bbox_xyxy;
    let cropped_bbox = full_image.crop_imm(
        x_min as u32,
        y_min as u32,
        (x_max - x_min) as u32,
        (y_max - y_min) as u32,
    );

    // 1. 透视校正 (Perspective Correction)
    let corrected_perspective = match correct_perspective(full_image.to_rgb8(), detection.quad_xy) {
        Ok(img) => img,
        Err(_) => return None, // 如果校正失败，则提前退出
    };

    let corrections = [
        ("cropped_bbox", cropped_bbox),
        ("corrected_perspective", corrected_perspective.into()),
    ];

    // 2. 迭代增强循环 (Enhancement Loop)
    for (_label, base_image) in &corrections {
        for scale_factor in [1.0, 0.5, 2.0, 0.25, 3.0, 4.0] {
            let (base_w, base_h) = base_image.dimensions();
            let (new_w, new_h) = (
                (base_w as f32 * scale_factor) as u32,
                (base_h as f32 * scale_factor) as u32,
            );

            if new_w < 25 || new_h < 25 || new_w > 1024 || new_h > 1024 {
                continue;
            }
            
            let resized_image = image::imageops::resize(
                base_image,
                new_w,
                new_h,
                image::imageops::FilterType::Triangle,
            );

            // a) 直接解码
            if let Some(decoded) = decoder(&resized_image.clone().into()) {
                return Some(decoded);
            }

            // b) 颜色反转
            let mut inverted_image = resized_image.clone();
            image::imageops::invert(&mut inverted_image);
            if let Some(decoded) = decoder(&inverted_image.into()) {
                return Some(decoded);
            }

            // c) 灰度图 + 高级增强
            let gray_image = image::imageops::grayscale(&resized_image);
            if let Some(decoded) = try_advanced_decodings(&gray_image, &decoder) {
                return Some(decoded);
            }
        }
    }

    None
}

/// 对灰度图应用高级解码策略（Otsu, Blur, Sharpen）
fn try_advanced_decodings(
    gray_image: &GrayImage,
    decoder: &impl Fn(&DynamicImage) -> Option<String>,
) -> Option<String> {
    // a) 直接解码灰度图
    if let Some(decoded) = decoder(&gray_image.clone().into()) {
        return Some(decoded);
    }

    // b) Otsu自适应二值化  (usls的DB和YOLOP模型后处理都用到了imageproc的轮廓查找，证明其可用)
    // `otsu_level` 计算阈值，`threshold` 应用阈值
    let otsu_threshold = contrast::otsu_level(gray_image);
    let binary_image = contrast::threshold(gray_image, otsu_threshold, ThresholdType::Binary);
    if let Some(decoded) = decoder(&binary_image.into()) {
        return Some(decoded);
    }
    
    // c) 模糊后解码 (对应 qreader 中的 blur_kernel_sizes)
    for sigma in [1.5, 2.5] { // 约对应 (5,5) 和 (7,7) 核
        let blurred = filter::gaussian_blur_f32(gray_image, sigma);
        if let Some(decoded) = decoder(&image::DynamicImage::ImageLuma8(blurred)) {
            return Some(decoded);
        }
    }

    // d) 锐化后解码
    let sharpen_kernel = [-1.0f32, -1.0, -1.0, -1.0, 9.0, -1.0, -1.0, -1.0, -1.0];
    let sharpened: GrayImage = filter::filter3x3::<Luma<u8>, f32, _>(gray_image, &sharpen_kernel);
    if let Some(decoded) = decoder(&sharpened.into()) {
        return Some(decoded);
    }

    None
}


fn correct_perspective(
    image: RgbImage,
    src_pts_f32: [[f32; 2]; 4],
) -> Result<DynamicImage> {
    let src_pts: [[f64; 2]; 4] = src_pts_f32.map(|p| [p[0] as f64, p[1] as f64]);

    let width1 = ((src_pts[0][0] - src_pts[1][0]).powi(2) + (src_pts[0][1] - src_pts[1][1]).powi(2)).sqrt();
    let width2 = ((src_pts[2][0] - src_pts[3][0]).powi(2) + (src_pts[2][1] - src_pts[3][1]).powi(2)).sqrt();
    let height1 = ((src_pts[0][0] - src_pts[3][0]).powi(2) + (src_pts[0][1] - src_pts[3][1]).powi(2)).sqrt();
    let height2 = ((src_pts[1][0] - src_pts[2][0]).powi(2) + (src_pts[1][1] - src_pts[2][1]).powi(2)).sqrt();
    let max_dim = width1.max(width2).max(height1).max(height2).ceil() as u32;

    if max_dim == 0 {
        return Err(anyhow!("Invalid quadrilateral with zero size"));
    }

    let dst_pts: [[f64; 2]; 4] = [
        [0.0, 0.0],
        [max_dim as f64 - 1.0, 0.0],
        [max_dim as f64 - 1.0, max_dim as f64 - 1.0],
        [0.0, max_dim as f64 - 1.0],
    ];

    let mut a = faer::Mat::<f64>::zeros(8, 9);
    for i in 0..4 {
        let (sx, sy) = (src_pts[i][0], src_pts[i][1]);
        let (dx, dy) = (dst_pts[i][0], dst_pts[i][1]);
        unsafe {
            a.write_unchecked(2 * i, 0, sx); 
            a.write_unchecked(2 * i, 1, sy); 
            a.write_unchecked(2 * i, 2, 1.0);
            a.write_unchecked(2 * i, 6, -dx * sx); 
            a.write_unchecked(2 * i, 7, -dx * sy); 
            a.write_unchecked(2 * i, 8, -dx);
            a.write_unchecked(2 * i + 1, 3, sx); 
            a.write_unchecked(2 * i + 1, 4, sy); 
            a.write_unchecked(2 * i + 1, 5, 1.0);
            a.write_unchecked(2 * i + 1, 6, -dy * sx); 
            a.write_unchecked(2 * i + 1, 7, -dy * sy); 
            a.write_unchecked(2 * i + 1, 8, -dy);
        }
    }
    
    let svd = a.svd();
    let h_col = svd.v().col(8);
    let h: [f32; 9] = h_col.iter().map(|&v| v as f32).collect::<Vec<_>>().try_into().unwrap();
    let proj = Projection::from_matrix(h).unwrap();

    let corrected_image = warp(
        &image,
        &proj,
        Interpolation::Bilinear,
        Rgb([0, 0, 0]),
    );
    let cropped = image::imageops::crop_imm(&corrected_image, 0, 0, max_dim, max_dim).to_image();

    Ok(cropped.into())
}

#[allow(unused_imports)]
mod test{
    use std::path::PathBuf;

    use crate::detection::detector::{Detector, YoloQrDetector};

    use super::*;
    use image::open;
    use rxing::{common::HybridBinarizer, BufferedImageLuminanceSource, Reader};
    use usls::Image;

    #[test]
    fn test_correct_perspective() {
        let model_path: PathBuf = PathBuf::from("/Users/wangjiajie/software/rxing/assets/qrdet-s.onnx");
        let mut detector = YoloQrDetector::new(&model_path);
        let image_path = "/Users/wangjiajie/software/rxing/assets/hard_qr.jpeg";
        let images = Image::try_read(image_path)
            .expect("Failed to read image");
        let image = images.to_rgb8();
        let results = detector.detect(images);
        assert!(!results.is_empty(), "No detection results found");
        let corrected = correct_perspective(image, results[0].quad_xy)
            .expect("Failed to correct perspective");
        corrected.save("/Users/wangjiajie/software/rxing/assets/corrected_perspective.png").expect("Failed to save corrected image");
    }

    #[test]
    fn test_fake_qr_pipeline() {
        let model_path: PathBuf = PathBuf::from("/Users/wangjiajie/software/rxing/assets/qrdet-s.onnx");
        let mut detector = YoloQrDetector::new(&model_path);
        let image_path = "/Users/wangjiajie/software/rxing/assets/fake_qr.jpeg";
        let images = Image::try_read(image_path)
            .expect("Failed to read image");
        let image = images.to_rgb8();
        let results = detector.detect(images);
        assert!(!results.is_empty(), "No detection results found");

        let start_time = std::time::Instant::now();
        let decoded = enhance_and_decode_qr(&image.into(), &results[0], |img: &DynamicImage| {
            let luma_source = BufferedImageLuminanceSource::new(img.clone());
            let binarizer = HybridBinarizer::new(luma_source);
            let mut binary_bitmap = rxing::BinaryBitmap::new(binarizer);
            let mut reader = rxing::qrcode::QRCodeReader::new();
            reader.decode(&mut binary_bitmap).ok().map(|result| result.getText().to_string())
        });
        println!("Time taken: {:?}", start_time.elapsed());
        // assert!(decoded.is_some(), "Failed to decode QR code");
        println!("Decoded QR code: {:?}", decoded);
    }
}
