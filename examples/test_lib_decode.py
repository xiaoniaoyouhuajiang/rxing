from PIL import Image
import rxing

img = Image.open("examples/test.png").convert("L")
width, height = img.size
luma_bytes = img.tobytes()
decode_a = rxing.decode_luma_pixels(luma_bytes, width, height, {})
decode_b = rxing.decode_luma_pixels(
    luma_bytes, width, height, {"POSSIBLE_FORMATS": [rxing.BarcodeFormat.DATA_MATRIX]}
)
assert decode_a == decode_b
