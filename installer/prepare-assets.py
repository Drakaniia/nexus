#!/usr/bin/env python3
"""
Nexus Asset Preparation Script (Python version)
Converts PNG logo to ICO format for installer
"""

import os
import sys
from pathlib import Path

try:
    from PIL import Image
    PIL_AVAILABLE = True
except ImportError:
    PIL_AVAILABLE = False

def convert_png_to_ico(png_path, ico_path):
    """Convert PNG to ICO with multiple sizes"""
    if not PIL_AVAILABLE:
        print("ERROR: PIL/Pillow not available. Install with: pip install pillow")
        return False

    try:
        # Open the image
        img = Image.open(png_path)

        # Convert to RGBA if not already
        if img.mode != 'RGBA':
            img = img.convert('RGBA')

        # Create ICO with multiple sizes
        sizes = [(256, 256), (128, 128), (64, 64), (48, 48), (32, 32), (16, 16)]
        img.save(ico_path, format='ICO', sizes=sizes)

        print(f"Successfully converted {png_path} to {ico_path}")
        return True

    except Exception as e:
        print(f"ERROR: Failed to convert image: {e}")
        return False

def main():
    print("=" * 60)
    print("Nexus Asset Preparation Script")
    print("=" * 60)
    print()

    # Check if icon.png exists
    assets_dir = Path("installerassets")
    png_path = assets_dir / "icon.png"
    ico_path = assets_dir / "icon.ico"

    if not png_path.exists():
        print(f"ERROR: {png_path} not found!")
        print("Please ensure the logo file is in the installerassets directory.")
        sys.exit(1)

    print(f"Found logo: {png_path}")
    print()

    if not PIL_AVAILABLE:
        print("PIL/Pillow not available.")
        print("To install: pip install pillow")
        print()
        print("Manual conversion options:")
        print("- Use online converter: https://convertio.co/png-ico/")
        print("- Use GIMP or Photoshop to export as ICO")
        print("- Save multiple sizes: 256x256, 128x128, 64x64, 48x48, 32x32, 16x16")
        sys.exit(1)

    print("Converting PNG to ICO...")
    if convert_png_to_ico(png_path, ico_path):
        print()
        print("=" * 60)
        print("Asset preparation complete!")
        print("=" * 60)
        print(f"Created: {ico_path}")
        print()
        print("You can now build the installer with build.bat")
    else:
        sys.exit(1)

if __name__ == "__main__":
    main()