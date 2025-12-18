import { readFileSync } from 'fs';
import sharp from 'sharp';

async function generateIcons() {
  const svgBuffer = readFileSync('src-tauri/icons/icon.svg');

  const sizes = [
    { size: 32, name: '32x32.png' },
    { size: 128, name: '128x128.png' },
    { size: 256, name: '128x128@2x.png' }
  ];

  for (const { size, name } of sizes) {
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(`src-tauri/icons/${name}`);
    console.log(`Generated ${name}`);
  }
}

generateIcons().catch(console.error);

