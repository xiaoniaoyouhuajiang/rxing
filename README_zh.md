# rxing-python: rxing Rust 库的 Python 绑定

[![PyPI version](https://img.shields.io/pypi/v/rxing.svg)](https://pypi.org/project/rxing/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/your-username/rxing-python/ci.yml?branch=main)](https://github.com/your-username/rxing-python/actions) 
[![License](https://img.shields.io/pypi/l/rxing.svg)](https://opensource.org/licenses/Apache-2.0)
[![Python Versions](https://img.shields.io/pypi/pyversions/rxing.svg)](https://pypi.org/project/rxing/)

**[View in English (查看英文版)](README.md)**

`rxing-python` 为 [`rxing`](https://crates.io/crates/rxing)（一个纯 Rust 实现的流行 ZXing 多格式一维/二维条码图像处理库）提供了简洁高效的 Python 绑定。该软件包使 Python 开发者能够利用 Rust 的速度和可靠性，轻松解码和编码多种条码格式。

## 特性

*   **多样化解码**: 从多种来源解码条码：
    *   图像文件路径 (`str`)
    *   图像文件内容 (`bytes`)
    *   Pillow `Image` 对象
    *   NumPy `ndarray` 对象
*   **多格式支持**: 支持广泛的一维和二维条码符号，包括：
    *   QR Code (二维码)
    *   Data Matrix (数据矩阵码)
    *   Aztec (阿兹特克码)
    *   PDF417
    *   Code 39, Code 93, Code 128
    *   EAN-8, EAN-13
    *   UPC-A, UPC-E
    *   ITF
    *   Codabar (库德巴码)
    *   以及更多 (继承自 `rxing`)
*   **条码编码**: 为多种条码格式生成 `BitMatrix` 表示。
*   **便捷的 `BitMatrix` 处理**:
    *   直接将 `BitMatrix` 保存为图像文件 (PNG, JPEG 等)。
    *   将 `BitMatrix` 转换为 Pillow `Image` 对象。
    *   将 `BitMatrix` 转换为 NumPy `ndarray` 对象。
    *   获取用于控制台显示的字符串表示。
*   **提示系统**: 利用解码和编码提示对处理过程进行更精细的控制。

## 安装

您可以使用 pip 安装 `rxing-python`：

```bash
pip install rxing
```

这也会安装必要的依赖项：`Pillow` 和 `NumPy`。
如果从源代码构建，请确保您拥有兼容的 Rust 工具链。我们为常用平台提供了预编译的 wheel 包。

## 快速上手

以下是如何快速开始解码和编码条码的示例。

### 解码条码

`rxing.decode()` 函数提供了统一的解码接口。

```python
import rxing
from PIL import Image
import numpy as np

# 假设您有一个条码图片 "barcode_image.png"
image_file = "path/to/your/barcode_image.png" 

# 1. 从图像文件路径解码
try:
    result_from_path = rxing.decode(image_file)
    if result_from_path:
        print(f"解码文本 (来自路径): {result_from_path.text}")
        print(f"格式 (来自路径): {result_from_path.barcode_format}")
        # print(f"定位点: {result_from_path.result_points}")
except FileNotFoundError:
    print("错误: 图像文件未找到。")
except ValueError as e: # rxing 通常因解码错误抛出 ValueError
    print(f"从路径解码错误: {e}")

# 2. 从图像字节解码
try:
    with open(image_file, "rb") as f:
        image_bytes = f.read()
    result_from_bytes = rxing.decode(image_bytes)
    if result_from_bytes:
        print(f"解码文本 (来自字节): {result_from_bytes.text}")
except ValueError as e:
    print(f"从字节解码错误: {e}")

# 3. 从 Pillow Image 对象解码
try:
    pil_image = Image.open(image_file)
    result_from_pil = rxing.decode(pil_image)
    if result_from_pil:
        print(f"解码文本 (来自 PIL): {result_from_pil.text}")
except ValueError as e:
    print(f"从 PIL Image 解码错误: {e}")

# 4. 从 NumPy 数组解码
try:
    # 假设您有一个 NumPy 数组 (例如，来自 OpenCV 或其他来源)
    # 为演示，将 PIL 图像转换为 NumPy 数组
    pil_img_for_numpy = Image.open(image_file).convert('L') # 灰度图
    numpy_array = np.array(pil_img_for_numpy)
    
    result_from_numpy = rxing.decode(numpy_array)
    if result_from_numpy:
        print(f"解码文本 (来自 NumPy): {result_from_numpy.text}")
except ValueError as e:
    print(f"从 NumPy 数组解码错误: {e}")

# 访问 RXingResult 属性
if 'result_from_path' in locals() and result_from_path: # 假设 result_from_path 成功获取
    print(f"\n--- 结果详情 ---")
    print(f"文本: {result_from_path.text}")
    print(f"格式: {result_from_path.barcode_format}")
    print(f"原始字节 (前20字节): {result_from_path.raw_bytes[:20]}...")
    print(f"位数: {result_from_path.num_bits}")
    if result_from_path.result_points:
        print(f"定位点 (第一个点): ({result_from_path.result_points[0].x}, {result_from_path.result_points[0].y})")
    # print(f"时间戳: {result_from_path.timestamp}")
    # print(f"元数据: {result_from_path.result_metadata}")
```

### 编码条码 (例如，QR码)

`rxing.encode()` 函数创建条码的 `BitMatrix` 表示。

```python
import rxing

data_to_encode = "你好，来自 rxing-python！"
barcode_format = "QR_CODE" # 例如："QR_CODE", "CODE_128", "EAN_13"

try:
    # 编码数据 (width 和 height 是渲染提示，BitMatrix 维度基于模块)
    # 在编码示例中使用更实用的大尺寸提示
    bit_matrix = rxing.encode(data_to_encode, barcode_format, width=256, height=256) 
    print(f"\n--- 编码后的 BitMatrix ---")
    print(f"维度 (模块): {bit_matrix.width}x{bit_matrix.height}")

    # 将 BitMatrix 保存为图像
    output_path = "my_qr_code_zh.png"
    bit_matrix.save(output_path)
    print(f"QR码已保存至: {output_path}")

    # 或转换为 Pillow Image 对象
    pil_image_encoded = bit_matrix.to_pil_image()
    # pil_image_encoded.show() # 如果您想显示它

    # 或转换为 NumPy 数组
    numpy_array_encoded = bit_matrix.to_numpy_array()
    # print(f"NumPy 数组形状: {numpy_array_encoded.shape}")

    # 打印字符串表示 (仅适用于小型条码)
    if bit_matrix.width <= 50 and bit_matrix.height <= 50: # 避免打印过大的矩阵
         print("字符串表示:")
         print(str(bit_matrix))

except ValueError as e:
    print(f"编码错误: {e}")
```

## API 概览

*   `rxing.decode(source, hints=None)`: 解码条码。
    *   `source`: `str`, `bytes`, `PIL.Image.Image`, 或 `numpy.ndarray`。
    *   `hints` (可选): 解码提示的 `dict`。
    *   返回: `RXingResult` 对象，失败时抛出 `ValueError`。
*   `rxing.encode(data, format, width=5, height=5, hints_dict=None)`: 编码数据。
    *   `data`: 要编码的 `str`。
    *   `format`: 条码格式的 `str` (例如 "QR_CODE")。
    *   `width`, `height` (可选): 渲染图像尺寸的提示。`BitMatrix` 维度基于模块。
    *   `hints_dict` (可选): 编码提示的 `dict`。
    *   返回: `BitMatrix` 对象，失败时抛出 `ValueError`。
*   `rxing.RXingResult`: 表示解码结果的类。
    *   属性: `text`, `barcode_format`, `raw_bytes`, `num_bits`, `result_points` (`Point` 列表), `result_metadata`, `timestamp`。
*   `rxing.BitMatrix`: 表示编码后条码矩阵的类。
    *   属性: `width`, `height` (单位：模块), `data` (原始布尔矩阵)。
    *   方法: `save()`, `to_pil_image()`, `to_numpy_array()`, `__str__()`。
*   `rxing.Point`: 表示坐标点。
    *   属性: `x`, `y`。
*   `rxing.BarcodeFormat`: 类似模块的对象，包含条码格式的字符串常量 (例如 `rxing.BarcodeFormat.QR_CODE`)。

## 高级用法

### 使用提示 (Hints)

**解码提示:**
```python
# 示例：更努力地尝试，并且只查找 QR 码
hints = {
    "TRY_HARDER": True,
    "POSSIBLE_FORMATS": ["QR_CODE"] # 使用字符串格式名称
}
# result = rxing.decode(source, hints=hints)
```
常用解码提示包括: `TRY_HARDER`, `PURE_BARCODE`, `POSSIBLE_FORMATS`, `CHARACTER_SET`, `ALSO_INVERTED`。

**编码提示:**
```python
# 示例：设置 QR 码的纠错级别和边距
hints = {
    "ERROR_CORRECTION": "H", # 例如："L", "M", "Q", "H"
    "MARGIN": 2 # 边距 (单位：模块)
    # "QR_VERSION": 5 # 指定 QR 版本
}
# bit_matrix = rxing.encode("data", "QR_CODE", hints_dict=hints)
```
常用编码提示包括: `ERROR_CORRECTION`, `CHARACTER_SET`, `MARGIN`, `QR_VERSION`。

## 贡献

欢迎贡献！请随时提交问题 (issues)、功能请求或拉取请求 (pull requests)。
如果您计划贡献，请：
1.  Fork 本仓库。
2.  为您的功能或错误修复创建一个新分支。
3.  开发并测试您的更改。
4.  确保您的代码遵循现有风格，并在适当时添加测试。
5.  提交拉取请求。

## 许可证

本项目采用 Apache-2.0 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。

## 致谢

*   本项目是优秀的 [`rxing`](https://github.com/rxing-core/rxing) 库的 Python 绑定。
*   `rxing` 本身是原始 [`ZXing`](https://github.com/zxing/zxing) 库的一个移植。
*   感谢 PyO3 和 Maturin 的开发者们，使得 Rust-Python 的互操作变得简单。
