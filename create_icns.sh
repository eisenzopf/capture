#!/bin/bash

# Create temporary iconset directory
mkdir -p AppIcon.iconset

# Generate PNG files at different sizes
magick -background none icons/AppIcon.svg -resize 16x16 AppIcon.iconset/icon_16x16.png
magick -background none icons/AppIcon.svg -resize 32x32 AppIcon.iconset/icon_16x16@2x.png
magick -background none icons/AppIcon.svg -resize 32x32 AppIcon.iconset/icon_32x32.png
magick -background none icons/AppIcon.svg -resize 64x64 AppIcon.iconset/icon_32x32@2x.png
magick -background none icons/AppIcon.svg -resize 128x128 AppIcon.iconset/icon_128x128.png
magick -background none icons/AppIcon.svg -resize 256x256 AppIcon.iconset/icon_128x128@2x.png
magick -background none icons/AppIcon.svg -resize 256x256 AppIcon.iconset/icon_256x256.png
magick -background none icons/AppIcon.svg -resize 512x512 AppIcon.iconset/icon_256x256@2x.png
magick -background none icons/AppIcon.svg -resize 512x512 AppIcon.iconset/icon_512x512.png
magick -background none icons/AppIcon.svg -resize 1024x1024 AppIcon.iconset/icon_512x512@2x.png

# Create .icns file
iconutil -c icns AppIcon.iconset

# Move the file to the correct location
mv AppIcon.icns icons/AppIcon.icns

# Clean up
rm -rf AppIcon.iconset
