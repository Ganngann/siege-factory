#!/usr/bin/env python3
"""
Génère les assets SVG isométriques pour le mod Genesis Protocol,
puis les convertit en PNG 64×64.
Usage: python generate_assets.py
"""

import os
import math
import subprocess
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SVG_DIR = BASE_DIR / "svg"
PNG_DIR = BASE_DIR / "png"
os.makedirs(SVG_DIR, exist_ok=True)
os.makedirs(PNG_DIR, exist_ok=True)

TILE_W = 64
TILE_H = 32
CX = TILE_W / 2
CY = TILE_H / 1.5


def darken(hex_color, factor):
    hex_color = hex_color.lstrip('#')
    r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)
    r = int(r * factor)
    g = int(g * factor)
    b = int(b * factor)
    return f'#{r:02x}{g:02x}{b:02x}'


def lighten(hex_color, factor):
    hex_color = hex_color.lstrip('#')
    r, g, b = int(hex_color[0:2], 16), int(hex_color[2:4], 16), int(hex_color[4:6], 16)
    r = min(255, int(r + (255 - r) * factor))
    g = min(255, int(g + (255 - g) * factor))
    b = min(255, int(b + (255 - b) * factor))
    return f'#{r:02x}{g:02x}{b:02x}'


# ── BÂTIMENTS ──

BUILDINGS = {
    "workbench":       {"c": "#8B5E3C", "w": 1, "h": 1, "d": "Table avec outils"},
    "campfire":        {"c": "#FF6622", "w": 1, "h": 1, "d": "Feu avec pierres"},
    "furnace":         {"c": "#884422", "w": 1, "h": 1, "d": "Four en briques"},
    "anvil":           {"c": "#666666", "w": 1, "h": 1, "d": "Enclume sur billot"},
    "burner_generator":{"c": "#DD6622", "w": 1, "h": 1, "d": "Générateur charbon"},
    "manual_miner":    {"c": "#AA7733", "w": 1, "h": 1, "d": "Mineur manuel"},
    "water_pump":      {"c": "#3399DD", "w": 1, "h": 1, "d": "Pompe à eau"},
    "steam_generator": {"c": "#CCDDEE", "w": 1, "h": 1, "d": "Chaudière vapeur"},
    "blast_furnace":   {"c": "#AA4422", "w": 1, "h": 1, "d": "Haut fourneau"},
    "gear_press":      {"c": "#887766", "w": 1, "h": 1, "d": "Presse mécanique"},
    "belt":            {"c": "#808080", "w": 1, "h": 1, "d": "Convoyeur"},
    "splitter":        {"c": "#AAAA00", "w": 1, "h": 1, "d": "Séparateur"},
    "electric_generator":{"c": "#FFAA33", "w": 1, "h": 1, "d": "Générateur électrique"},
    "power_pole":      {"c": "#888888", "w": 1, "h": 1, "d": "Pylône électrique"},
    "assembler":       {"c": "#4D99CC", "w": 1, "h": 1, "d": "Assembleur"},
    "chemical_lab":    {"c": "#664488", "w": 2, "h": 2, "d": "Labo chimique"},
    "oil_pump":        {"c": "#444455", "w": 1, "h": 1, "d": "Pompe à pétrole"},
    "storage_chest":   {"c": "#CC9900", "w": 1, "h": 1, "d": "Coffre"},
    "motor_foundry":   {"c": "#AA8844", "w": 2, "h": 1, "d": "Fonderie moteurs"},
    "battery_station": {"c": "#33AA33", "w": 1, "h": 1, "d": "Station batteries"},
    "electronics_lab": {"c": "#33AA88", "w": 2, "h": 2, "d": "Labo électronique"},
    "assembly_crane":  {"c": "#3377AA", "w": 2, "h": 1, "d": "Grue d'assemblage"},
    "aerial_belt":     {"c": "#88AACC", "w": 1, "h": 1, "d": "Convoyeur aérien"},
    "sorter":          {"c": "#66AA66", "w": 1, "h": 1, "d": "Trieur"},
    "nanite_assembler":{"c": "#44DDBB", "w": 2, "h": 2, "d": "Assembleur nanite"},
    "deep_core_drill": {"c": "#664433", "w": 3, "h": 2, "d": "Foreuse profonde"},
    "compactor":       {"c": "#AAAA77", "w": 1, "h": 1, "d": "Compacteur"},
    "high_speed_belt": {"c": "#CC8844", "w": 1, "h": 1, "d": "Convoyeur rapide"},
    "excavation_rig":  {"c": "#775533", "w": 2, "h": 2, "d": "Engin excavation"},
    "bio_lab":         {"c": "#66BB6A", "w": 2, "h": 2, "d": "Bio-laboratoire"},
    "tissue_cultivator":{"c": "#AB47BC", "w": 2, "h": 2, "d": "Culture tissus"},
    "synthesizer":     {"c": "#FF7043", "w": 1, "h": 1, "d": "Synthétiseur"},
    "scanner_array":   {"c": "#42A5F5", "w": 2, "h": 2, "d": "Réseau scanners"},
    "bio_printer":     {"c": "#4DB6AC", "w": 2, "h": 2, "d": "Bio-imprimante"},
}

CAPSULE_TIERS = [
    ("genesis_capsule_t0", "#445566", "Éteinte"),
    ("genesis_capsule_t1", "#5577AA", "Voyants verts"),
    ("genesis_capsule_t2", "#6699CC", "Liquide visible"),
    ("genesis_capsule_t3", "#88BBDD", "Lueur interne"),
    ("genesis_capsule_t4", "#AACCCC", "Battement"),
    ("genesis_capsule_t5", "#CCDDDD", "Vitre trouble"),
    ("genesis_capsule_t6", "#DDEEEE", "Signes vitaux"),
    ("genesis_capsule_t7", "#EEFFFF", "Prêt réveil"),
]

RESOURCES = {
    "scrap_metal":      "#887766",
    "clay":             "#C4A882",
    "plant_fiber":      "#5A8C3C",
    "planks":           "#A0724A",
    "stone_brick":      "#887766",
    "ceramic":          "#C4956A",
    "rope":             "#8B7355",
    "iron_parts":       "#999999",
    "iron_ingot":       "#AAAAAA",
    "copper_ingot":     "#CC8844",
    "water":            "#3399DD",
    "steam":            "#CCDDEE",
    "acid":             "#88FF44",
    "silicon":          "#AACCDD",
    "processor":        "#33BB33",
    "alloy":            "#8888AA",
    "organic_compound": "#66BB6A",
    "enzyme":           "#AB47BC",
    "protein":          "#FF7043",
    "synthetic_blood":  "#EF5350",
    "neural_map":       "#42A5F5",
    "stem_cells":       "#FFD54F",
    "bio_mass":         "#4DB6AC",
    "neural_interface": "#7E57C2",
    "synthetic_heart":  "#EF5350",
    "genome_sequence":  "#66BB6A",
}


# ── GÉNÉRATEUR SVG BÂTIMENT ISOMÉTRIQUE ──

def make_building_svg(stem, color, w, h, desc):
    ox = CX + 2
    oy = TILE_H + 10
    uw = 10
    uh = 5
    bh = 3.0

    def p(gx, gy):
        px = ox + (gx - gy) * uw
        py = oy + (gx + gy) * uh
        return px, py

    def poly(pts):
        return " ".join(f"{p(x,y)[0]:.1f},{p(x,y)[1]:.1f}" for x, y in pts)

    top = [(0, -h), (w, -h), (w, 0), (0, 0)]
    left = [(0, 0), (w, 0), (w, h), (0, h)]
    right = [(0, 0), (0, h), (-w, 0), (-w, 0)]

    lines = [
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">',
        f'<polygon points="{poly(left)}" fill="{darken(color, 0.5)}" stroke="#333" stroke-width="0.5"/>',
        f'<polygon points="{poly(top)}" fill="{color}" stroke="#333" stroke-width="0.5"/>',
    ]

    if stem == "campfire":
        lines.append(f'<polygon points="{poly([(0.2,-h+0.5),(w-0.2,-h+0.5),(w-0.2,0),(0.2,0)])}" fill="{darken(color, 0.7)}" stroke="#333" stroke-width="0.3"/>')
        cx2 = p(w/2, -h/2)[0]
        cy2 = p(w/2, -h/2)[1]
        lines.append(f'<ellipse cx="{cx2:.1f}" cy="{cy2:.1f}" rx="3" ry="2" fill="#FFDD44" opacity="0.9"/>')
        lines.append(f'<ellipse cx="{cx2:.1f}" cy="{cy2-2:.1f}" rx="2" ry="3" fill="#FF8800" opacity="0.7"/>')

    elif stem == "power_pole":
        lines.append(f'<line x1="{p(0.5,0)[0]:.1f}" y1="{p(0.5,0)[1]:.1f}" x2="{p(0.5,-h)[0]:.1f}" y2="{p(0.5,-h)[1]:.1f}" stroke="#555" stroke-width="2"/>')
        cx2 = p(0.5, -h-0.5)[0]
        cy2 = p(0.5, -h-0.5)[1]
        lines.append(f'<circle cx="{cx2:.1f}" cy="{cy2:.1f}" r="3" fill="#FFAA00" stroke="#333" stroke-width="0.5"/>')

    elif stem in ("belt", "high_speed_belt", "aerial_belt"):
        lines.append(f'<polygon points="{poly([(0.1,-h+0.5),(w-0.1,-h+0.5),(w-0.1,0),(0.1,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2 = p(w/2, -h/2)[0]
        cy2 = p(w/2, -h/2)[1]
        lines.append(f'<line x1="{cx2-6:.1f}" y1="{cy2:.1f}" x2="{cx2+6:.1f}" y2="{cy2:.1f}" stroke="{darken(color, 0.7)}" stroke-width="1"/>')

    elif stem == "splitter":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<circle cx="{cx2:.1f}" cy="{cy2:.1f}" r="3" fill="#333"/>')
        lines.append(f'<line x1="{cx2:.1f}" y1="{cy2-4:.1f}" x2="{cx2:.1f}" y2="{cy2+4:.1f}" stroke="#333" stroke-width="1"/>')
        lines.append(f'<line x1="{cx2-4:.1f}" y1="{cy2:.1f}" x2="{cx2+4:.1f}" y2="{cy2:.1f}" stroke="#333" stroke-width="1"/>')

    elif stem == "sorter":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<polygon points="{cx2-4:.1f},{cy2-3:.1f} {cx2+4:.1f},{cy2:.1f} {cx2-4:.1f},{cy2+3:.1f}" fill="#333"/>')

    elif stem == "anvil":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<polygon points="{cx2-5:.1f},{cy2:.1f} {cx2:.1f},{cy2-4:.1f} {cx2+5:.1f},{cy2:.1f} {cx2:.1f},{cy2+2:.1f}" fill="#333"/>')

    elif stem in ("deep_core_drill", "excavation_rig"):
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<rect x="{cx2-2:.1f}" y="{cy2-4:.1f}" width="4" height="8" fill="#444" rx="1"/>')

    elif stem == "storage_chest":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<rect x="{cx2-4:.1f}" y="{cy2-2:.1f}" width="8" height="4" fill="{darken(color, 0.7)}" rx="1"/>')

    elif stem == "assembler":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<rect x="{cx2-4:.1f}" y="{cy2-3:.1f}" width="8" height="6" fill="{lighten(color, 0.2)}" rx="1" stroke="#333" stroke-width="0.3"/>')
        lines.append(f'<circle cx="{cx2:.1f}" cy="{cy2:.1f}" r="1.5" fill="#FFF" opacity="0.5"/>')

    elif stem == "oil_pump":
        lines.append(f'<polygon points="{poly([(0,-h+0.5),(w,-h+0.5),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<line x1="{cx2:.1f}" y1="{cy2:.1f}" x2="{cx2:.1f}" y2="{cy2-5:.1f}" stroke="#555" stroke-width="2"/>')

    elif stem == "water_pump":
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p(w/2, -h/2)
        lines.append(f'<path d="M{cx2-3:.1f} {cy2+2:.1f} Q{cx2:.1f} {cy2-4:.1f} {cx2+3:.1f} {cy2+2:.1f}" fill="none" stroke="#FFF" stroke-width="1" opacity="0.7"/>')

    elif stem in ("nanite_assembler", "electronics_lab", "chemical_lab", "bio_lab", "scanner_array", "bio_printer", "tissue_cultivator"):
        ox2 = ox + 5
        oy2 = oy + 5
        def p2(gx, gy):
            px = ox2 + (gx - gy) * uw
            py = oy2 + (gx + gy) * uh
            return px, py
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        cx2, cy2 = p2(w/2, -h/2)
        lines.append(f'<circle cx="{cx2:.1f}" cy="{cy2:.1f}" r="4" fill="none" stroke="{lighten(color, 0.3)}" stroke-width="0.5" opacity="0.6"/>')
        lines.append(f'<circle cx="{cx2:.1f}" cy="{cy2:.1f}" r="1.5" fill="{lighten(color, 0.3)}" opacity="0.8"/>')

    else:
        lines.append(f'<polygon points="{poly([(0,-h),(w,-h),(w,0),(0,0)])}" fill="{color}" stroke="#333" stroke-width="0.5"/>')

    lines.append('</svg>')
    return '\n'.join(lines)


# ── GÉNÉRATEUR SVG CAPSULE ──

def make_capsule_svg(stem, color, desc):
    ox, oy = CX, TILE_H + 8
    bw, bh = 22, 11
    ch = 16

    lines = [
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">',
        f'<ellipse cx="{ox}" cy="{oy + 2}" rx="{bw/2}" ry="{bh/2}" fill="rgba(0,0,0,0.2)"/>',
        f'<ellipse cx="{ox}" cy="{oy - ch/2}" rx="{bw/2}" ry="{ch}" fill="{color}" stroke="#333" stroke-width="1"/>',
        f'<ellipse cx="{ox}" cy="{oy - ch/2}" rx="{bw/2 - 2}" ry="{ch * 0.3}" fill="none" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>',
    ]

    if "t7" in stem:
        wc = "#EEFFFF"
    elif "t6" in stem:
        wc = "#CCEEFF"
    elif "t5" in stem:
        wc = "#88AACC"
    elif "t4" in stem:
        wc = "#88BBFF"
    elif "t3" in stem:
        wc = "#5577AA"
    else:
        wc = "#223344"

    lines.append(f'<circle cx="{ox}" cy="{oy - ch/2}" r="5" fill="{wc}" stroke="#333" stroke-width="0.8"/>')

    if stem not in ("genesis_capsule_t0", "genesis_capsule_t1"):
        lines.append(f'<circle cx="{ox - 8}" cy="{oy - ch/2 - 3}" r="1.5" fill="#33FF33" opacity="0.8"/>')
        lines.append(f'<circle cx="{ox + 8}" cy="{oy - ch/2 - 3}" r="1.5" fill="#FF3333" opacity="0.8"/>')

    lines.append('</svg>')
    return '\n'.join(lines)


# ── GÉNÉRATEUR SVG RESSOURCE ──

def make_resource_svg(stem, color):
    lines = [
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" width="64" height="64">',
    ]

    if stem in ("water", "steam"):
        lines.append(f'<path d="M32 8 Q32 24 32 28 Q32 36 24 36 Q16 36 16 28 Q16 20 24 16 L32 8Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        if stem == "steam":
            lines.append(f'<circle cx="38" cy="20" r="3" fill="rgba(255,255,255,0.4)"/>')
            lines.append(f'<circle cx="44" cy="14" r="2" fill="rgba(255,255,255,0.3)"/>')
    elif stem == "acid":
        lines.append(f'<path d="M32 10 Q32 26 32 30 Q32 38 24 38 Q16 38 16 30 Q16 22 24 18 L32 10Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<path d="M32 10 L36 18 L32 16 Z" fill="#FFF" opacity="0.3"/>')
    elif "fiber" in stem:
        lines.append(f'<path d="M32 48 Q16 40 16 28 Q16 16 32 16 Q48 16 48 28 Q48 40 32 48Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="32" y1="48" x2="32" y2="16" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="24" y1="40" x2="40" y2="24" stroke="#333" stroke-width="0.3" opacity="0.5"/>')
    elif "plank" in stem:
        lines.append(f'<rect x="12" y="20" width="40" height="24" rx="2" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="20" y1="20" x2="20" y2="44" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
        lines.append(f'<line x1="44" y1="20" x2="44" y2="44" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
    elif "rope" in stem:
        lines.append(f'<ellipse cx="32" cy="32" rx="16" ry="10" fill="none" stroke="{color}" stroke-width="4"/>')
        lines.append(f'<ellipse cx="32" cy="32" rx="10" ry="6" fill="none" stroke="{darken(color, 0.8)}" stroke-width="2"/>')
    elif "brick" in stem:
        lines.append(f'<rect x="10" y="20" width="44" height="24" rx="1" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="32" y1="20" x2="32" y2="32" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
        lines.append(f'<line x1="10" y1="32" x2="54" y2="32" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
    elif stem == "ceramic":
        lines.append(f'<path d="M20 20 Q20 48 32 48 Q44 48 44 20 Q44 14 32 14 Q20 14 20 20Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<ellipse cx="32" cy="18" rx="12" ry="4" fill="none" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
        lines.append(f'<line x1="32" y1="18" x2="32" y2="44" stroke="{darken(color, 0.7)}" stroke-width="0.3" opacity="0.5"/>')
    elif "circuit" in stem or "processor" in stem:
        lines.append(f'<rect x="12" y="14" width="40" height="36" rx="2" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<rect x="18" y="20" width="28" height="24" rx="1" fill="{darken(color, 0.8)}" stroke="#333" stroke-width="0.3"/>')
        for px in [22, 30, 38]:
            lines.append(f'<circle cx="{px}" cy="28" r="1.5" fill="#FFF"/>')
            lines.append(f'<circle cx="{px}" cy="36" r="1.5" fill="#FFF"/>')
        if "processor" in stem:
            lines.append(f'<rect x="26" y="24" width="12" height="16" rx="1" fill="#FFF" opacity="0.2"/>')
    elif "gear" in stem:
        r = 14
        lines.append(f'<circle cx="32" cy="32" r="{r}" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="32" cy="32" r="4" fill="#333"/>')
        for angle in range(0, 360, 45):
            a = math.radians(angle)
            x1 = 32 + (r - 3) * math.cos(a)
            y1 = 32 + (r - 3) * math.sin(a)
            lines.append(f'<rect x="{x1-2:.1f}" y="{y1-2:.1f}" width="4" height="8" fill="{color}" stroke="#333" stroke-width="0.3" transform="rotate({angle}, {x1:.1f}, {y1:.1f})"/>')
    elif "screw" in stem:
        lines.append(f'<rect x="24" y="12" width="16" height="40" rx="2" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="24" y1="24" x2="40" y2="16" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
        lines.append(f'<line x1="24" y1="36" x2="40" y2="28" stroke="{darken(color, 0.7)}" stroke-width="0.5"/>')
    elif "motor" in stem:
        lines.append(f'<rect x="16" y="18" width="32" height="28" rx="4" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="32" cy="32" r="6" fill="#333"/>')
        lines.append(f'<rect x="30" y="28" width="4" height="8" fill="#FFF"/>')
    elif "battery" in stem:
        lines.append(f'<rect x="18" y="12" width="28" height="40" rx="4" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<rect x="22" y="16" width="20" height="14" rx="2" fill="{darken(color, 0.7)}"/>')
        lines.append(f'<rect x="22" y="34" width="20" height="14" rx="2" fill="{lighten(color, 0.3)}"/>')
        lines.append(f'<rect x="24" y="10" width="16" height="4" rx="1" fill="#333"/>')
    elif "nano" in stem:
        lines.append(f'<rect x="14" y="14" width="36" height="36" rx="4" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="32" cy="32" r="8" fill="{lighten(color, 0.3)}" opacity="0.8"/>')
        lines.append(f'<circle cx="32" cy="32" r="3" fill="#FFF" opacity="0.5"/>')
    elif "crystal" in stem or "laser" in stem:
        lines.append(f'<polygon points="32,8 48,48 16,48" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="32" y1="8" x2="32" y2="48" stroke="rgba(255,255,255,0.3)" stroke-width="1"/>')
    elif "ingot" in stem or "parts" in stem or "scrap" in stem:
        lines.append(f'<path d="M16 24 L24 16 L44 16 L52 24 L44 44 L24 44 Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="24" y1="16" x2="24" y2="44" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
        lines.append(f'<line x1="44" y1="16" x2="44" y2="44" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
        if "scrap" in stem:
            lines.append(f'<line x1="16" y1="24" x2="52" y2="24" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
    elif "blood" in stem:
        lines.append(f'<path d="M32 10 Q32 30 32 34 Q32 42 24 42 Q16 42 16 34 Q16 26 24 22 L32 10Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="24" cy="34" r="4" fill="{lighten(color, 0.3)}" opacity="0.5"/>')
    elif "neural" in stem:
        lines.append(f'<ellipse cx="32" cy="32" rx="18" ry="14" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<path d="M22 28 Q28 24 32 28 Q36 24 42 28" fill="none" stroke="{lighten(color, 0.3)}" stroke-width="2"/>')
        lines.append(f'<path d="M20 36 Q26 32 32 36 Q38 32 44 36" fill="none" stroke="{lighten(color, 0.3)}" stroke-width="2"/>')
        lines.append(f'<circle cx="32" cy="32" r="2" fill="#FFF" opacity="0.6"/>')
    elif "stem" in stem or "cell" in stem:
        lines.append(f'<circle cx="32" cy="32" r="16" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="32" cy="32" r="6" fill="{lighten(color, 0.3)}"/>')
        lines.append(f'<line x1="32" y1="16" x2="32" y2="48" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
        lines.append(f'<line x1="16" y1="32" x2="48" y2="32" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
    elif "heart" in stem:
        lines.append(f'<path d="M32 16 Q32 16 28 20 Q20 28 20 34 Q20 44 32 48 Q44 44 44 34 Q44 28 36 20 Q32 16 32 16Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<path d="M32 24 L32 40" stroke="{lighten(color, 0.3)}" stroke-width="1" opacity="0.5"/>')
    elif "genome" in stem:
        lines.append(f'<path d="M36 8 Q44 16 36 24 Q28 32 36 40 Q44 48 36 56" fill="none" stroke="{color}" stroke-width="3"/>')
        lines.append(f'<path d="M28 8 Q20 16 28 24 Q36 32 28 40 Q20 48 28 56" fill="none" stroke="{darken(color, 0.7)}" stroke-width="3"/>')
        for y in range(14, 52, 6):
            lines.append(f'<line x1="28" y1="{y}" x2="36" y2="{y-2}" stroke="#333" stroke-width="0.5"/>')
            lines.append(f'<line x1="36" y1="{y+4}" x2="28" y2="{y+2}" stroke="#333" stroke-width="0.5"/>')
    elif "alloy" in stem or "composite" in stem:
        lines.append(f'<rect x="14" y="22" width="36" height="20" rx="3" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<rect x="18" y="26" width="28" height="12" rx="2" fill="none" stroke="{lighten(color, 0.3)}" stroke-width="0.5"/>')
        lines.append(f'<line x1="22" y1="26" x2="22" y2="38" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
        lines.append(f'<line x1="42" y1="26" x2="42" y2="38" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
    elif "protein" in stem or "enzyme" in stem:
        lines.append(f'<path d="M16 28 Q16 16 28 16 Q40 16 44 24 Q48 32 44 40 Q40 48 28 48 Q16 48 16 36Z" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<circle cx="28" cy="24" r="4" fill="{lighten(color, 0.3)}" opacity="0.7"/>')
        lines.append(f'<circle cx="36" cy="36" r="4" fill="{lighten(color, 0.3)}" opacity="0.7"/>')
        if "enzyme" in stem:
            lines.append(f'<circle cx="22" cy="36" r="3" fill="{lighten(color, 0.3)}" opacity="0.5"/>')
    elif "compound" in stem or "organic" in stem:
        lines.append(f'<circle cx="32" cy="32" r="16" fill="none" stroke="{color}" stroke-width="3"/>')
        lines.append(f'<circle cx="32" cy="32" r="6" fill="{color}"/>')
        lines.append(f'<circle cx="22" cy="24" r="3" fill="{lighten(color, 0.3)}" opacity="0.7"/>')
        lines.append(f'<circle cx="42" cy="24" r="3" fill="{lighten(color, 0.3)}" opacity="0.7"/>')
        lines.append(f'<circle cx="32" cy="42" r="3" fill="{lighten(color, 0.3)}" opacity="0.7"/>')
    elif "pipe" in stem:
        lines.append(f'<rect x="8" y="26" width="48" height="12" rx="6" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<rect x="14" y="29" width="36" height="6" rx="3" fill="{darken(color, 0.8)}"/>')
    elif "interface" in stem:
        lines.append(f'<rect x="12" y="14" width="40" height="36" rx="3" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<ellipse cx="32" cy="32" rx="12" ry="10" fill="{lighten(color, 0.2)}" stroke="#333" stroke-width="0.3"/>')
        lines.append(f'<circle cx="32" cy="32" r="3" fill="#FFF" opacity="0.6"/>')
    elif "silicon" in stem:
        lines.append(f'<rect x="16" y="16" width="32" height="32" rx="4" fill="{color}" stroke="#333" stroke-width="0.5"/>')
        lines.append(f'<line x1="16" y1="16" x2="48" y2="48" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
        lines.append(f'<line x1="48" y1="16" x2="16" y2="48" stroke="{darken(color, 0.7)}" stroke-width="0.3"/>')
    else:
        lines.append(f'<rect x="16" y="16" width="32" height="32" rx="3" fill="{color}" stroke="#333" stroke-width="0.5"/>')

    lines.append('</svg>')
    return '\n'.join(lines)


# ── GÉNÉRATION ──

def generate_all():
    print("Génération des SVG de bâtiments...")
    for stem, info in BUILDINGS.items():
        svg = make_building_svg(stem, info["c"], info["w"], info["h"], info["d"])
        with open(SVG_DIR / f"{stem}_base.svg", "w") as f:
            f.write(svg)
        print(f"  ✓ {stem}_base.svg")

    print("\nGénération des SVG de capsule...")
    for stem, color, desc in CAPSULE_TIERS:
        svg = make_capsule_svg(stem, color, desc)
        with open(SVG_DIR / f"{stem}_base.svg", "w") as f:
            f.write(svg)
        print(f"  ✓ {stem}_base.svg")

    print("\nGénération des SVG de ressources...")
    for stem, color in RESOURCES.items():
        svg = make_resource_svg(stem, color)
        with open(SVG_DIR / f"{stem}_base.svg", "w") as f:
            f.write(svg)
        print(f"  ✓ {stem}_base.svg")

    print(f"\n✨ {len(BUILDINGS) + len(CAPSULE_TIERS) + len(RESOURCES)} SVG générés !")


def convert_to_png():
    print("\nConversion SVG → PNG 64×64...")
    svgs = sorted(SVG_DIR.glob("*_base.svg"))

    converter = None
    try:
        import cairosvg
        converter = "cairosvg"
        print("  ✓ cairosvg disponible")
    except ImportError:
        try:
            subprocess.run(["rsvg-convert", "--version"], capture_output=True, check=True)
            converter = "rsvg-convert"
            print("  ✓ rsvg-convert disponible")
        except (FileNotFoundError, subprocess.CalledProcessError):
            pass

    if not converter:
        print("  ⚠ Aucun convertisseur trouvé !")
        print("  Installe cairosvg: pip install cairosvg")
        print("  Ou rsvg-convert: choco install rsvg-convert")
        return

    ok = 0
    fail = 0
    for svg in svgs:
        png = PNG_DIR / svg.name.replace("_base.svg", "_base.png")
        try:
            if converter == "cairosvg":
                import cairosvg
                cairosvg.svg2png(url=str(svg), write_to=str(png), output_width=64, output_height=64)
            else:
                subprocess.run(
                    ["rsvg-convert", "-w", "64", "-h", "64", "-o", str(png), str(svg)],
                    check=True, capture_output=True
                )
            ok += 1
            print(f"  ✓ {png.name}")
        except Exception as e:
            print(f"  ✗ {svg.name}: {e}")
            fail += 1

    print(f"\n✨ {ok} PNG générés" + (f", {fail} échecs" if fail else ""))


if __name__ == "__main__":
    generate_all()
    convert_to_png()
