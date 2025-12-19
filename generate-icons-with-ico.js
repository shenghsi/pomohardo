import { readFileSync } from 'fs';
import sharp from 'sharp';

const svgPath = 'src-tauri/icons/icon.svg';
const sizes = [
  { size: 32, name: '32x32.png' },
  { size: 128, name: '128x128.png' },
  { size: 256, name: '128x128@2x.png' }
];

async function generateIcons() {
  console.log('Generating icons from SVG...');
  
  try {
    const svgBuffer = readFileSync(svgPath);
    
    // Generate PNG files
    await Promise.all(sizes.map(({ size, name }) => 
      sharp(svgBuffer)
        .resize(size, size)
        .png()
        .toFile(`src-tauri/icons/${name}`)
        .then(() => console.log(`✓ Generated ${name}`))
    ));
    
    // Generate ICO file for Windows
    console.log('Generating icon.ico for Windows...');
    await sharp(svgBuffer)
      .resize(256, 256)
      .png()
      .toFile('src-tauri/icons/icon.ico')
      .then(() => console.log('✓ Generated icon.ico'));
    
    console.log('All icons generated successfully!');
  } catch (err) {
    console.error('Error generating icons:', err);
    if (err.code === 'MODULE_NOT_FOUND') {
      console.error('sharp not installed. Please run: npm install');
    }
    process.exit(1);
  }
}

generateIcons();