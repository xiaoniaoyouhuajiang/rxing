from .rxing_lib import (
    decode_luma_pixels as _decode_luma_pixels,
    decode_image_bytes as _decode_image_bytes,
    decode_from_file_path as _decode_from_file_path,
    encode as _encode,
    RXingResult,
    Point,
    BitMatrix as _RustBitMatrix,
    BarcodeFormat,
)
import PIL.Image
import numpy as np
import io

# --- Conditional import for detection feature ---
try:
    from .rxing_lib import decode_barcode_with_detection as _decode_barcode_with_detection
    _DETECTION_AVAILABLE = True
except ImportError:
    _DETECTION_AVAILABLE = False

def decode(source, hints=None):
    """
    Decodes a barcode from various sources using traditional methods.

    :param source: The source to decode from. Can be:
                   - str: Path to an image file.
                   - bytes: Image file content as bytes.
                   - PIL.Image.Image: A Pillow Image object.
                   - numpy.ndarray: A NumPy array representing an image.
    :param hints: Optional dictionary of decoding hints.
    :return: RXingResult object.
    :raises TypeError: If the source type is not supported.
    :raises ValueError: If decoding fails.
    """
    if hints is None:
        hints = {}

    if isinstance(source, str):
        return _decode_from_file_path(source, hints)
    elif isinstance(source, bytes):
        return _decode_image_bytes(source, hints)
    elif isinstance(source, PIL.Image.Image):
        img = source
        if img.mode not in ("L", "RGB", "RGBA"):
            img = img.convert("L")
        elif img.mode != "L":
            img = img.convert("L")

        width, height = img.size
        luma_data = img.tobytes()
        return _decode_luma_pixels(luma_data, width, height, hints)
    elif isinstance(source, np.ndarray):
        if source.dtype != np.uint8:
            raise TypeError("NumPy array must be of dtype uint8.")

        if source.ndim == 2:
            pil_img = PIL.Image.fromarray(source, mode="L")
        elif source.ndim == 3 and source.shape[2] in (3, 4):
            pil_img = PIL.Image.fromarray(
                source, mode="RGB" if source.shape[2] == 3 else "RGBA"
            )
            pil_img = pil_img.convert("L")
        else:
            raise TypeError("NumPy array must be 2D (grayscale) or 3D (RGB/RGBA).")

        width, height = pil_img.size
        luma_data = pil_img.tobytes()
        return _decode_luma_pixels(luma_data, width, height, hints)
    else:
        raise TypeError(
            "Unsupported source type. Expected str, bytes, PIL.Image.Image, or numpy.ndarray."
        )

def decode_with_detection(source, hints=None):
    """
    Decodes a QR code from an image using a detection model first.
    This can succeed on more difficult images but is slower.

    This function is only available if the 'detection' feature was installed
    (e.g., `pip install rxing[detection]`).

    :param source: The source to decode from. Can be:
                   - str: Path to an image file.
                   - bytes: Image file content as bytes.
                   - PIL.Image.Image: A Pillow Image object.
                   - numpy.ndarray: A NumPy array representing an image.
    :param hints: Optional dictionary of decoding hints (currently not used but reserved).
    :return: A string containing the decoded text, or None if not found.
    :raises RuntimeError: If the 'detection' feature is not installed.
    :raises TypeError: If the source type is not supported.
    :raises ValueError: If the image cannot be processed.
    :raises IOError: If the model cannot be downloaded or accessed.
    """
    if not _DETECTION_AVAILABLE:
        raise RuntimeError(
            "The 'detection' feature is not available. "
            "Please install it using 'pip install rxing[detection]'."
        )

    if isinstance(source, str):
        with open(source, "rb") as f:
            image_bytes = f.read()
    elif isinstance(source, bytes):
        image_bytes = source
    elif isinstance(source, PIL.Image.Image):
        buffer = io.BytesIO()
        # Ensure image is in a format that can be saved and read, like PNG
        source.save(buffer, format="PNG")
        image_bytes = buffer.getvalue()
    elif isinstance(source, np.ndarray):
        if source.dtype != np.uint8:
            raise TypeError("NumPy array must be of dtype uint8.")
        pil_img = PIL.Image.fromarray(source)
        buffer = io.BytesIO()
        pil_img.save(buffer, format="PNG")
        image_bytes = buffer.getvalue()
    else:
        raise TypeError(
            "Unsupported source type. Expected str, bytes, PIL.Image.Image, or numpy.ndarray."
        )

    return _decode_barcode_with_detection(image_bytes, hints)


def encode(
    data: str, format: str, width: int = 29, height: int = 29, hints_dict: dict = None
):
    """
    Encodes data into a barcode/QR code.
    ... (docstring content remains the same)
    """
    if hints_dict is None:
        hints_dict = {}
    return _encode(data, format, width, height, hints_dict)


# --- Methods to add to BitMatrix ---
def _bitmatrix_to_pil_image(self) -> PIL.Image.Image:
    """Converts the BitMatrix to a Pillow Image object (mode '1')."""
    if self.width == 0 or self.height == 0:
        return PIL.Image.new("1", (0, 0))

    img_l = PIL.Image.new("L", (self.width, self.height))
    pixels_l = img_l.load()
    matrix_data = self.data
    for y in range(self.height):
        for x in range(self.width):
            pixels_l[x, y] = 0 if matrix_data[y][x] else 255

    return img_l.convert("1")


def _bitmatrix_to_numpy_array(self) -> np.ndarray:
    """Converts the BitMatrix to a NumPy array (dtype=bool)."""
    return np.array(self.data, dtype=bool)


def _bitmatrix_save(self, file_path: str, image_format: str = "PNG"):
    """Saves the BitMatrix as an image file."""
    pil_img = self.to_pil_image()
    pil_img.save(file_path, format=image_format.upper())


def _bitmatrix_str(self) -> str:
    """Returns a string representation of the BitMatrix."""
    if self.width == 0 or self.height == 0:
        return "<BitMatrix (empty)>"

    matrix_data = self.data
    s = []
    for y in range(self.height):
        row_str = "".join(
            ["██" if matrix_data[y][x] else "  " for x in range(self.width)]
        )
        s.append(row_str)
    return "\n".join(s)


_RustBitMatrix.to_pil_image = _bitmatrix_to_pil_image
_RustBitMatrix.to_numpy_array = _bitmatrix_to_numpy_array
_RustBitMatrix.save = _bitmatrix_save
_RustBitMatrix.__str__ = _bitmatrix_str

BitMatrix = _RustBitMatrix

__all__ = [
    "decode",
    "encode",
    "RXingResult",
    "Point",
    "BitMatrix",
    "BarcodeFormat",
]

if _DETECTION_AVAILABLE:
    __all__.append("decode_with_detection")
