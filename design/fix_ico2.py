#!/usr/bin/env python3
"""Fix ICO file generation using proper method"""

import os
import struct
from PIL import Image

source_path = r'e:\Claw\wtools\src-tauri\icons\icon.png'
output_dir = r'e:\Claw\wtools\src-tauri\icons'

# Load source
source = Image.open(source_path)
if source.mode != 'RGBA':
    source = source.convert('RGBA')

# ICO needs specific sizes
sizes = [16, 32, 48, 64, 128, 256]

print("Generating ICO with {} images...".format(len(sizes)))

# Convert each size to BMP/PNG data for ICO
images_data = []
for size in sizes:
    img = source.resize((size, size), Image.Resampling.LANCZOS)
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    # Save to bytes as PNG (better quality for alpha)
    from io import BytesIO
    output = BytesIO()
    img.save(output, format='PNG')
    data = output.getvalue()
    images_data.append((size, data))
    print("  Size {}x{}: {} bytes".format(size, size, len(data)))

# Build ICO file manually
ico_path = os.path.join(output_dir, 'icon.ico')

with open(ico_path, 'wb') as f:
    # ICO Header
    f.write(struct.pack('<HHH', 0, 1, len(images_data)))  # Reserved, Type (1=ICO), Count
    
    # Calculate offset to image data
    # Header: 6 bytes
    # Directory entries: 16 bytes each
    data_offset = 6 + 16 * len(images_data)
    
    # Image directory
    for i, (size, data) in enumerate(images_data):
        width = size if size < 256 else 0
        height = size if size < 256 else 0
        colors = 0
        reserved = 0
        planes = 1
        bpp = 32
        data_size = len(data)
        
        f.write(struct.pack('<BBBBHHII', 
            width, height, colors, reserved,
            planes, bpp, data_size, data_offset))
        
        data_offset += data_size
    
    # Image data
    for size, data in images_data:
        f.write(data)

file_size = os.path.getsize(ico_path)
print("\n[OK] icon.ico generated: {} bytes".format(file_size))

# Verify
with open(ico_path, 'rb') as f:
    header = f.read(6)
    count = struct.unpack('<H', header[4:6])[0]
    print("  Contains {} images".format(count))
