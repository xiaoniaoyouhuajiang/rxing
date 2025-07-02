use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rxing::{
    common::HybridBinarizer, BarcodeFormat, BinaryBitmap, BufferedImageLuminanceSource,
    DecodeHints as RxingDecodeHints, EncodeHints as RxingEncodeHints, Luma8LuminanceSource,
    MultiFormatReader, MultiFormatWriter, RXingResult as InnerRXingResult, Reader, Writer,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// Conditional imports for the 'detection' feature
#[cfg(feature = "detection")]
use {
    image::DynamicImage,
    rxing_detection::{
        Detector, YoloQrDetector,
        enhance_and_decode_qr, get_or_download_model_path,
    },
    usls::Image as UslsImage,
};


#[pyclass(name = "RXingResult")]
#[derive(Clone)]
struct PyRXingResult {
    #[pyo3(get)]
    text: String,
    #[pyo3(get)]
    raw_bytes: Option<Vec<u8>>,
    #[pyo3(get)]
    num_bits: usize,
    #[pyo3(get)]
    result_points: Option<Vec<PyPoint>>,
    #[pyo3(get)]
    barcode_format: String,
    #[pyo3(get)]
    result_metadata: Option<HashMap<String, String>>,
    #[pyo3(get)]
    timestamp: u128,
}

impl From<InnerRXingResult> for PyRXingResult {
    fn from(res: InnerRXingResult) -> Self {
        PyRXingResult {
            text: res.getText().to_string(),
            raw_bytes: Some(res.getRawBytes().to_vec()),
            num_bits: res.getNumBits(),
            result_points: Some(
                res.getPoints()
                    .iter()
                    .map(|p| PyPoint { x: p.x, y: p.y })
                    .collect(),
            ),
            barcode_format: res.getBarcodeFormat().to_string(),
            result_metadata: Some(
                res.getRXingResultMetadata()
                    .iter()
                    .map(|(k, v)| (format!("{:?}", k), format!("{:?}", v)))
                    .collect(),
            ),
            timestamp: res.getTimestamp(),
        }
    }
}

#[pyclass(name = "Point")]
#[derive(Clone, Debug)]
struct PyPoint {
    #[pyo3(get, set)]
    x: f32,
    #[pyo3(get, set)]
    y: f32,
}

#[pyclass(name = "BitMatrix")]
#[derive(Clone)]
struct PyBitMatrix {
    #[pyo3(get)]
    width: u32,
    #[pyo3(get)]
    height: u32,
    inner_matrix: rxing::common::BitMatrix,
}

#[pymethods]
impl PyBitMatrix {
    #[getter]
    fn data(&self) -> Vec<Vec<bool>> {
        let mut data = Vec::with_capacity(self.height as usize);
        for y in 0..self.height {
            let mut row = Vec::with_capacity(self.width as usize);
            for x in 0..self.width {
                row.push(self.inner_matrix.get(x, y));
            }
            data.push(row);
        }
        data
    }
}

impl From<rxing::common::BitMatrix> for PyBitMatrix {
    fn from(bm: rxing::common::BitMatrix) -> Self {
        PyBitMatrix {
            width: bm.getWidth(),
            height: bm.getHeight(),
            inner_matrix: bm,
        }
    }
}

fn py_dict_to_decode_hints(
    _py: Python,
    dict_opt: Option<&Bound<PyDict>>,
) -> PyResult<RxingDecodeHints> {
    let mut hints = RxingDecodeHints::default();
    if let Some(dict) = dict_opt {
        for (key_any, value_any) in dict.iter() {
            let key_str: String = key_any.extract()?;
            match key_str.to_uppercase().as_str() {
                "TRY_HARDER" => hints.TryHarder = Some(value_any.extract()?),
                "PURE_BARCODE" => hints.PureBarcode = Some(value_any.extract()?),
                "POSSIBLE_FORMATS" => {
                    let formats_list: &Bound<PyList> = value_any.downcast().map_err(|_| {
                        PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                            "POSSIBLE_FORMATS value must be a list of strings",
                        )
                    })?;
                    let mut possible_formats = HashSet::new();
                    for format_any in formats_list.iter() {
                        let format_str: String = format_any.extract()?;
                        let format = BarcodeFormat::from(format_str.to_uppercase());
                        possible_formats.insert(format);
                    }
                    if !possible_formats.is_empty() {
                        hints.PossibleFormats = Some(possible_formats);
                    }
                }
                "CHARACTER_SET" => hints.CharacterSet = Some(value_any.extract()?),
                "ALSO_INVERTED" => hints.AlsoInverted = Some(value_any.extract()?),
                _ => {
                    eprintln!("Warning: Unknown decode hint: {}", key_str);
                }
            }
        }
    }
    Ok(hints)
}

fn py_dict_to_encode_hints(
    _py: Python,
    dict_opt: Option<&Bound<PyDict>>,
) -> PyResult<RxingEncodeHints> {
    let mut hints = RxingEncodeHints::default();
    if let Some(dict) = dict_opt {
        for (key_any, value_any) in dict.iter() {
            let key_str: String = key_any.extract()?;
            match key_str.to_uppercase().as_str() {
                "ERROR_CORRECTION" => hints.ErrorCorrection = Some(value_any.extract()?),
                "CHARACTER_SET" => hints.CharacterSet = Some(value_any.extract()?),
                "MARGIN" => hints.Margin = Some(value_any.extract()?),
                "QR_VERSION" => hints.QrVersion = Some(value_any.extract()?),
                _ => {
                    eprintln!("Warning: Unknown encode hint: {}", key_str);
                }
            }
        }
    }
    Ok(hints)
}

#[pyfunction]
fn decode_luma_pixels(
    py: Python,
    luma_data: &[u8],
    width: u32,
    height: u32,
    hints_dict: Option<&Bound<PyDict>>,
) -> PyResult<PyRXingResult> {
    let pixels = luma_data.to_vec();
    if (width * height) as usize != pixels.len() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Pixel data length does not match width * height.",
        ));
    }

    let hints = py_dict_to_decode_hints(py, hints_dict)?;
    let luma_source = Luma8LuminanceSource::new(pixels, width, height);
    let binarizer = HybridBinarizer::new(luma_source);
    let mut binary_bitmap = BinaryBitmap::new(binarizer);
    let mut reader = MultiFormatReader::default();

    match reader.decode_with_hints(&mut binary_bitmap, &hints) {
        Ok(result) => Ok(PyRXingResult::from(result)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "RXing decoding failed: {:?}",
            e
        ))),
    }
}

#[cfg(feature = "image")]
#[pyfunction]
fn decode_image_bytes(
    py: Python,
    image_file_bytes: &[u8],
    hints_dict: Option<&Bound<PyDict>>,
) -> PyResult<PyRXingResult> {
    let hints = py_dict_to_decode_hints(py, hints_dict)?;

    match image::load_from_memory(image_file_bytes) {
        Ok(dynamic_image) => {
            let luma_source = BufferedImageLuminanceSource::new(dynamic_image);
            let binarizer = HybridBinarizer::new(luma_source);
            let mut binary_bitmap = BinaryBitmap::new(binarizer);
            let mut reader = MultiFormatReader::default();

            match reader.decode_with_hints(&mut binary_bitmap, &hints) {
                Ok(result) => Ok(PyRXingResult::from(result)),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "RXing decoding failed: {:?}",
                    e
                ))),
            }
        }
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to load image from bytes: {:?}",
            e
        ))),
    }
}

#[cfg(feature = "image")]
#[pyfunction]
fn decode_from_file_path(
    py: Python,
    file_path_str: &str,
    hints_dict: Option<&Bound<PyDict>>,
) -> PyResult<PyRXingResult> {
    let path = PathBuf::from(file_path_str);

    if !path.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("File not found: {}", file_path_str),
        ));
    }

    let hints = py_dict_to_decode_hints(py, hints_dict)?;

    match image::open(&path) {
        Ok(dynamic_image) => {
            let luma_source = BufferedImageLuminanceSource::new(dynamic_image);
            let binarizer = HybridBinarizer::new(luma_source);
            let mut binary_bitmap = BinaryBitmap::new(binarizer);
            let mut reader = MultiFormatReader::default();

            match reader.decode_with_hints(&mut binary_bitmap, &hints) {
                Ok(result) => Ok(PyRXingResult::from(result)),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "RXing decoding failed for file {}: {:?}",
                    file_path_str, e
                ))),
            }
        }
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
            "Failed to open or decode image file {}: {:?}",
            file_path_str, e
        ))),
    }
}

#[cfg(feature = "detection")]
#[pyfunction]
fn decode_barcode_with_detection(
    _py: Python,
    image_file_bytes: &[u8],
    _hints_dict: Option<&Bound<PyDict>>, // hints not used yet, but kept for API consistency
) -> PyResult<Option<String>> {
    // 1. Get model path (downloads if necessary)
    let model_path = get_or_download_model_path()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to get model: {}", e)))?;

    // 2. Initialize detector
    let mut detector = YoloQrDetector::new(&model_path);

    // 3. Load image from bytes
    let usls_image = UslsImage::try_from(image::load_from_memory(image_file_bytes).unwrap())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to load image for detection: {}", e)))?;
    
    let dynamic_image = image::load_from_memory(image_file_bytes)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to load image for decoding: {}", e)))?;

    // 4. Detect barcodes
    let detections = detector.detect(usls_image);
    if detections.is_empty() {
        return Ok(None);
    }

    // 5. Attempt to decode each detection
    // This part could be parallelized if multiple detections are common
    for detection in &detections {
        let decoder_closure = |img: &DynamicImage| {
            let luma_source = BufferedImageLuminanceSource::new(img.clone());
            let binarizer = HybridBinarizer::new(luma_source);
            let mut binary_bitmap = BinaryBitmap::new(binarizer);
            let mut reader = MultiFormatReader::default();
            reader.decode(&mut binary_bitmap).ok().map(|result| result.getText().to_string())
        };

        if let Some(decoded_text) = enhance_and_decode_qr(&dynamic_image, detection, &decoder_closure) {
            return Ok(Some(decoded_text)); // Return the first successful decoding
        }
    }

    Ok(None) // No barcode could be decoded
}


#[pyfunction]
fn encode(
    py: Python,
    data: &str,
    format: &str,
    width: i32,
    height: i32,
    hints_dict: Option<&Bound<PyDict>>,
) -> PyResult<PyBitMatrix> {
    let barcode_format = BarcodeFormat::from(format.to_uppercase());

    let hints = py_dict_to_encode_hints(py, hints_dict)?;
    let writer = MultiFormatWriter;

    match writer.encode_with_hints(data, &barcode_format, width, height, &hints) {
        Ok(bit_matrix) => Ok(PyBitMatrix::from(bit_matrix)),
        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "RXing encoding failed: {:?}",
            e
        ))),
    }
}

#[pymodule]
#[pyo3(name = "rxing_lib")]
fn rxing_py_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyRXingResult>()?;
    m.add_class::<PyPoint>()?;
    m.add_class::<PyBitMatrix>()?;

    m.add_function(wrap_pyfunction!(decode_luma_pixels, m)?)?;
    #[cfg(feature = "image")]
    m.add_function(wrap_pyfunction!(decode_image_bytes, m)?)?;
    #[cfg(feature = "image")]
    m.add_function(wrap_pyfunction!(decode_from_file_path, m)?)?;
    m.add_function(wrap_pyfunction!(encode, m)?)?;

    #[cfg(feature = "detection")]
    m.add_function(wrap_pyfunction!(decode_barcode_with_detection, m)?)?;

    let py_barcode_format_module = PyModule::new(_py, "BarcodeFormat")?;
    py_barcode_format_module.add("AZTEC", BarcodeFormat::AZTEC.to_string())?;
    py_barcode_format_module.add("CODABAR", BarcodeFormat::CODABAR.to_string())?;
    py_barcode_format_module.add("CODE_39", BarcodeFormat::CODE_39.to_string())?;
    py_barcode_format_module.add("CODE_93", BarcodeFormat::CODE_93.to_string())?;
    py_barcode_format_module.add("CODE_128", BarcodeFormat::CODE_128.to_string())?;
    py_barcode_format_module.add("DATA_MATRIX", BarcodeFormat::DATA_MATRIX.to_string())?;
    py_barcode_format_module.add("EAN_8", BarcodeFormat::EAN_8.to_string())?;
    py_barcode_format_module.add("EAN_13", BarcodeFormat::EAN_13.to_string())?;
    py_barcode_format_module.add("ITF", BarcodeFormat::ITF.to_string())?;
    py_barcode_format_module.add("MAXICODE", BarcodeFormat::MAXICODE.to_string())?;
    py_barcode_format_module.add("PDF_417", BarcodeFormat::PDF_417.to_string())?;
    py_barcode_format_module.add("QR_CODE", BarcodeFormat::QR_CODE.to_string())?;
    py_barcode_format_module.add("RSS_14", BarcodeFormat::RSS_14.to_string())?;
    py_barcode_format_module.add("RSS_EXPANDED", BarcodeFormat::RSS_EXPANDED.to_string())?;
    py_barcode_format_module.add("UPC_A", BarcodeFormat::UPC_A.to_string())?;
    py_barcode_format_module.add("UPC_E", BarcodeFormat::UPC_E.to_string())?;
    py_barcode_format_module.add(
        "UPC_EAN_EXTENSION",
        BarcodeFormat::UPC_EAN_EXTENSION.to_string(),
    )?;
    m.add_submodule(&py_barcode_format_module)?;

    Ok(())
}
