#!/bin/bash
set -e

# Usage: ./generate-icns.sh <source_image> <output_icns_file>
SOURCE_IMAGE="$1"
OUTPUT_ICNS="$2"

if [ -z "$SOURCE_IMAGE" ] || [ -z "$OUTPUT_ICNS" ]; then
  echo "Usage: $0 <path_to_source_image> <path_to_output_icns>"
  exit 1
fi

ICONSET_DIR="icon.iconset"
echo "Creating temporary iconset directory..."
rm -rf "${ICONSET_DIR}"
mkdir -p "${ICONSET_DIR}"

echo "Resizing images to PNG format using sips..."
sips -s format png -z 16 16     "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_16x16.png" > /dev/null
sips -s format png -z 32 32     "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_16x16@2x.png" > /dev/null
sips -s format png -z 32 32     "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_32x32.png" > /dev/null
sips -s format png -z 64 64     "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_32x32@2x.png" > /dev/null
sips -s format png -z 128 128   "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_128x128.png" > /dev/null
sips -s format png -z 256 256   "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_128x128@2x.png" > /dev/null
sips -s format png -z 256 256   "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_256x256.png" > /dev/null
sips -s format png -z 512 512   "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_256x256@2x.png" > /dev/null
sips -s format png -z 512 512   "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_512x512.png" > /dev/null
sips -s format png -z 1024 1024 "${SOURCE_IMAGE}" --out "${ICONSET_DIR}/icon_512x512@2x.png" > /dev/null

echo "Compiling iconset to .icns using iconutil..."
iconutil -c icns "${ICONSET_DIR}" -o "${OUTPUT_ICNS}"

echo "Cleaning up temporary iconset..."
rm -rf "${ICONSET_DIR}"

echo "Successfully generated: ${OUTPUT_ICNS}"
