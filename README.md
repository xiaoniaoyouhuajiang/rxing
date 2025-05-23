# rxing-python: Python Bindings for the rxing Rust Library

[![PyPI version](https://img.shields.io/pypi/v/rxing.svg)](https://pypi.org/project/rxing/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/rxing-python/ci.yml?branch=main)](https://github.com/your-username/rxing-python/actions) 
[![License](https://img.shields.io/pypi/l/rxing.svg)](https://opensource.org/licenses/Apache-2.0)
[![Python Versions](https://img.shields.io/pypi/pyversions/rxing.svg)](https://pypi.org/project/rxing/)

**[中文说明 (View in Chinese)](README_zh.md)**

`rxing-python` provides simple and efficient Python bindings for [`rxing`](https://crates.io/crates/rxing), a pure Rust port of the popular ZXing multi-format 1D/2D barcode image processing library. This package allows Python developers to easily decode and encode a wide variety of barcode formats using the speed and reliability of Rust.

## Features

*   **Versatile Decoding**: Decode barcodes from various sources:
    *   Image file paths (`str`)
    *   Image file content (`bytes`)
    *   Pillow `Image` objects
    *   NumPy `ndarray` objects
*   **Multi-Format Support**: Supports a wide range of 1D and 2D barcode symbologies, including:
    *   QR Code
    *   Data Matrix
    *   Aztec
    *   PDF417
    *   Code 39, Code 93, Code 128
    *   EAN-8, EAN-13
    *   UPC-A, UPC-E
    *   ITF
    *   Codabar
    *   And more (inherited from `rxing`)
*   **Barcode Encoding**: Generate `BitMatrix` representations for various barcode formats.
*   **Convenient `BitMatrix` Handling**:
    *   Save `BitMatrix` directly to image files (PNG, JPEG, etc.).
    *   Convert `BitMatrix` to Pillow `Image` objects.
    *   Convert `BitMatrix` to NumPy `ndarray` objects.
    *   Get a string representation for console display.
*   **Hint System**: Utilize decoding and encoding hints for more control over the process.

## Installation

You can install `rxing-python` using pip:

```bash
pip install rxing
```

This will also install the necessary dependencies: `Pillow` and `NumPy`.
Ensure you have a compatible Rust toolchain if building from source. Pre-built wheels are provided for common platforms.

*(Note: The actual PyPI package name might differ; replace `rxing` with the correct name if needed.)*

## Quickstart

Here's how you can quickly get started with decoding and encoding barcodes.

### Decoding a Barcode

The `rxing.decode()` function provides a unified interface for decoding.

```python
import rxing
from PIL import Image
import numpy as np

# 1. Decode from an image file path
try:
    result_from_path = rxing.decode("path/to/your/barcode_image.png")
    if result_from_path:
        print(f"Decoded Text (from path): {result_from_path.text}")
        print(f"Format (from path): {result_from_path.barcode_format}")
        # print(f"Points: {result_from_path.result_points}")
except FileNotFoundError:
    print("Error: Image file not found.")
except ValueError as e: # rxing typically raises ValueError for decode errors
    print(f"Error decoding from path: {e}")

# 2. Decode from image bytes
try:
    with open("path/to/your/barcode_image.png", "rb") as f:
        image_bytes = f.read()
    result_from_bytes = rxing.decode(image_bytes)
    if result_from_bytes:
        print(f"Decoded Text (from bytes): {result_from_bytes.text}")
except ValueError as e:
    print(f"Error decoding from bytes: {e}")

# 3. Decode from a Pillow Image object
try:
    pil_image = Image.open("path/to/your/barcode_image.png")
    result_from_pil = rxing.decode(pil_image)
    if result_from_pil:
        print(f"Decoded Text (from PIL): {result_from_pil.text}")
except ValueError as e:
    print(f"Error decoding from PIL Image: {e}")

# 4. Decode from a NumPy array
try:
    # Assuming you have a NumPy array (e.g., from OpenCV or other sources)
    # For demonstration, convert a PIL image to NumPy array
    pil_img_for_numpy = Image.open("path/to/your/barcode_image.png").convert('L') # Grayscale
    numpy_array = np.array(pil_img_for_numpy)
    
    result_from_numpy = rxing.decode(numpy_array)
    if result_from_numpy:
        print(f"Decoded Text (from NumPy): {result_from_numpy.text}")
except ValueError as e:
    print(f"Error decoding from NumPy array: {e}")

# Accessing RXingResult properties
if result_from_path: # Assuming result_from_path was successful
    print(f"\n--- Result Details ---")
    print(f"Text: {result_from_path.text}")
    print(f"Format: {result_from_path.barcode_format}")
    print(f"Raw Bytes: {result_from_path.raw_bytes[:20]}...") # Show first 20 raw bytes
    print(f"Number of Bits: {result_from_path.num_bits}")
    if result_from_path.result_points:
        print(f"Result Points (first point): ({result_from_path.result_points[0].x}, {result_from_path.result_points[0].y})")
    # print(f"Timestamp: {result_from_path.timestamp}")
    # print(f"Metadata: {result_from_path.result_metadata}")
```

### Encoding a Barcode (e.g., QR Code)

The `rxing.encode()` function creates a `BitMatrix` representation of the barcode.

```python
import rxing

data_to_encode = "Hello from rxing-python!"
barcode_format = "QR_CODE" # e.g., "QR_CODE", "CODE_128", "EAN_13"

try:
    # Encode data (width and height are hints for rendering, BitMatrix dimensions are module-based)
    # Using larger, more practical default hints for encoding example
    bit_matrix = rxing.encode(data_to_encode, barcode_format, width=256, height=256) 
    print(f"\n--- Encoded BitMatrix ---")
    print(f"Dimensions (modules): {bit_matrix.width}x{bit_matrix.height}")

    # Save the BitMatrix as an image
    output_path = "my_qr_code.png"
    bit_matrix.save(output_path)
    print(f"Saved QR code to: {output_path}")

    # Or convert to a Pillow Image
    pil_image_encoded = bit_matrix.to_pil_image()
    # pil_image_encoded.show() # If you want to display it

    # Or convert to a NumPy array
    numpy_array_encoded = bit_matrix.to_numpy_array()
    # print(f"NumPy array shape: {numpy_array_encoded.shape}")

    # Print a string representation (small barcodes only)
    if bit_matrix.width <= 50 and bit_matrix.height <= 50: # Avoid printing huge matrices
         print("String representation:")
         print(str(bit_matrix))

except ValueError as e:
    print(f"Error encoding: {e}")
```

## API Overview

*   `rxing.decode(source, hints=None)`: Decodes a barcode.
    *   `source`: `str`, `bytes`, `PIL.Image.Image`, or `numpy.ndarray`.
    *   `hints` (optional): A `dict` of decoding hints.
    *   Returns: `RXingResult` object or raises `ValueError` on failure.
*   `rxing.encode(data, format, width=5, height=5, hints_dict=None)`: Encodes data.
    *   `data`: `str` to encode.
    *   `format`: `str` barcode format (e.g., "QR_CODE").
    *   `width`, `height` (optional): Hints for rendered image size. `BitMatrix` dimensions are module-based.
    *   `hints_dict` (optional): A `dict` of encoding hints.
    *   Returns: `BitMatrix` object or raises `ValueError` on failure.
*   `rxing.RXingResult`: Class representing decoding results.
    *   Properties: `text`, `barcode_format`, `raw_bytes`, `num_bits`, `result_points` (list of `Point`), `result_metadata`, `timestamp`.
*   `rxing.BitMatrix`: Class representing the encoded barcode matrix.
    *   Properties: `width`, `height` (in modules), `data` (raw boolean matrix).
    *   Methods: `save()`, `to_pil_image()`, `to_numpy_array()`, `__str__()`.
*   `rxing.Point`: Represents a coordinate point.
    *   Properties: `x`, `y`.
*   `rxing.BarcodeFormat`: Module-like object containing string constants for barcode formats (e.g., `rxing.BarcodeFormat.QR_CODE`).

## Advanced Usage

### Using Hints

**Decoding Hints:**
```python
# Example: Try harder and only look for QR codes
hints = {
    "TRY_HARDER": True,
    "POSSIBLE_FORMATS": ["QR_CODE"] # Use string format name
}
# result = rxing.decode(source, hints=hints)
```
Common decode hints include: `TRY_HARDER`, `PURE_BARCODE`, `POSSIBLE_FORMATS`, `CHARACTER_SET`, `ALSO_INVERTED`.

**Encoding Hints:**
```python
# Example: Set QR code error correction level and margin
hints = {
    "ERROR_CORRECTION": "H", # e.g., "L", "M", "Q", "H"
    "MARGIN": 2 # Margin in modules
    # "QR_VERSION": 5 # Specify QR version
}
# bit_matrix = rxing.encode("data", "QR_CODE", hints_dict=hints)
```
Common encode hints include: `ERROR_CORRECTION`, `CHARACTER_SET`, `MARGIN`, `QR_VERSION`.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.
If you plan to contribute, please:
1.  Fork the repository.
2.  Create a new branch for your feature or bug fix.
3.  Develop and test your changes.
4.  Ensure your code follows existing style and add tests if applicable.
5.  Submit a pull request.

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](LICENSE) file for details.
(This assumes you will add an Apache-2.0 LICENSE file, consistent with `rxing`.)

## Acknowledgements

*   This project is a Python binding for the excellent [`rxing`](https://github.com/rxing-core/rxing) library.
*   `rxing` itself is a port of the original [`ZXing`](https://github.com/zxing/zxing) library.
*   Thanks to the developers of PyO3 and Maturin for enabling easy Rust-Python interoperability.

---
*Replace `your-username/rxing-python` in badge URLs with your actual GitHub repository path.*
*Ensure the PyPI package name in installation instructions and badges is correct.*
