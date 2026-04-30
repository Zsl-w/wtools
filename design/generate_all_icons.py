#!/usr/bin/env python3
"""Generate all icon sizes from 1024x1024 source"""

import os
from PIL import Image

# Source icon
source_path = r'e:\Claw\wtools\src-tauri\icons\icon.png'
output_dir = r'e:\Claw\wtools\src-tauri\icons'

# Load source image
source = Image.open(source_path)
if source.size != (1024, 1024):
    print("Warning: Source icon is not 1024x1024, resizing...")
    source = source.resize((1024, 1024), Image.Resampling.LANCZOS)

# Required sizes for Tauri
sizes = [16, 32, 128, 256]

print("Generating icon sizes...")

# Generate individual PNG files
for size in sizes:
    resized = source.resize((size, size), Image.Resampling.LANCZOS)
    output_path = os.path.join(output_dir, '{}x{}.png'.format(size, size))
    resized.save(output_path, 'PNG')
    print("  [OK] {}x{}.png".format(size, size))

# Also ensure 32x32.png exists for tray
if 32 in sizes:
    tray_path = os.path.join(output_dir, '32x32.png')
    if not os.path.exists(tray_path):
        os.link(os.path.join(output_dir, '32x32.png'), tray_path)

# Generate ICO file with multiple sizes
ico_sizes = [16, 32, 48, 64, 128, 256]
ico_images = []

print("\nGenerating icon.ico...")
for size in ico_sizes:
    resized = source.resize((size, size), Image.Resampling.LANCZOS)
    ico_images.append(resized)

# Save ICO - first image is the base, others are appended
ico_path = os.path.join(output_dir, 'icon.ico')
ico_images[0].save(
    ico_path,
    format='ICO',
    sizes=[(s, s) for s in ico_sizes],
    append_images=ico_images[1:]
)
print("  [OK] icon.ico (contains: {} sizes)".format(len(ico_sizes)))

# Generate StoreLogo for Windows
store_size = 50
store_img = source.resize((store_size, store_size), Image.Resampling.LANCZOS)
store_path = os.path.join(output_dir, 'StoreLogo.png')
store_img.save(store_path, 'PNG')
print("  [OK] StoreLogo.png (50x50)")

print("\n[Done] All icons generated successfully!")
print("Files in {}:".format(output_dir))
for f in sorted(os.listdir(output_dir)):
    filepath = os.path.join(output_dir, f)
    size = os.path.getsize(filepath)
    print("  - {} ({} bytes)".format(f, size))
