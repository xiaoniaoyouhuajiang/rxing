from .rxing_lib import (
    decode_luma_pixels as _decode_luma_pixels,
    decode_image_bytes as _decode_image_bytes,
    decode_from_file_path as _decode_from_file_path,
    encode as _encode, # Import Rust encode as _encode
    RXingResult,
    Point,
    BitMatrix as _RustBitMatrix,
    BarcodeFormat,
)
import PIL.Image
import numpy as np

def decode(source, hints=None):
    """
    Decodes a barcode from various sources.

    :param source: The source to decode from. Can be:
                   - str: Path to an image file.
                   - bytes: Image file content as bytes.
                   - PIL.Image.Image: A Pillow Image object.
                   - numpy.ndarray: A NumPy array representing an image.
                                    (expects uint8, 2D for grayscale, 3D for RGB/RGBA)
    :param hints: Optional dictionary of decoding hints.
    :return: RXingResult object.
    :raises TypeError: If the source type is not supported.
    """
    if hints is None:
        hints = {}

    if isinstance(source, str):
        return _decode_from_file_path(source, hints)
    elif isinstance(source, bytes):
        return _decode_image_bytes(source, hints)
    elif isinstance(source, PIL.Image.Image):
        img = source
        if img.mode not in ('L', 'RGB', 'RGBA'):
            img = img.convert('L')
        elif img.mode != 'L':
             img = img.convert('L')

        width, height = img.size
        luma_data = img.tobytes()
        return _decode_luma_pixels(luma_data, width, height, hints)
    elif isinstance(source, np.ndarray):
        if source.dtype != np.uint8:
            raise TypeError("NumPy array must be of dtype uint8.")

        if source.ndim == 2: # Grayscale
            pil_img = PIL.Image.fromarray(source, mode='L')
        elif source.ndim == 3 and source.shape[2] in (3, 4): # RGB or RGBA
            pil_img = PIL.Image.fromarray(source, mode='RGB' if source.shape[2] == 3 else 'RGBA')
            pil_img = pil_img.convert('L')
        else:
            raise TypeError("NumPy array must be 2D (grayscale) or 3D (RGB/RGBA).")

        width, height = pil_img.size
        luma_data = pil_img.tobytes()
        return _decode_luma_pixels(luma_data, width, height, hints)
    else:
        raise TypeError(
            "Unsupported source type. Expected str, bytes, PIL.Image.Image, or numpy.ndarray."
        )

def encode(data: str, format: str, width: int = 29, height: int = 29, hints_dict: dict = None):
    """
    Encodes data into a barcode/QR code.

    The `width` and `height` parameters are hints to the encoder for the desired output
    pixel dimensions of a rendered image. However, the returned `BitMatrix` object
    will have its dimensions (modules or logical units) determined by the barcode
    standard, data content, and encoding hints (e.g., QR code version).
    The encoder will attempt to choose a module size that best fits the provided
    `width` and `height` for a final rendered image, but the `BitMatrix.width` and
    `BitMatrix.height` will reflect the actual module count.

    :param data: The string data to encode.
    :param format: The barcode format to use (e.g., "QR_CODE", "CODE_128").
    :param width: Hint for the desired output width in pixels for a rendered image. Defaults to 5.
                  The library will determine the actual module count for the BitMatrix.
                  A value of 5 is very small and likely to be overridden by the minimum
                  module requirements of the chosen barcode format.
    :param height: Hint for the desired output height in pixels for a rendered image. Defaults to 5.
                   Similar to width, this is a hint and the BitMatrix module count will
                   be determined by the standard.
    :param hints_dict: Optional dictionary of encoding hints. Defaults to None.
    :return: BitMatrix object representing the encoded barcode. Its dimensions are in modules.
    :raises ValueError: If encoding fails (e.g., invalid format, data too large for format).
    """
    if hints_dict is None:
        hints_dict = {}
    return _encode(data, format, width, height, hints_dict)

# --- Methods to add to BitMatrix ---
def _bitmatrix_to_pil_image(self) -> PIL.Image.Image:
    """Converts the BitMatrix to a Pillow Image object (mode '1')."""
    if self.width == 0 or self.height == 0:
        return PIL.Image.new('1', (0,0))

    img_l = PIL.Image.new('L', (self.width, self.height))
    pixels_l = img_l.load()
    matrix_data = self.data
    for y in range(self.height):
        for x in range(self.width):
            pixels_l[x, y] = 0 if matrix_data[y][x] else 255

    return img_l.convert('1')

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
        row_str = "".join(["██" if matrix_data[y][x] else "  " for x in range(self.width)])
        s.append(row_str)
    return "\n".join(s)

_RustBitMatrix.to_pil_image = _bitmatrix_to_pil_image
_RustBitMatrix.to_numpy_array = _bitmatrix_to_numpy_array
_RustBitMatrix.save = _bitmatrix_save
_RustBitMatrix.__str__ = _bitmatrix_str

BitMatrix = _RustBitMatrix

__all__ = [
    "decode",
    "encode", # Expose the new Python wrapper for encode
    "RXingResult",
    "Point",
    "BitMatrix",
    "BarcodeFormat",
]
