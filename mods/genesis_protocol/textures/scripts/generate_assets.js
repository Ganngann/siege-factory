#!/usr/bin/env node
/**
 * Génère les assets SVG isométriques pour le mod Genesis Protocol,
 * puis les convertit en PNG 64×64 via sharp si disponible.
 * Usage: node generate_assets.js
 */

const fs = require("fs");
const path = require("path");

const BASE_DIR = path.resolve(__dirname, "..");
const SVG_DIR = path.join(BASE_DIR, "svg");
const PNG_DIR = BASE_DIR;

fs.mkdirSync(SVG_DIR, { recursive: true });
fs.mkdirSync(PNG_DIR, { recursive: true });

// ── Helpers ──

const darken = (hex, factor) => {
  const [r, g, b] = hex.match(/[A-Fa-f0-9]{2}/g).map((c) => parseInt(c, 16));
  return `#${[r, g, b].map((c) => Math.floor(c * factor).toString(16).padStart(2, "0")).join("")}`;
};

const lighten = (hex, factor) => {
  const [r, g, b] = hex.match(/[A-Fa-f0-9]{2}/g).map((c) => parseInt(c, 16));
  return `#${[r, g, b]
    .map((c) => Math.min(255, Math.floor(c + (255 - c) * factor)).toString(16).padStart(2, "0"))
    .join("")}`;
};

// ── Building definitions ──

const BUILDINGS = [
  ["workbench", "#8B5E3C", 1, 1],
  ["campfire", "#FF6622", 1, 1],
  ["furnace", "#884422", 1, 1],
  ["anvil", "#666666", 1, 1],
  ["burner_generator", "#DD6622", 1, 1],
  ["manual_miner", "#AA7733", 1, 1],
  ["water_pump", "#3399DD", 1, 1],
  ["steam_generator", "#CCDDEE", 1, 1],
  ["blast_furnace", "#AA4422", 1, 1],
  ["gear_press", "#887766", 1, 1],
  ["belt", "#808080", 1, 1],
  ["splitter", "#AAAA00", 1, 1],
  ["electric_generator", "#FFAA33", 1, 1],
  ["power_pole", "#888888", 1, 1],
  ["assembler", "#4D99CC", 1, 1],
  ["chemical_lab", "#664488", 2, 2],
  ["oil_pump", "#444455", 1, 1],
  ["storage_chest", "#CC9900", 1, 1],
  ["motor_foundry", "#AA8844", 2, 1],
  ["battery_station", "#33AA33", 1, 1],
  ["electronics_lab", "#33AA88", 2, 2],
  ["assembly_crane", "#3377AA", 2, 1],
  ["aerial_belt", "#88AACC", 1, 1],
  ["sorter", "#66AA66", 1, 1],
  ["nanite_assembler", "#44DDBB", 2, 2],
  ["deep_core_drill", "#664433", 3, 2],
  ["compactor", "#AAAA77", 1, 1],
  ["high_speed_belt", "#CC8844", 1, 1],
  ["excavation_rig", "#775533", 2, 2],
  ["bio_lab", "#66BB6A", 2, 2],
  ["tissue_cultivator", "#AB47BC", 2, 2],
  ["synthesizer", "#FF7043", 1, 1],
  ["scanner_array", "#42A5F5", 2, 2],
  ["bio_printer", "#4DB6AC", 2, 2],
];

const CAPSULE = [
  ["genesis_capsule", "#334455"],
  ["genesis_capsule_t0", "#445566"],
  ["genesis_capsule_t1", "#5577AA"],
  ["genesis_capsule_t2", "#6699CC"],
  ["genesis_capsule_t3", "#88BBDD"],
  ["genesis_capsule_t4", "#AACCCC"],
  ["genesis_capsule_t5", "#CCDDDD"],
  ["genesis_capsule_t6", "#DDEEEE"],
  ["genesis_capsule_t7", "#EEFFFF"],
];

const RESOURCES = {
  stone_pickaxe: "#8B7355",
  stone_axe: "#8B5E3C",
  stone_blade: "#887766",
  hammer: "#AA7733",
  scrap_metal: "#887766",
  clay: "#C4A882",
  plant_fiber: "#5A8C3C",
  planks: "#A0724A",
  stone_brick: "#887766",
  ceramic: "#C4956A",
  rope: "#8B7355",
  iron_parts: "#999999",
  iron_ingot: "#AAAAAA",
  copper_ingot: "#CC8844",
  water: "#3399DD",
  steam: "#CCDDEE",
  acid: "#88FF44",
  silicon: "#AACCDD",
  processor: "#33BB33",
  alloy: "#8888AA",
  organic_compound: "#66BB6A",
  enzyme: "#AB47BC",
  protein: "#FF7043",
  synthetic_blood: "#EF5350",
  neural_map: "#42A5F5",
  stem_cells: "#FFD54F",
  bio_mass: "#4DB6AC",
  neural_interface: "#7E57C2",
  synthetic_heart: "#EF5350",
  genome_sequence: "#66BB6A",
};

// ── SVG generators (top-down view) ──

function tw(n) {
  return n * 64;
}
function th(n) {
  return n * 64;
}

const TILE = 64;

function makeBuildingSVG(stem, color, w, h) {
  // Bounding box centered in 64x64, scaled by footprint
  const bw = (w / 3) * TILE * 0.75; // base width in px
  const bh = (h / 3) * TILE * 0.75; // base height in px
  const cx = TILE / 2;
  const cy = TILE / 2;
  const rx = bw / 2;
  const ry = bh / 2;
  const lx = cx - rx;
  const ly = cy - ry;

  let extra = "";

  const dark = darken(color, 0.7);

  if (stem === "campfire") {
    extra = `
    <ellipse cx="${cx}" cy="${cy}" rx="${rx}" ry="${ry}" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="8" fill="#FFDD44" opacity="0.9"/>
    <circle cx="${cx}" cy="${cy - 3}" r="5" fill="#FF8800" opacity="0.8"/>
    <circle cx="${cx + 3}" cy="${cy + 2}" r="4" fill="#FF4400" opacity="0.6"/>
    <rect x="${cx - 9}" y="${cy + 6}" width="18" height="4" rx="1" fill="${darken(color, 0.5)}"/>`;
  } else if (stem === "workbench") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${lighten(color, 0.1)}" stroke="#333" stroke-width="0.5"/>
    <line x1="${cx - 8}" y1="${cy - 4}" x2="${cx + 8}" y2="${cy - 4}" stroke="${dark}" stroke-width="1"/>
    <line x1="${cx - 8}" y1="${cy + 4}" x2="${cx + 8}" y2="${cy + 4}" stroke="${dark}" stroke-width="1"/>`;
  } else if (stem === "furnace") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <ellipse cx="${cx}" cy="${cy}" rx="5" ry="5" fill="#FF4400" opacity="0.6"/>`;
  } else if (stem === "anvil") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <polygon points="${cx - 8},${cy + 6} ${cx},${cy - 8} ${cx + 8},${cy + 6}" fill="${dark}" stroke="#333" stroke-width="0.5"/>`;
  } else if (["belt", "high_speed_belt", "aerial_belt"].includes(stem)) {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="1" fill="${color}" stroke="#333" stroke-width="1"/>
    <line x1="${lx + 4}" y1="${cy}" x2="${lx + bw - 4}" y2="${cy}" stroke="${dark}" stroke-width="3"/>
    <circle cx="${lx + 6}" cy="${cy}" r="3" fill="${dark}"/>
    <circle cx="${lx + bw - 6}" cy="${cy}" r="3" fill="${dark}"/>`;
    if (stem === "high_speed_belt") {
      extra += `
    <line x1="${lx + 4}" y1="${cy - 4}" x2="${lx + bw - 4}" y2="${cy - 4}" stroke="#FFF" stroke-width="0.5" opacity="0.5"/>`;
    }
    if (stem === "aerial_belt") {
      extra += `
    <rect x="${lx}" y="${ly - 3}" width="${bw}" height="3" rx="1" fill="${dark}" opacity="0.3"/>`;
    }
  } else if (stem === "splitter") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="5" fill="${darken(color, 0.6)}" stroke="#333" stroke-width="0.5"/>
    <line x1="${cx}" y1="${ly + 4}" x2="${cx}" y2="${ly + bh - 4}" stroke="#333" stroke-width="1.5"/>
    <line x1="${lx + 4}" y1="${cy}" x2="${lx + bw - 4}" y2="${cy}" stroke="#333" stroke-width="1.5"/>`;
  } else if (stem === "sorter") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <polygon points="${cx - 6},${cy - 4} ${cx + 6},${cy} ${cx - 6},${cy + 4}" fill="${dark}" stroke="#333" stroke-width="0.5"/>`;
  } else if (stem === "power_pole") {
    extra = `
    <circle cx="${cx}" cy="${cy}" r="5" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="2.5" fill="#FFAA00" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 1}" y="${cy - 10}" width="2" height="6" fill="${color}"/>
    <rect x="${cx - 4}" y="${cy - 10}" width="8" height="2" fill="${color}"/>`;
  } else if (stem === "burner_generator") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="4" fill="#FF8800" opacity="0.5"/>`;
  } else if (stem === "manual_miner") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <polygon points="${cx},${ly + 4} ${lx + 4},${ly + bh - 4} ${lx + bw - 4},${ly + bh - 4}" fill="${darken(color, 0.6)}" stroke="#333" stroke-width="0.5"/>`;
  } else if (stem === "water_pump") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="6" fill="${lighten(color, 0.3)}" stroke="#333" stroke-width="0.5"/>
    <path d="M${cx - 3} ${cy + 2} Q${cx} ${cy - 6} ${cx + 3} ${cy + 2}" fill="none" stroke="#FFF" stroke-width="1.5" opacity="0.8"/>`;
  } else if (stem === "steam_generator") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${darken(color, 0.9)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="5" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="1.5"/>
    <circle cx="${cx}" cy="${cy}" r="2" fill="#FFF" opacity="0.5"/>`;
  } else if (stem === "blast_furnace") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="2" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 3}" y="${ly + 2}" width="6" height="4" fill="#FF6600" opacity="0.7"/>`;
  } else if (stem === "gear_press") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="6" fill="${darken(color, 0.6)}" stroke="#333" stroke-width="0.5"/>
    <line x1="${cx}" y1="${cy - 4}" x2="${cx}" y2="${cy + 4}" stroke="#FFF" stroke-width="1.5"/>`;
  } else if (stem === "electric_generator") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <text x="${cx}" y="${cy + 2}" text-anchor="middle" font-size="10" fill="#333" font-weight="bold">⚡</text>`;
  } else if (stem === "assembler") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 5}" y="${ly + 5}" width="${bw - 10}" height="${bh - 10}" rx="2" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="3" fill="#FFF" opacity="0.6"/>`;
  } else if (stem === "chemical_lab") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 5}" y="${ly + 5}" width="${bw - 10}" height="${bh - 10}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx - 6}" cy="${cy - 4}" r="4" fill="#88FF44" opacity="0.5"/>
    <circle cx="${cx + 6}" cy="${cy + 4}" r="4" fill="#44FF88" opacity="0.5"/>`;
  } else if (stem === "oil_pump") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="5" fill="#333" opacity="0.4"/>
    <line x1="${cx}" y1="${ly + 2}" x2="${cx}" y2="${ly + bh - 2}" stroke="#555" stroke-width="2"/>`;
  } else if (stem === "storage_chest") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 5}" y="${ly + 5}" width="${bw - 10}" height="${bh - 10}" rx="2" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 6}" y="${cy - 2}" width="12" height="4" rx="1" fill="#FFD700" opacity="0.8"/>`;
  } else if (stem === "motor_foundry") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="3" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx - 6}" cy="${cy}" r="4" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/>
    <circle cx="${cx + 6}" cy="${cy}" r="4" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/>`;
  } else if (stem === "battery_station") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 5}" y="${ly + 5}" width="${bw - 10}" height="${bh - 10}" rx="2" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 3}" y="${cy - 6}" width="6" height="12" rx="1" fill="#FFF" opacity="0.3"/>
    <line x1="${cx - 2}" y1="${cy - 4}" x2="${cx + 2}" y2="${cy - 4}" stroke="#FFF" stroke-width="0.5"/>
    <line x1="${cx - 2}" y1="${cy}" x2="${cx + 2}" y2="${cy}" stroke="#FFF" stroke-width="0.5"/>
    <line x1="${cx - 2}" y1="${cy + 4}" x2="${cx + 2}" y2="${cy + 4}" stroke="#FFF" stroke-width="0.5"/>`;
  } else if (stem === "electronics_lab") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 6}" y="${cy - 5}" width="12" height="10" rx="1" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/>`;
  } else if (stem === "assembly_crane") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="2" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <line x1="${cx}" y1="${ly + 2}" x2="${cx}" y2="${ly + bh - 2}" stroke="${lighten(color, 0.3)}" stroke-width="2"/>
    <line x1="${lx + 2}" y1="${ly + bh/2}" x2="${lx + bw - 2}" y2="${ly + bh/2}" stroke="${lighten(color, 0.3)}" stroke-width="1.5"/>`;
  } else if (stem === "nanite_assembler") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="6" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="2" fill="#FFF" opacity="0.8"/>
    <circle cx="${cx - 5}" cy="${cy - 4}" r="1.5" fill="#FFF" opacity="0.5"/>
    <circle cx="${cx + 5}" cy="${cy + 4}" r="1.5" fill="#FFF" opacity="0.5"/>`;
  } else if (stem === "deep_core_drill") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="2" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <ellipse cx="${cx}" cy="${cy}" rx="6" ry="4" fill="#444" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 2}" y="${ly + 2}" width="4" height="${bh - 4}" fill="#555"/>`;
  } else if (stem === "compactor") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="2" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="1" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <line x1="${lx + 6}" y1="${ly + 6}" x2="${lx + bw - 6}" y2="${ly + bh - 6}" stroke="${lighten(color, 0.2)}" stroke-width="1.5"/>`;
  } else if (stem === "excavation_rig") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="3" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <polygon points="${cx},${ly + 3} ${lx + 5},${ly + bh - 5} ${lx + bw - 5},${ly + bh - 5}" fill="${darken(color, 0.5)}" stroke="#333" stroke-width="0.3"/>`;
  } else if (stem === "bio_lab") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="6" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="5" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="5" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="1"/>
    <circle cx="${cx}" cy="${cy}" r="2" fill="#FFF" opacity="0.6"/>
    <path d="M${cx - 4} ${cy + 3} Q${cx} ${cy + 7} ${cx + 4} ${cy + 3}" fill="none" stroke="#FFF" stroke-width="0.5" opacity="0.5"/>`;
  } else if (stem === "tissue_cultivator") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx}" cy="${cy}" r="6" fill="${lighten(color, 0.2)}" opacity="0.4" stroke="#333" stroke-width="0.3"/>
    <circle cx="${cx}" cy="${cy}" r="3" fill="${lighten(color, 0.3)}" opacity="0.6"/>`;
  } else if (stem === "synthesizer") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 4}" y="${ly + 4}" width="${bw - 8}" height="${bh - 8}" rx="2" fill="${darken(color, 0.7)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx - 4}" cy="${cy}" r="3" fill="#FFF" opacity="0.3"/>
    <circle cx="${cx + 4}" cy="${cy}" r="3" fill="#FFF" opacity="0.3"/>
    <line x1="${cx - 4}" y1="${cy}" x2="${cx + 4}" y2="${cy}" stroke="#FFF" stroke-width="1"/>`;
  } else if (stem === "scanner_array") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <circle cx="${cx - 5}" cy="${cy - 3}" r="4" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="1"/>
    <circle cx="${cx + 5}" cy="${cy + 3}" r="4" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="1"/>
    <circle cx="${cx - 5}" cy="${cy - 3}" r="1.5" fill="#FFF" opacity="0.5"/>
    <circle cx="${cx + 5}" cy="${cy + 3}" r="1.5" fill="#FFF" opacity="0.5"/>`;
  } else if (stem === "bio_printer") {
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="4" fill="${color}" stroke="#333" stroke-width="1"/>
    <rect x="${lx + 3}" y="${ly + 3}" width="${bw - 6}" height="${bh - 6}" rx="3" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.5"/>
    <rect x="${cx - 4}" y="${cy - 5}" width="8" height="10" rx="1" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/>
    <line x1="${cx - 3}" y1="${cy}" x2="${cx + 3}" y2="${cy}" stroke="#FFF" stroke-width="0.5"/>`;
  } else {
    // Generic building
    extra = `
    <rect x="${lx}" y="${ly}" width="${bw}" height="${bh}" rx="3" fill="${color}" stroke="#333" stroke-width="1"/>`;
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">
  ${extra}
</svg>`;
}

function makeCapsuleSVG(stem, color) {
  const cx = 32, cy = 32;

  let wc = "#112233";
  if (stem.includes("t7")) wc = "#EEFFFF";
  else if (stem.includes("t6")) wc = "#CCEEFF";
  else if (stem.includes("t5")) wc = "#88AACC";
  else if (stem.includes("t4")) wc = "#88BBFF";
  else if (stem.includes("t3")) wc = "#5577AA";
  else if (stem.includes("t2")) wc = "#446688";
  else if (stem.includes("t1")) wc = "#334466";
  else if (stem.includes("t0")) wc = "#223344";

  let extra = "";
  let shadow = `<ellipse cx="${cx}" cy="${cy + 4}" rx="22" ry="8" fill="rgba(0,0,0,0.15)"/>`;

  if (stem === "genesis_capsule") {
    // Dead / dormant state — cracked, debris, no lights
    extra = `
  <ellipse cx="${cx}" cy="${cy}" rx="20" ry="16" fill="${color}" stroke="#222" stroke-width="1.5"/>
  <ellipse cx="${cx}" cy="${cy}" rx="16" ry="12" fill="none" stroke="${darken(color, 0.5)}" stroke-width="0.5"/>
  <circle cx="${cx}" cy="${cy}" r="6" fill="#111" stroke="#222" stroke-width="1"/>
  <path d="M18 28 L20 32 L16 34" fill="none" stroke="#222" stroke-width="1" opacity="0.6"/>
  <path d="M46 26 L44 30 L48 29" fill="none" stroke="#222" stroke-width="0.8" opacity="0.5"/>
  <path d="M14 38 L18 40 L15 42" fill="none" stroke="#222" stroke-width="0.8" opacity="0.4"/>
  <circle cx="26" cy="22" r="1.5" fill="#222" opacity="0.3"/>
  <circle cx="40" cy="42" r="1" fill="#222" opacity="0.3"/>
  <rect x="10" y="44" width="6" height="3" rx="1" fill="#2A2A2A" opacity="0.4"/>
  <rect x="44" y="40" width="5" height="2" rx="1" fill="#2A2A2A" opacity="0.3"/>`;
  } else {
    let lights = "";
    if (!stem.includes("t0") && !stem.includes("t1")) {
      lights = `
    <circle cx="${cx - 10}" cy="${cy - 2}" r="2" fill="#33FF33" opacity="0.9"/>
    <circle cx="${cx + 10}" cy="${cy - 2}" r="2" fill="#FF3333" opacity="0.9"/>`;
    }

    extra = `
  <ellipse cx="${cx}" cy="${cy}" rx="20" ry="16" fill="${color}" stroke="#333" stroke-width="1.5"/>
  <ellipse cx="${cx}" cy="${cy}" rx="16" ry="12" fill="none" stroke="${darken(color, 0.7)}" stroke-width="0.5"/>
  <circle cx="${cx}" cy="${cy}" r="6" fill="${wc}" stroke="#333" stroke-width="1"/>
  ${lights}`;
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">${shadow}${extra}
</svg>`;
}

function makeResourceSVG(stem, color) {
  let inner = "";

  if (["water", "steam"].includes(stem)) {
    inner = `<path d="M32 8 Q32 24 32 28 Q32 36 24 36 Q16 36 16 28 Q16 20 24 16 L32 8Z" fill="${color}" stroke="#333" stroke-width="0.5"/>`;
    if (stem === "steam")
      inner += `<circle cx="38" cy="20" r="3" fill="rgba(255,255,255,0.4)"/><circle cx="44" cy="14" r="2" fill="rgba(255,255,255,0.3)"/>`;
  } else if (stem === "acid") {
    inner = `<path d="M32 10 Q32 26 32 30 Q32 38 24 38 Q16 38 16 30 Q16 22 24 18 L32 10Z" fill="${color}" stroke="#333" stroke-width="0.5"/><path d="M32 10 L36 18 L32 16 Z" fill="#FFF" opacity="0.3"/>`;
  } else if (stem.includes("fiber")) {
    inner = `<path d="M32 48 Q16 40 16 28 Q16 16 32 16 Q48 16 48 28 Q48 40 32 48Z" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="32" y1="48" x2="32" y2="16" stroke="#333" stroke-width="0.5"/><line x1="24" y1="40" x2="40" y2="24" stroke="#333" stroke-width="0.3" opacity="0.5"/>`;
  } else if (stem.includes("plank")) {
    inner = `<rect x="12" y="20" width="40" height="24" rx="2" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="20" y1="20" x2="20" y2="44" stroke="${darken(color, 0.7)}" stroke-width="0.5"/><line x1="44" y1="20" x2="44" y2="44" stroke="${darken(color, 0.7)}" stroke-width="0.5"/>`;
  } else if (stem.includes("rope")) {
    inner = `<ellipse cx="32" cy="32" rx="16" ry="10" fill="none" stroke="${color}" stroke-width="4"/><ellipse cx="32" cy="32" rx="10" ry="6" fill="none" stroke="${darken(color, 0.8)}" stroke-width="2"/>`;
  } else if (stem.includes("brick")) {
    inner = `<rect x="10" y="20" width="44" height="24" rx="1" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="32" y1="20" x2="32" y2="32" stroke="${darken(color, 0.7)}" stroke-width="0.5"/><line x1="10" y1="32" x2="54" y2="32" stroke="${darken(color, 0.7)}" stroke-width="0.5"/>`;
  } else if (stem === "ceramic") {
    inner = `<path d="M20 20 Q20 48 32 48 Q44 48 44 20 Q44 14 32 14 Q20 14 20 20Z" fill="${color}" stroke="#333" stroke-width="0.5"/><ellipse cx="32" cy="18" rx="12" ry="4" fill="none" stroke="${darken(color, 0.7)}" stroke-width="0.5"/><line x1="32" y1="18" x2="32" y2="44" stroke="${darken(color, 0.7)}" stroke-width="0.3" opacity="0.5"/>`;
  } else if (["circuit", "processor"].includes(stem)) {
    inner = `<rect x="12" y="14" width="40" height="36" rx="2" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="18" y="20" width="28" height="24" rx="1" fill="${darken(color, 0.8)}" stroke="#333" stroke-width="0.3"/>`;
    [22, 30, 38].forEach((px) => {
      inner += `<circle cx="${px}" cy="28" r="1.5" fill="#FFF"/><circle cx="${px}" cy="36" r="1.5" fill="#FFF"/>`;
    });
    if (stem === "processor") inner += `<rect x="26" y="24" width="12" height="16" rx="1" fill="#FFF" opacity="0.2"/>`;
  } else if (stem.includes("gear")) {
    const r = 14;
    inner = `<circle cx="32" cy="32" r="${r}" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="32" cy="32" r="4" fill="#333"/>`;
    for (let a = 0; a < 360; a += 45) {
      const rad = (a * Math.PI) / 180;
      const x1 = 32 + (r - 3) * Math.cos(rad);
      const y1 = 32 + (r - 3) * Math.sin(rad);
      inner += `<rect x="${x1 - 2}" y="${y1 - 2}" width="4" height="8" fill="${color}" stroke="#333" stroke-width="0.3" transform="rotate(${a}, ${x1}, ${y1})"/>`;
    }
  } else if (stem.includes("screw")) {
    inner = `<rect x="24" y="12" width="16" height="40" rx="2" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="24" y1="24" x2="40" y2="16" stroke="${darken(color, 0.7)}" stroke-width="0.5"/><line x1="24" y1="36" x2="40" y2="28" stroke="${darken(color, 0.7)}" stroke-width="0.5"/>`;
  } else if (stem.includes("motor")) {
    inner = `<rect x="16" y="18" width="32" height="28" rx="4" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="32" cy="32" r="6" fill="#333"/><rect x="30" y="28" width="4" height="8" fill="#FFF"/>`;
  } else if (stem.includes("battery")) {
    inner = `<rect x="18" y="12" width="28" height="40" rx="4" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="22" y="16" width="20" height="14" rx="2" fill="${darken(color, 0.7)}"/><rect x="22" y="34" width="20" height="14" rx="2" fill="${lighten(color, 0.3)}"/><rect x="24" y="10" width="16" height="4" rx="1" fill="#333"/>`;
  } else if (stem.includes("nano")) {
    inner = `<rect x="14" y="14" width="36" height="36" rx="4" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="32" cy="32" r="8" fill="${lighten(color, 0.3)}" opacity="0.8"/><circle cx="32" cy="32" r="3" fill="#FFF" opacity="0.5"/>`;
  } else if (stem.includes("crystal")) {
    inner = `<polygon points="32,8 48,48 16,48" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="32" y1="8" x2="32" y2="48" stroke="rgba(255,255,255,0.3)" stroke-width="1"/>`;
  } else if (["ingot", "parts", "scrap"].some((k) => stem.includes(k))) {
    inner = `<path d="M16 24 L24 16 L44 16 L52 24 L44 44 L24 44 Z" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="24" y1="16" x2="24" y2="44" stroke="${darken(color, 0.7)}" stroke-width="0.3"/><line x1="44" y1="16" x2="44" y2="44" stroke="${darken(color, 0.7)}" stroke-width="0.3"/>`;
    if (stem.includes("scrap")) inner += `<line x1="16" y1="24" x2="52" y2="24" stroke="${darken(color, 0.7)}" stroke-width="0.3"/>`;
  } else if (stem.includes("blood")) {
    inner = `<path d="M32 10 Q32 30 32 34 Q32 42 24 42 Q16 42 16 34 Q16 26 24 22 L32 10Z" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="24" cy="34" r="4" fill="${lighten(color, 0.3)}" opacity="0.5"/>`;
  } else if (stem.includes("neural")) {
    inner = `<ellipse cx="32" cy="32" rx="18" ry="14" fill="${color}" stroke="#333" stroke-width="0.5"/><path d="M22 28 Q28 24 32 28 Q36 24 42 28" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="2"/><path d="M20 36 Q26 32 32 36 Q38 32 44 36" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="2"/><circle cx="32" cy="32" r="2" fill="#FFF" opacity="0.6"/>`;
  } else if (["stem", "cell"].some((k) => stem.includes(k))) {
    inner = `<circle cx="32" cy="32" r="16" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="32" cy="32" r="6" fill="${lighten(color, 0.3)}"/><line x1="32" y1="16" x2="32" y2="48" stroke="${darken(color, 0.7)}" stroke-width="0.3"/><line x1="16" y1="32" x2="48" y2="32" stroke="${darken(color, 0.7)}" stroke-width="0.3"/>`;
  } else if (stem.includes("heart")) {
    inner = `<path d="M32 16 Q32 16 28 20 Q20 28 20 34 Q20 44 32 48 Q44 44 44 34 Q44 28 36 20 Q32 16 32 16Z" fill="${color}" stroke="#333" stroke-width="0.5"/><path d="M32 24 L32 40" stroke="${lighten(color, 0.3)}" stroke-width="1" opacity="0.5"/>`;
  } else if (stem.includes("genome")) {
    inner = `<path d="M36 8 Q44 16 36 24 Q28 32 36 40 Q44 48 36 56" fill="none" stroke="${color}" stroke-width="3"/><path d="M28 8 Q20 16 28 24 Q36 32 28 40 Q20 48 28 56" fill="none" stroke="${darken(color, 0.7)}" stroke-width="3"/>`;
    for (let y = 14; y < 52; y += 6) inner += `<line x1="28" y1="${y}" x2="36" y2="${y - 2}" stroke="#333" stroke-width="0.5"/><line x1="36" y1="${y + 4}" x2="28" y2="${y + 2}" stroke="#333" stroke-width="0.5"/>`;
  } else if (["alloy", "composite"].some((k) => stem.includes(k))) {
    inner = `<rect x="14" y="22" width="36" height="20" rx="3" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="18" y="26" width="28" height="12" rx="2" fill="none" stroke="${lighten(color, 0.3)}" stroke-width="0.5"/><line x1="22" y1="26" x2="22" y2="38" stroke="${darken(color, 0.7)}" stroke-width="0.3"/><line x1="42" y1="26" x2="42" y2="38" stroke="${darken(color, 0.7)}" stroke-width="0.3"/>`;
  } else if (["protein", "enzyme"].some((k) => stem.includes(k))) {
    inner = `<path d="M16 28 Q16 16 28 16 Q40 16 44 24 Q48 32 44 40 Q40 48 28 48 Q16 48 16 36Z" fill="${color}" stroke="#333" stroke-width="0.5"/><circle cx="28" cy="24" r="4" fill="${lighten(color, 0.3)}" opacity="0.7"/><circle cx="36" cy="36" r="4" fill="${lighten(color, 0.3)}" opacity="0.7"/>`;
    if (stem.includes("enzyme")) inner += `<circle cx="22" cy="36" r="3" fill="${lighten(color, 0.3)}" opacity="0.5"/>`;
  } else if (["compound", "organic"].some((k) => stem.includes(k))) {
    inner = `<circle cx="32" cy="32" r="16" fill="none" stroke="${color}" stroke-width="3"/><circle cx="32" cy="32" r="6" fill="${color}"/><circle cx="22" cy="24" r="3" fill="${lighten(color, 0.3)}" opacity="0.7"/><circle cx="42" cy="24" r="3" fill="${lighten(color, 0.3)}" opacity="0.7"/><circle cx="32" cy="42" r="3" fill="${lighten(color, 0.3)}" opacity="0.7"/>`;
  } else if (stem.includes("pipe")) {
    inner = `<rect x="8" y="26" width="48" height="12" rx="6" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="14" y="29" width="36" height="6" rx="3" fill="${darken(color, 0.8)}"/>`;
  } else if (stem.includes("interface")) {
    inner = `<rect x="12" y="14" width="40" height="36" rx="3" fill="${color}" stroke="#333" stroke-width="0.5"/><ellipse cx="32" cy="32" rx="12" ry="10" fill="${lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/><circle cx="32" cy="32" r="3" fill="#FFF" opacity="0.6"/>`;
  } else if (stem.includes("pickaxe")) {
    inner = `<path d="M28 48 L28 28 L16 16 L20 12 L32 24 L36 20 L40 24 L36 28 L36 48Z" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="26" y="44" width="12" height="6" rx="2" fill="${darken(color, 0.7)}"/>`;
  } else if (stem.includes("stone_axe")) {
    inner = `<path d="M24 48 L24 28 L12 20 Q16 10 28 18 L32 20 L36 16 L40 20 L32 28 L32 48Z" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="22" y="44" width="12" height="6" rx="2" fill="${darken(color, 0.7)}"/>`;
  } else if (stem.includes("blade")) {
    inner = `<path d="M32 10 L40 24 L38 26 L34 22 L32 46 L30 22 L26 26 L24 24Z" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="29" y="42" width="6" height="8" rx="1" fill="${darken(color, 0.7)}"/>`;
  } else if (stem.includes("hammer")) {
    inner = `<rect x="26" y="24" width="24" height="18" rx="3" fill="${color}" stroke="#333" stroke-width="0.5"/><rect x="14" y="42" width="10" height="6" rx="2" fill="${darken(color, 0.7)}"/><rect x="18" y="24" width="10" height="24" rx="2" fill="${darken(color, 0.5)}"/>`;
  } else if (stem.includes("silicon")) {
    inner = `<rect x="16" y="16" width="32" height="32" rx="4" fill="${color}" stroke="#333" stroke-width="0.5"/><line x1="16" y1="16" x2="48" y2="48" stroke="${darken(color, 0.7)}" stroke-width="0.3"/><line x1="48" y1="16" x2="16" y2="48" stroke="${darken(color, 0.7)}" stroke-width="0.3"/>`;
  } else {
    inner = `<rect x="16" y="16" width="32" height="32" rx="3" fill="${color}" stroke="#333" stroke-width="0.5"/>`;
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">
  ${inner}
</svg>`;
}

// ── Main ──

function generateAll() {
  console.log("Génération des SVG de bâtiments...");
  for (const [stem, color, w, h] of BUILDINGS) {
    const svg = makeBuildingSVG(stem, color, w, h);
    fs.writeFileSync(path.join(SVG_DIR, `${stem}_base.svg`), svg);
    console.log(`  ✓ ${stem}_base.svg`);
  }

  console.log("\nGénération des SVG de capsule...");
  for (const [stem, color] of CAPSULE) {
    const svg = makeCapsuleSVG(stem, color);
    fs.writeFileSync(path.join(SVG_DIR, `${stem}_base.svg`), svg);
    console.log(`  ✓ ${stem}_base.svg`);
  }

  console.log("\nGénération des SVG de ressources...");
  for (const [stem, color] of Object.entries(RESOURCES)) {
    const svg = makeResourceSVG(stem, color);
    fs.writeFileSync(path.join(SVG_DIR, `${stem}_base.svg`), svg);
    console.log(`  ✓ ${stem}_base.svg`);
  }

  console.log(
    `\n✨ ${
      BUILDINGS.length + CAPSULE.length + Object.keys(RESOURCES).length
    } SVG générés !`
  );
}

async function convertToPng() {
  try {
    const sharp = require("sharp");
    console.log("\nConversion SVG → PNG 64x64...");
    const files = fs.readdirSync(SVG_DIR).filter((f) => f.endsWith("_base.svg"));
    let ok = 0,
      fail = 0;
    for (const file of files) {
      const svgPath = path.join(SVG_DIR, file);
      const pngPath = path.join(PNG_DIR, file.replace("_base.svg", "_base.png"));
      try {
        const svgBuf = fs.readFileSync(svgPath);
        await sharp(svgBuf).resize(64, 64).png().toFile(pngPath);
        ok++;
        console.log(`  ✓ ${path.basename(pngPath)}`);
      } catch (e) {
        console.log(`  ✗ ${file}: ${e.message}`);
        fail++;
      }
    }
    console.log(`\n✨ ${ok} PNG générés${fail ? `, ${fail} échecs` : ""}`);
  } catch {
    console.log(
      '\n⚠ sharp non disponible. Pour convertir en PNG: npm install sharp\nLes SVG sont disponibles dans textures/svg/'
    );
  }
}

generateAll();
convertToPng();
