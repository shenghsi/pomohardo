import { readFileSync, writeFileSync } from 'fs';
import sharp from 'sharp';

const svgPath = 'src-tauri/icons/icon.svg';

async function generateProperIco() {
  console.log('Generating proper ICO file for Windows...');
  
  try {
    const svgBuffer = readFileSync(svgPath);
    
    // Generate multiple sizes for ICO file (16, 32, 48, 256)
    const sizes = [16, 32, 48, 256];
    const pngBuffers = [];
    
    for (const size of sizes) {
      const pngBuffer = await sharp(svgBuffer)
        .resize(size, size)
        .png()
        .toBuffer();
      pngBuffers.push({ size, buffer: pngBuffer });
      console.log(`✓ Generated ${size}x${size} PNG buffer`);
    }
    
    // Create ICO file manually
    const icoBuffer = createIcoFile(pngBuffers);
    writeFileSync('src-tauri/icons/icon.ico', icoBuffer);
    
    console.log('✓ Generated proper icon.ico file');
  } catch (err) {
    console.error('Error generating ICO:', err);
    process.exit(1);
  }
}

function createIcoFile(pngBuffers) {
  // ICO file format:
  // Header (6 bytes): 0x00 0x00 0x01 0x00 [count] 0x00
  // Directory entries (16 bytes each)
  // PNG data
  
  const count = pngBuffers.length;
  const headerSize = 6;
  const directorySize = count * 16;
  
  let totalSize = headerSize + directorySize;
  const directories = [];
  
  // Calculate offsets and prepare directory entries
  for (let i = 0; i < count; i++) {
    const { size, buffer } = pngBuffers[i];
    const offset = totalSize;
    totalSize += buffer.length;
    
    directories.push({
      width: size === 256 ? 0 : size,  // 0 means 256 in ICO format
      height: size === 256 ? 0 : size,
      colorCount: 0,
      reserved: 0,
      planes: 1,
      bitCount: 32,
      bytesInRes: buffer.length,
      imageOffset: offset,
      buffer: buffer
    });
  }
  
  // Create the ICO buffer
  const icoBuffer = Buffer.alloc(totalSize);
  let pos = 0;
  
  // Write header
  icoBuffer.writeUInt16LE(0, pos); pos += 2;  // Reserved
  icoBuffer.writeUInt16LE(1, pos); pos += 2;  // Type (1 = ICO)
  icoBuffer.writeUInt16LE(count, pos); pos += 2;  // Count
  
  // Write directory entries
  for (const dir of directories) {
    icoBuffer.writeUInt8(dir.width, pos); pos += 1;
    icoBuffer.writeUInt8(dir.height, pos); pos += 1;
    icoBuffer.writeUInt8(dir.colorCount, pos); pos += 1;
    icoBuffer.writeUInt8(dir.reserved, pos); pos += 1;
    icoBuffer.writeUInt16LE(dir.planes, pos); pos += 2;
    icoBuffer.writeUInt16LE(dir.bitCount, pos); pos += 2;
    icoBuffer.writeUInt32LE(dir.bytesInRes, pos); pos += 4;
    icoBuffer.writeUInt32LE(dir.imageOffset, pos); pos += 4;
  }
  
  // Write PNG data
  for (const dir of directories) {
    dir.buffer.copy(icoBuffer, pos);
    pos += dir.buffer.length;
  }
  
  return icoBuffer;
}

generateProperIco();