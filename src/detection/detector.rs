use std::path::Path;

use anyhow::{Result, anyhow};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use imageproc::{contours::{find_contours_with_threshold, Contour}, geometry::{approximate_polygon_dp, arc_length}};
use usls::{models::YOLO, Config, Image, Y};
use rxing::BarcodeFormat;

const CONF_THRESHOLD: f32 = 0.5; // 对应 qrdet 的 conf_th
const NMS_IOU_THRESHOLD: f32 = 0.3; // 对应 qrdet 的 nms_iou

pub struct DetectionResult {
    pub quad_xy: [[f32; 2]; 4],
    // 从YOLO模型获得的原始 xyxy 边界框
    pub bbox_xyxy: [f32; 4],
}

pub trait Detector {
    fn detect(&mut self, data: Image) -> Vec<DetectionResult>;
    fn get_barcode_format(&self) -> BarcodeFormat;
}

pub struct YoloQrDetector {
    model: YOLO,
}

impl YoloQrDetector {
    pub fn new(model_path: &Path) -> Self {
        let config = Config::yolo()
            .with_model_file(&model_path.to_string_lossy())
            .with_version(8.try_into().expect("Invalid YOLO version"))
            .with_task("segment".parse().expect("Invalid task"))
            .with_scale("s".parse().expect("Invalid sacle"))
            .with_class_confs(&[CONF_THRESHOLD])
            .with_nc(1)
            .with_class_names(&["qr_code"])
            .with_iou(NMS_IOU_THRESHOLD)
            .with_topk(10);
        let model = YOLO::new(config.commit().expect("Failed to commit config")).unwrap();
        YoloQrDetector { model }
    }
}

pub fn y_to_detection_results(y: Y) -> Result<Vec<DetectionResult>> {
    // 1. 从Y对象中获取掩码和边界框的 slice
    let masks = y.masks().ok_or_else(|| anyhow!("Y object does not contain Masks"))?;
    let bboxes = y.hbbs().ok_or_else(|| anyhow!("Y object does not contain Bboxes (via hbbs method)"))?;

    if masks.len() != bboxes.len() {
        return Err(anyhow!(
            "Mismatch between number of masks ({}) and bounding boxes ({}).",
            masks.len(),
            bboxes.len()
        ));
    }

    // 2. 并行处理每一个检测到的物体 (Mask 和 Bbox)
    let results: Vec<DetectionResult> = bboxes
        .par_iter()
        .zip(masks.par_iter())
        .filter_map(|(hbb, mask_obj)| {
            // 对每个 mask 查找轮廓并提取角点
            let mask_image = mask_obj.mask();
            let contours: Vec<Contour<i32>> = find_contours_with_threshold(mask_image, 0);

            let qr_contour = contours.iter().max_by_key(|c| c.points.len())?;

            let perimeter = arc_length(&qr_contour.points, true);
            let mut final_corners = None;
            for i in 1..=10 { // try to tune from 1% to 10% of the perimeter
                let epsilon = (i as f64) * 0.01 * perimeter;
                let corners = approximate_polygon_dp(&qr_contour.points, epsilon, true);
                if corners.len() == 4 {
                    final_corners = Some(corners);
                    break;
                }
                if corners.len() < 4 && i > 2 { // i > 2 是为了防止过早退出
                    break;
                }
            }

            // 如果成功简化为四边形，则处理；否则忽略此检测结果
            if let Some(corners) = final_corners {
                let mut sorted_corners = corners;

                // 第一次排序
                sorted_corners.sort_by_key(|p| p.y + p.x);
                let top_left = sorted_corners[0];
                let bottom_right = sorted_corners[3];
            
                // 第二次排序
                sorted_corners.sort_by_key(|p| p.y - p.x);
                let bottom_left = sorted_corners[0];
                let top_right = sorted_corners[3];
            
                // 重新按期望顺序组装
                let ordered_corners = [top_left, top_right, bottom_right, bottom_left];
                
                let quad_xy: [[f32; 2]; 4] = std::array::from_fn(|i| {
                    [ordered_corners[i].x as f32, ordered_corners[i].y as f32]
                });
                let bbox_xyxy = [hbb.x(), hbb.y(), hbb.x() + hbb.w(), hbb.y() + hbb.h()];

                Some(DetectionResult {
                    quad_xy: quad_xy,
                    bbox_xyxy: bbox_xyxy,
                })
            } else {
                None // 忽略不是四边形的轮廓
            }
        })
        .collect();

    Ok(results)
}

impl Detector for YoloQrDetector {
    fn detect(&mut self, data: Image) -> Vec<DetectionResult> {
        // fit the input data into the model
        let wrapped = [data];
        let results = self.model.forward(&wrapped).expect("Model forward failed");
        results.into_iter().map(|y| {
            y_to_detection_results(y).unwrap_or_else(|e| {
                eprintln!("Error converting Y to DetectionResults: {}", e);
                vec![]
            })
        }).flatten().collect()
    }

    fn get_barcode_format(&self) -> BarcodeFormat {
        BarcodeFormat::QR_CODE
    }
}


#[cfg(test)]
mod tests {
    use usls::{Annotator, DataLoader, Style, SKELETON_COCO_19, SKELETON_COLOR_COCO_19};

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_model_forward() {
        let model_path = PathBuf::from("/Users/wangjiajie/software/rxing/assets/qrdet-s.onnx");
        let annotator = Annotator::default()
        .with_obb_style(Style::obb().with_draw_fill(true))
        .with_hbb_style(
            Style::hbb()
                .with_draw_fill(true)
                .with_palette(&usls::Color::palette_coco_80()),
        )
        .with_keypoint_style(
            Style::keypoint()
                .with_skeleton((SKELETON_COCO_19, SKELETON_COLOR_COCO_19).into())
                .show_confidence(false)
                .show_id(true)
                .show_name(false),
        )
        .with_mask_style(Style::mask().with_draw_mask_polygon_largest(true).with_draw_mask_hbbs(true));
        let config = Config::yolo()
            .with_model_file(&model_path.to_string_lossy())
            .with_version(8.try_into().expect("Invalid YOLO version"))
            .with_task("segment".parse().expect("Invalid task"))
            .with_scale("s".parse().expect("Invalid sacle"))
            .with_class_confs(&[CONF_THRESHOLD])
            .with_nc(1)
            .with_class_names(&["qr_code"])
            .with_iou(NMS_IOU_THRESHOLD)
            .with_topk(10);
        let mut model = YOLO::new(config.commit().expect("Failed to commit config")).unwrap();
        let image_path = "/Users/wangjiajie/software/rxing/assets/qr_entity.png";
        let dataloader = DataLoader::new(image_path).unwrap().with_batch(1).build().unwrap();
        for image in &dataloader {
            // forward() 包含了预处理、推理和后处理的完整流程
            let results = model.forward(&image).unwrap();
            assert_eq!(results.len(), 1);
            // assert_eq!(results[0].hbbs().unwrap().len(), 3);
            let mut count = 0;
            for (x, y) in image.iter().zip(results.iter()) {
                println!("Detected objects: {:?}", y);
                annotator.annotate(x, y).expect("annotate failed").save(format!("/Users/wangjiajie/software/rxing/assets/qr_entity_{}.jpg", count)).unwrap();
                count += 1;
            }
        }
    }

    #[test]
    fn test_detector() {
        let model_path = PathBuf::from("/Users/wangjiajie/software/rxing/assets/qrdet-s.onnx");
        let mut detector = YoloQrDetector::new(&model_path);
        let image_path = "/Users/wangjiajie/software/rxing/assets/qr_entity.png";
        let images = Image::try_read(image_path)
            .expect("Failed to read image");
        let results = detector.detect(images);
        assert!(!results.is_empty(), "No detection results found");
        for result in &results {
            println!("Detected quad: {:?}", result.quad_xy);
            println!("Detected bbox: {:?}", result.bbox_xyxy);
        }
    }
}