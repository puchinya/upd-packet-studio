import os
import sys
from PIL import Image, ImageFilter

def main():
    # We need to extract the original 1024x1024 image.
    # Since we deleted temp_icon.iconset, we will extract it again from icon.icns.
    # Let's check if temp_icon.iconset exists, if not we will recreate it.
    source_path = "temp_icon.iconset/icon_512x512@2x.png"
    
    if not os.path.exists(source_path):
        print("Extracting images from icon.icns...")
        os.system("iconutil -c iconset icon.icns -o temp_icon.iconset")
        
    if not os.path.exists(source_path):
        print(f"Error: Source image {source_path} could not be extracted.")
        sys.exit(1)
        
    print(f"Loading source image: {source_path}")
    src_img = Image.open(source_path).convert("RGBA")
    
    # Canvas Size: 1024x1024
    width, height = 1024, 1024
    # macOS Big Sur app icon content size: 824x824 (padding: 100px on each side)
    content_size = 824
    padding = (width - content_size) // 2 # 100px
    
    print("Generating high-quality Squircle (Superellipse) mask...")
    # Generate at 2x resolution for antialiasing, then downsample
    mask_scale = 2
    mask_w = content_size * mask_scale
    mask_h = content_size * mask_scale
    
    mask_large = Image.new("L", (mask_w, mask_h), 0)
    pixels = mask_large.load()
    
    half_w = mask_w / 2.0
    half_h = mask_h / 2.0
    
    # Superellipse exponent. n=4.7 matches macOS Big Sur squircle curve closely.
    n = 4.7
    a = half_w
    b = half_h
    eps = 0.004
    
    for y_idx in range(mask_h):
        y_val = (y_idx + 0.5) - half_h
        for x_idx in range(mask_w):
            x_val = (x_idx + 0.5) - half_w
            
            # Superellipse equation (|x|/a)^n + (|y|/b)^n
            val = (abs(x_val) / a)**n + (abs(y_val) / b)**n
            
            # Smooth step edge for antialiasing
            if val <= 1.0 - eps:
                pixels[x_idx, y_idx] = 255
            elif val >= 1.0 + eps:
                pixels[x_idx, y_idx] = 0
            else:
                alpha = int((1.0 + eps - val) / (2 * eps) * 255)
                pixels[x_idx, y_idx] = max(0, min(255, alpha))
    
    mask = mask_large.resize((content_size, content_size), Image.Resampling.LANCZOS)
    
    print("Resizing source image...")
    src_resized = src_img.resize((content_size, content_size), Image.Resampling.LANCZOS)
    
    # Apply Squircle mask to resized source
    icon_content = Image.new("RGBA", (content_size, content_size), (0, 0, 0, 0))
    icon_content.paste(src_resized, (0, 0), mask)
    
    print("Generating macOS-style drop shadow...")
    # We place the mask on a 1024x1024 canvas, shift it down, and apply Gaussian blur.
    shadow_mask = Image.new("L", (width, height), 0)
    shadow_mask.paste(mask, (padding, padding))
    
    # The shadow is dark black with ~24% opacity
    shadow_color = Image.new("RGBA", (width, height), (0, 0, 0, 60))
    shadow_img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    shadow_img = Image.composite(shadow_color, shadow_img, shadow_mask)
    
    # Apply strong blur to match macOS premium soft shadow
    shadow_blur = 18
    shadow_blurred = shadow_img.filter(ImageFilter.GaussianBlur(shadow_blur))
    
    # Shift shadow down by 14 pixels
    shadow_offset_y = 14
    shadow_final = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    shadow_final.paste(shadow_blurred, (0, shadow_offset_y))
    
    print("Compositing layers...")
    # Composite: transparent canvas -> shadow -> icon content
    final_img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    final_img.paste(shadow_final, (0, 0))
    final_img.paste(icon_content, (padding, padding), icon_content)
    
    # Save the 1024x1024 PNG for reference (optional, but good for building icns)
    output_png = "icon_macos.png"
    print(f"Saving macOS PNG icon to: {output_png}")
    final_img.save(output_png)
    
    # Generate the 64x64 raw RGBA byte file for Rust integration
    rgba_output_path = "src/icon.rgba"
    print(f"Generating 64x64 raw RGBA file at: {rgba_output_path}")
    rgba_img = final_img.resize((64, 64), Image.Resampling.LANCZOS)
    rgba_bytes = rgba_img.tobytes()
    
    # Verify exact size: 64 * 64 * 4 = 16384 bytes
    assert len(rgba_bytes) == 16384, f"Unexpected byte length: {len(rgba_bytes)}"
    
    with open(rgba_output_path, "wb") as f:
        f.write(rgba_bytes)
        
    # Clean up temp_icon.iconset
    if os.path.exists("temp_icon.iconset"):
        print("Cleaning up temporary iconset...")
        os.system("rm -rf temp_icon.iconset")
        
    print("Done!")

if __name__ == "__main__":
    main()
