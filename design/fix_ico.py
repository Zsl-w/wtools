#!/usr/bin/env python3
"""Fix ICO file generation"""

import os
from PIL import Image

source_path = r'e:\Claw\wtools\src-tauri\icons\icon.png'
output_dir = r'e:\Claw\wtools\src-tauri\icons'

# Load source
source = Image.open(source_path)
if source.mode != 'RGBA':
    source = source.convert('RGBA')

# ICO needs specific sizes, generate all
sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]

print("Generating ICO with sizes:", sizes)

# Create list of (image, size) tuples for ICO
ico_data = []
for size in sizes:
    img = source.resize((size, size), Image.Resampling.LANCZOS)
    # Ensure RGBA mode for transparency
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    ico_data.append(img)

# Save ICO file properly
ico_path = os.path.join(output_dir, 'icon.ico')

# PIL's ICO save: first image is saved, append_images adds more sizes
ico_data[0].save(
    ico_path,
    format='ICO',
    sizes=[(s, s) for s in sizes],
    append_images=ico_data[1:]
)

file_size = os.path.getsize(ico_path)
print("[OK] icon.ico generated: {} bytes".format(file_size))

# Verify the file
with open(ico_path, 'rb') as f:
    header = f.read(6)
    # ICO header: Reserved (2 bytes), Type (2 bytes), Count (2 bytes)
    reserved = int.from_bytes(header[0:2], 'little')
    img_type = int.from_bytes(header[2:4], 'little')
    count = int.from_bytes(header[4:6], 'little')
    print("  ICO Header: Reserved={}, Type={}, Image Count={}".format(reserved, img_type, count))
