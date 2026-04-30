#!/usr/bin/env python3
"""Generate wTools icon"""

import os

# Create 1024x1024 image with gradient and icon
try:
    from PIL import Image, ImageDraw
    
    output_dir = r'e:\Claw\wtools\src-tauri\icons'
    os.makedirs(output_dir, exist_ok=True)
    
    # Create 1024x1024 image
    img = Image.new('RGBA', (1024, 1024), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Draw rounded rectangle with gradient approximation
    corner_radius = 224
    
    # Draw base rectangle (blue gradient approximation - solid for now)
    draw.rounded_rectangle([0, 0, 1024, 1024], radius=corner_radius, fill=(0, 82, 217))
    
    # Add lighter gradient overlay
    for y in range(0, 1024, 4):
        alpha = int(30 * (1 - y / 1024))  # Fade from top
        if alpha > 0:
            draw.line([(0, y), (1024, y)], fill=(255, 255, 255, alpha))
    
    # Draw search icon (magnifying glass)
    # Circle
    center_x, center_y = 472, 472
    radius = 136
    # Draw circle outline
    for angle in range(0, 360, 1):
        import math
        x1 = center_x + radius * math.cos(math.radians(angle))
        y1 = center_y + radius * math.sin(math.radians(angle))
        x2 = center_x + (radius + 24) * math.cos(math.radians(angle))
        y2 = center_y + (radius + 24) * math.sin(math.radians(angle))
        draw.line([(x1, y1), (x2, y2)], fill=(255, 255, 255), width=1)
    
    # Draw handle
    handle_start = (center_x + int(radius * 0.7), center_y + int(radius * 0.7))
    handle_end = (center_x + int(radius * 1.4), center_y + int(radius * 1.4))
    draw.line([handle_start, handle_end], fill=(255, 255, 255), width=48)
    
    # Draw magnifier circle as thick ellipse
    draw.ellipse([center_x - radius - 24, center_y - radius - 24, 
                  center_x + radius + 24, center_y + radius + 24], 
                 outline=(255, 255, 255), width=48)
    
    # Save PNG
    png_path = os.path.join(output_dir, 'icon.png')
    img.save(png_path)
    print("[OK] PNG icon generated: {}".format(png_path))
    
    # Generate multiple sizes for ICO
    sizes = [16, 32, 48, 64, 128, 256]
    ico_images = []
    for size in sizes:
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        ico_images.append(resized)
    
    # Save ICO
    ico_path = os.path.join(output_dir, 'icon.ico')
    ico_images[0].save(ico_path, format='ICO', sizes=[(s, s) for s in sizes], append_images=ico_images[1:])
    print("[OK] ICO icon generated: {}".format(ico_path))
    
    # Also save 32x32 icon for system tray
    tray_icon = img.resize((32, 32), Image.Resampling.LANCZOS)
    tray_path = os.path.join(output_dir, '32x32.png')
    tray_icon.save(tray_path)
    print("[OK] Tray icon generated: {}".format(tray_path))
    
    print("\n[Done] All icons generated successfully!")
    print("Output directory: {}".format(output_dir))
    
except ImportError as e:
    print("Error: PIL (Pillow) is not available: {}".format(e))
    print("\nPlease install Pillow:")
    print("  pip install Pillow")
    print("\nOr use the SVG file with an online converter:")
    print("  SVG: e:/Claw/wtools/design/icon.svg")
