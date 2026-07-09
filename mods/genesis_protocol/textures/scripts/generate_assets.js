#!/usr/bin/env node
/**
 * Convertit les SVG manuels (textures/svg/manual/) en PNG via sharp.
 * Usage: node generate_assets.js
 */

const fs = require("fs");
const path = require("path");

const BASE_DIR = path.resolve(__dirname, "..");
const MANUAL_DIR = path.join(BASE_DIR, "svg", "manual");
const PNG_DIR = BASE_DIR;

async function main() {
  if (!fs.existsSync(MANUAL_DIR)) {
    console.log("Aucun dossier manual/ trouvé.");
    return;
  }

  const buildingSizes = {
    genesis_capsule: [256, 256],
    genesis_capsule_t8: [256, 256],
    genesis_capsule_t1: [256, 256],
    genesis_capsule_t2: [256, 256],
    genesis_capsule_t3: [256, 256],
    genesis_capsule_t4: [256, 256],
    genesis_capsule_t5: [256, 256],
    genesis_capsule_t6: [256, 256],
    genesis_capsule_t7: [256, 256],
    deep_core_drill: [320, 320],
    scanner_array: [192, 192],
    assembly_crane: [192, 128],
    blast_furnace: [128, 128],
    chemical_lab: [128, 128],
    motor_foundry: [128, 128],
    water_pump: [128, 128],
    oil_pump: [128, 128],
    electronics_lab: [128, 128],
    nanite_assembler: [128, 128],
    bio_lab: [128, 128],
    tissue_cultivator: [128, 128],
    bio_printer: [128, 128],
  };

  try {
    const sharp = require("sharp");
    const files = fs.readdirSync(MANUAL_DIR).filter(f => f.endsWith("_base.svg"));
    if (files.length === 0) {
      console.log("Aucun SVG trouvé dans manual/.");
      return;
    }
    let ok = 0, fail = 0;
    for (const file of files) {
      const stem = file.replace("_base.svg", "");
      const svgPath = path.join(MANUAL_DIR, file);
      const pngPath = path.join(PNG_DIR, file.replace("_base.svg", "_base.png"));
      const size = buildingSizes[stem] || [64, 64];
      try {
        const svgBuf = fs.readFileSync(svgPath);
        await sharp(svgBuf).resize(size[0], size[1]).png().toFile(pngPath);
        ok++;
        console.log(`  ✓ ${path.basename(pngPath)} (${size[0]}x${size[1]})`);
      } catch (e) {
        console.log(`  ✗ ${file}: ${e.message}`);
        fail++;
      }
    }
    console.log(`\n✨ ${ok} PNG générés${fail ? `, ${fail} échecs` : ""}`);
  } catch {
    console.log('\n⚠ sharp non disponible. Pour installer: npm install sharp');
  }
}

main();
