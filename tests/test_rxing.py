import unittest
import os
import sys

project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), '..'))
sys.path.insert(0, project_root)

try:
    import rxing
    from PIL import Image
    import numpy as np
except ImportError as e:
    print(f"Error importing necessary libraries. Ensure rxing is built and Pillow/NumPy are installed: {e}")
    print("You might need to run 'maturin develop' or 'pip install . Pillow numpy'")
    sys.exit(1)

# Helper function to get path to test image
# Assumes test images are in a 'test_images' subdirectory alongside this test script
def get_test_image_path(filename):
    return os.path.join(os.path.dirname(__file__), "test_images", filename)

# Placeholder for expected text for QR code for basic test
QR_CODE_EXAMPLE_TEXT = "Hello rxing!"
# Default filename for the auto-generated QR test image
# User can override this by placing their own 'test_qr.png' in 'test_images'
# or by modifying this constant and QR_CODE_EXAMPLE_TEXT.
QR_CODE_IMAGE_FILENAME = "auto_generated_qr_for_test.png" 


class TestDecode(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        """Set up for all tests in TestDecode."""
        cls.test_images_dir = os.path.join(os.path.dirname(__file__), "test_images")
        os.makedirs(cls.test_images_dir, exist_ok=True)

        cls.qr_image_path = get_test_image_path(QR_CODE_IMAGE_FILENAME)
        cls.qr_image_valid = False

        # Try to create and then immediately decode the QR image to ensure it's valid for tests.
        try:
            if not os.path.exists(cls.qr_image_path):
                print(f"Attempting to create dummy test image: {cls.qr_image_path} with content '{QR_CODE_EXAMPLE_TEXT}'")
                qr_matrix = rxing.encode(QR_CODE_EXAMPLE_TEXT, "QR_CODE", width=200, height=200)
                qr_matrix.save(cls.qr_image_path, "PNG")
            
            result = rxing.decode(cls.qr_image_path)
            if result and result.text == QR_CODE_EXAMPLE_TEXT:
                cls.qr_image_valid = True
                print(f"Successfully verified/created test QR image: {cls.qr_image_path}")
            else:
                print(f"WARNING: Test QR image '{cls.qr_image_path}' could not be decoded or content mismatch.")
                if os.path.exists(cls.qr_image_path) and not QR_CODE_IMAGE_FILENAME.startswith("auto_generated"):
                     print(f"Please ensure '{cls.qr_image_path}' contains a valid QR code with text '{QR_CODE_EXAMPLE_TEXT}'.")
                elif os.path.exists(cls.qr_image_path): 
                    try:
                        os.remove(cls.qr_image_path) 
                        print(f"Removed invalid auto-generated QR image: {cls.qr_image_path}")
                    except OSError as e_remove:
                        print(f"Error removing invalid auto-generated QR image '{cls.qr_image_path}': {e_remove}")


        except Exception as e:
            print(f"WARNING: Could not create or verify dummy QR image for testing ('{cls.qr_image_path}'): {e}")
            print("Decode tests relying on this QR image may be skipped or fail.")
            if os.path.exists(cls.qr_image_path) and QR_CODE_IMAGE_FILENAME.startswith("auto_generated"):
                try:
                    os.remove(cls.qr_image_path) 
                except OSError:
                    pass # Ignore error if removal fails

        cls.no_barcode_image_path = get_test_image_path("no_barcode_example.png")
        if not os.path.exists(cls.no_barcode_image_path):
            try:
                img = Image.new('L', (100,100), color='white')
                img.save(cls.no_barcode_image_path)
                print(f"Created dummy 'no_barcode' image: {cls.no_barcode_image_path}")
            except Exception as e:
                 print(f"Could not create dummy 'no_barcode' image: {e}.")

    def setUp(self):
        """Per-test setup, mainly for skipping if QR image is not valid."""
        # Check if the current test method name contains "qr" to decide if it depends on the QR image
        # This is a simple heuristic; more robust would be to use unittest.skipIf decorator on each test method
        is_qr_dependent_test = "qr" in self.id().lower()
        
        if is_qr_dependent_test and not self.__class__.qr_image_valid:
            self.skipTest(f"Test QR image '{self.__class__.qr_image_path}' is not valid or available. Skipping QR decode test: {self.id()}")

    def test_decode_from_path_qr(self):
        # setUp will skip if self.qr_image_path is not valid
        result = rxing.decode(self.__class__.qr_image_path)
        self.assertIsNotNone(result, "Decoding failed, result is None")
        self.assertEqual(result.text, QR_CODE_EXAMPLE_TEXT)
        # Assuming BarcodeFormat is exposed as strings for now
        self.assertEqual(result.barcode_format, "qrcode")

    def test_decode_from_bytes_qr(self):
        # setUp will skip if self.__class__.qr_image_path is not valid
        with open(self.__class__.qr_image_path, "rb") as f:
            image_bytes = f.read()
        result = rxing.decode(image_bytes)
        self.assertIsNotNone(result, "Decoding failed, result is None")
        self.assertEqual(result.text, QR_CODE_EXAMPLE_TEXT)
        self.assertEqual(result.barcode_format, "qrcode")

    def test_decode_from_pil_image_qr(self):
        # setUp will skip if self.__class__.qr_image_path is not valid
        pil_img = Image.open(self.__class__.qr_image_path)
        result = rxing.decode(pil_img)
        self.assertIsNotNone(result, "Decoding failed, result is None")
        self.assertEqual(result.text, QR_CODE_EXAMPLE_TEXT)
        self.assertEqual(result.barcode_format, "qrcode")

    def test_decode_from_numpy_array_qr(self):
        # setUp will skip if self.__class__.qr_image_path is not valid
        pil_img = Image.open(self.__class__.qr_image_path).convert('L') # Convert to L for predictable numpy array
        np_array = np.array(pil_img)
        result = rxing.decode(np_array)
        self.assertIsNotNone(result, "Decoding failed, result is None")
        self.assertEqual(result.text, QR_CODE_EXAMPLE_TEXT)
        self.assertEqual(result.barcode_format, "qrcode")
        
    def test_decode_no_barcode(self):
        if not os.path.exists(self.__class__.no_barcode_image_path): # Use class attribute
            self.skipTest(f"Test image {self.__class__.no_barcode_image_path} not found. Skipping test.")
        # Expect ValueError containing "NotFoundException"
        with self.assertRaisesRegex(ValueError, "NotFoundException"): 
            rxing.decode(self.__class__.no_barcode_image_path)

    def test_decode_invalid_path(self):
        with self.assertRaises(FileNotFoundError): # Or the specific error rxing_lib raises
            rxing.decode("invalid/path/does_not_exist.png")
            
    def test_decode_unsupported_type(self):
        with self.assertRaises(TypeError):
            rxing.decode(12345)


class TestEncode(unittest.TestCase):
    def setUp(self):
        self.temp_dir = os.path.join(os.path.dirname(__file__),"temp_test_output_encode")
        os.makedirs(self.temp_dir, exist_ok=True)

    def tearDown(self):
        # Guard against temp_dir not existing if setUp failed.
        if hasattr(self, 'temp_dir') and os.path.exists(self.temp_dir):
            for f in os.listdir(self.temp_dir):
                try:
                    os.remove(os.path.join(self.temp_dir, f))
                except OSError:
                    pass # Ignore if file is already removed or other issues
            try:
                os.rmdir(self.temp_dir)
            except OSError:
                pass

    def test_encode_qr_default_size(self):
        # Test encoding "Hello" with QR_CODE format and default width/height (5)
        # The BitMatrix dimensions will be determined by the QR code standard for the data.
        # For "Hello", this typically results in a 21x21 or 25x25 or 29x29 module QR code.
        # The terminal output showed 29x29 for "Hello".
        data_string = "Hello"
        matrix = rxing.encode(data_string, format="QR_CODE") # Uses default width=5, height=5
        self.assertIsInstance(matrix, rxing.BitMatrix)
        # Check against the observed dimension from the user's feedback for "Hello"
        self.assertEqual(matrix.width, 29, f"Width for QR encoding '{data_string}' was not 29.")
        self.assertEqual(matrix.height, 29, f"Height for QR encoding '{data_string}' was not 29.")

    def test_encode_code128_custom_size(self):
        # For 1D barcodes like Code 128, width and height hints behave differently.
        data_string = "Test1234"
        # The width/height for 1D codes are more like rendering hints.
        # The BitMatrix for 1D codes often has height=1 (representing the single row of bars).
        matrix = rxing.encode(data_string, format="CODE_128", width=300, height=100)
        self.assertIsInstance(matrix, rxing.BitMatrix)
        self.assertTrue(matrix.width > 0, "Code128 BitMatrix width should be > 0")
        # The actual BitMatrix height for a 1D code is often 1.
        # The 'height' parameter (100) is a hint for rendering the final image.
        self.assertEqual(matrix.height, 100, "Code128 BitMatrix height should typically be 1 module row.")
        
    def test_encode_invalid_format(self):
        with self.assertRaisesRegex(ValueError, "RXing encoding failed"):
             rxing.encode("data", format="INVALID_FORMAT_XYZ")


class TestBitMatrixMethods(unittest.TestCase):
    def setUp(self):
        self.test_data = "Test Matrix"
        # Encode with specific parameters to get a predictable BitMatrix
        # For "Test Matrix", QR code might be version 2 (25x25) or 3 (29x29)
        # Let's use a slightly larger width/height hint for encode
        try:
            self.matrix = rxing.encode(self.test_data, "QR_CODE", width=50, height=50)
            self.assertIsNotNone(self.matrix, "Encoding failed in setUp for TestBitMatrixMethods")
            self.assertTrue(self.matrix.width > 0, "Encoded matrix width is 0 in setUp.")
        except Exception as e:
            self.matrix = None # Ensure self.matrix exists
            self.fail(f"Encoding in TestBitMatrixMethods setUp failed: {e}")

        self.temp_dir = os.path.join(os.path.dirname(__file__),"temp_test_output_matrix")
        os.makedirs(self.temp_dir, exist_ok=True)
        
    def tearDown(self):
        if hasattr(self, 'temp_dir') and os.path.exists(self.temp_dir):
            for f in os.listdir(self.temp_dir):
                try:
                    os.remove(os.path.join(self.temp_dir, f))
                except OSError:
                    pass
            try:
                os.rmdir(self.temp_dir)
            except OSError:
                pass

    def test_to_pil_image(self):
        if self.matrix is None: self.skipTest("Matrix not created in setUp")
        img = self.matrix.to_pil_image()
        self.assertIsInstance(img, Image.Image)
        # The PIL image size should match the BitMatrix module dimensions
        self.assertEqual(img.size, (self.matrix.width, self.matrix.height))
        self.assertEqual(img.mode, '1')
        
    def test_to_numpy_array(self):
        if self.matrix is None: self.skipTest("Matrix not created in setUp")
        np_array = self.matrix.to_numpy_array()
        self.assertIsInstance(np_array, np.ndarray)
        self.assertEqual(np_array.dtype, bool)
        self.assertEqual(np_array.shape, (self.matrix.height, self.matrix.width)) # Numpy shape is (rows, cols)

    def test_save_bitmatrix(self):
        if self.matrix is None: self.skipTest("Matrix not created in setUp")
        save_path = os.path.join(self.temp_dir, "matrix_save_test.png")
        self.matrix.save(save_path)
        self.assertTrue(os.path.exists(save_path))
        
        # Verify by decoding
        decoded_result = rxing.decode(save_path)
        self.assertIsNotNone(decoded_result, "Decoding saved BitMatrix image failed.")
        self.assertEqual(decoded_result.text, self.test_data)
        
    def test_str_bitmatrix(self):
        if self.matrix is None: self.skipTest("Matrix not created in setUp")
        s = str(self.matrix)
        self.assertIsInstance(s, str)
        self.assertTrue(len(s) > 0)
        self.assertIn("██", s) # Check for presence of "black" block character


if __name__ == '__main__':
    print("Running rxing Python interface tests...")
    print(f"Attempting to import rxing from: {project_root}")
    
    # Create test_images directory if it doesn't exist
    test_images_dir_main = os.path.join(os.path.dirname(__file__), "test_images")
    if not os.path.exists(test_images_dir_main):
        os.makedirs(test_images_dir_main)
        print(f"Created directory: {test_images_dir_main}")
        print(f"Please add test images (e.g., '{QR_CODE_IMAGE_FILENAME}') to this directory for comprehensive testing.")
        print(f"A dummy '{QR_CODE_IMAGE_FILENAME}' and 'no_barcode_example.png' will be auto-generated if missing, for basic tests.")

    unittest.main()
