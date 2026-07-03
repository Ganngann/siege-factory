# SVG Style Guide — Siege Factory

## 1. Canvas & Grid

- **Base tile** : 32×32 px (in-game via `map_config.toml`)
- **SVG scale** : ×2 → every tile is 64×64 px in SVG for crisp rendering
- **Multi-tile** : SVG canvas = tiles × 64 (e.g. 2×2 → 128×128)
- **Padding** : 2 px min between building edge and tile border
  - Exception: belts and output ports have 0 px padding on connection sides
- **Readability** : details ≥ 1 px stroke allowed (rendered at 0.5 px in-game)

| Footprint | Game px | SVG canvas | Example |
|---|---|---|---|---|
| 1×1  | 32×32   | 64×64    | `miner_east.svg` |
| 1×2  | 32×64   | 64×128   | `miner_east_tall.svg` |
| 2×2  | 64×64   | 128×128  | `miner_east_2x2.svg` |
| 3×2  | 96×64   | 192×128  | `miner_east_3x2.svg` |
| 3×3  | 96×96   | 192×192  | `miner_east_3x3.svg` |

### Stroke Consistency

All SVGs use the **same stroke widths** regardless of canvas size.
Since every SVG is rendered at ×2 scale, a 1 px stroke in SVG =
0.5 px in-game for **any building size**.

| Element | Stroke width |
|---|---|
| Border (main outline) | 1 px |
| Mechanical detail | 1–1.5 px |
| Shaft / rail | 2 px |
| Highlight / shadow | 0.5 px |
| Fine detail (bolts) | 0.5 px |

## 2. Belt Ports

Every building has **belt ports** on tile edges where belts connect.
In top-down view, ports are bright yellow rectangles for visibility.

### Port position

The port is always at the **center of the tile edge**, aligned with the
belt lane:

```
  port y=24..40, h=16    centered on the tile edge
```

### Port alignment table

| Building size | Port Y (in SVG) | Port H | Notes |
|---|---|---|---|---|
| 1×1 | 24–40 | 16 | Single port, centered per face |
| 2×2 | 24–40, 88–104 | 16 | 2 ports per face (one per tile) |

### Visual design of a port

```svg
<rect x="W-4" y="24" width="4" height="16" rx="1" fill="#ffcc00" stroke="#cc9900" stroke-width="0.5"/>
<line x1="W-4" y1="24" x2="W-4" y2="40" stroke="#ffe066" stroke-width="0.5"/>  <!-- highlight -->
```

Where `W` = SVG canvas width (64 for 1×1, 128 for 2×2, etc.).
Ports on the left edge use `x="0"` ; top edge uses `y="0"` with `width="16"` and `height="4"`.

## 3. Color Palette

| Role | Color | Opacity |
|---|---|---|
| Building base | `#4a4a4a`–`#6a6a6a` (gradient) | 100 % |
| Border | `#3e3e3e` (1 px stroke) | 100 % |
| Highlight (top/left edge) | `#555`–`#6a6a6a` (0.5 px stroke) | 100 % |
| Inner rails / columns | `#525252`–`#585858` | 100 % |
| Cross-bracing / detail | `#5c5c5c` | 100 % |
| Mechanical parts (shafts, bolts) | `#6e6e6e`–`#7a7a7a` | 100 % |
| Drill bits | `#777` | 100 % |
| Direction indicators (arrows) | `#aaa` | 60–70 % |
| Belt surface active band | `#5a5a5a` | 100 % |
| Port recess | `#2e2e2e`–`#3a3a3a` | 100 % |
| **Belt port (top-down)** | **`#ffcc00`** | **100 %** |

## 4. Layering

Each SVG is split into **named `<g>` groups** for runtime compositing:

```svg
<svg ...>
  <!-- BUILDING BASE — fixed grayscale structure -->
  <g id="base">
    <!-- main body, structural details, shadows — always rendered -->
  </g>

  <!-- OWNER COLOR — tinted by the owning player -->
  <g id="owner_color">
    <!-- Use fill="white" for areas that will receive the player's color -->
    <!-- Black / transparent = ignored by tint -->
    <!-- NO GRADIENTS in this layer (lost by tinting) -->
  </g>

  <!-- LEVEL COLOR — tinted by the building level -->
  <g id="level_color">
    <!-- Same rules as owner_color: fill="white" = tintable zone -->
  </g>
</svg>
```

Runtime result:
```
(base layer, grayscale) + (owner_color × PLAYER_TINT) + (level_color × LEVEL_TINT)
```

## 5. Naming Convention

```
<building_id>[_orientation][_size][_variant].svg

Orientations: east, north, west, south
Size suffix:  tall (1×2), 2x2, 3x2, 3x3
              (omitted for default 1×1)

Examples:
  belt_east.svg              — straight belt, → direction (1×1)
  belt_north.svg             — straight belt, ↑ direction (1×1)
  belt_turn_en.svg           — curved belt, east → north
  miner_east.svg             — miner facing east (1×1)
  miner_east_tall.svg        — miner facing east (1×2)
  miner_east_2x2.svg         — miner facing east (2×2)
  miner_east_3x2.svg         — miner facing east (3×2)
  miner_east_3x3.svg         — miner facing east (3×3)
  assembler_east.svg         — assembler facing east (1×1)
  hq_east.svg                — HQ facing east (2×2)
  turret_east.svg            — turret facing east (1×1)
  storage.svg                — storage container (1×1)
  wall_h.svg                 — wall horizontal (1×1)
  wall_v.svg                 — wall vertical (1×1)
  splitter_east.svg          — splitter, 3-way (1×1)
  sorter_east.svg            — sorter, 3-way + filter (1×1)
  soldier.svg                — soldier unit
  worker.svg                 — worker unit
```

## 6. Orientation Requirements per Building Type

| Type | Orientations | Notes |
|---|---|---|
| Straight belt | 1 + code rotation | Texture rotated via `Transform` |
| Curved belt | 1 + code rotation | Same shape, just arrow direction |
| Miner | 1 + code rotation (east) | Output direction = rotation |
| Assembler | 1 + code rotation (east) | Input left / output right |
| Furnace | 1 (static) | Symmetrical, no orientation |
| Turret | 4 (east done) | Barrel direction |
| Wall | 2 (h, v) | Rectangle, 90° rotation |
| Storage | 1 (static) | No orientation, symmetrical |
| Splitter / Sorter | 1 + code rotation (east) | 3-way junction, rotation handles direction |
| HQ | 1 + code rotation (east) | 128×128 (2×2), symmetrical enough to rotate |
| Soldier / Worker | 1 | Small entity, ~48×48

## 7. Rules

### Do
- One SVG = one building type, one orientation, one variant
- Use `<g id="...">` for the 3 color layers
- Use gradients in `#base`, solids in `#owner_color` / `#level_color`
- Keep connector edges aligned (belts: entry/exit at same Y or X as straight belts)
- Use **same stroke widths** for all canvas sizes (stroke consistency)

### Don't
- No game data (items, ore, ammo, HP bars, labels) — those are separate entities in code
- No gradients in `owner_color` or `level_color` layers
- No SVG animations (Bevy renders SVGs as static textures)

### Top-Down Conventions (nouveau design system)

All SVGs now use a **top-down (bird's eye) view**:

| Rule | Detail |
|---|---|
| **Fill the tile** | Building body must fill the entire tile with 2–4 px padding. No empty space inside the footprint. |
| **Simple shapes** | Minimal details. A building must be identifiable by **silhouette + color** alone. |
| **Ports = yellow (#ffcc00)** | Belt connection ports are bright yellow rectangles, centered on the tile edge. One port per face. |
| **Depth = dark circles** | Holes/pits (e.g. drill shaft) are dark circles `#2e2e2e`–`#1a1a1a`. |
| **No profile** | No side/front views. Everything is seen from above. No vertical walls, no visible doors on fronts. |

## 8. Belt Reference

### Straight belt (`belt_east.svg`)

```
┌──────────────────┐
│ ════▶═══▶═══▶═══│  ← top rail (#555)
│ ●════●════●════● │  ← surface + roller dots
│ ════▶═══▶═══▶═══│  ← bottom rail (#555)
└──────────────────┘
```

- Belt surface : 64×48 px, centered vertically (y=8 to y=56)
- Rollers : vertical lines every 8 px, `#888`–`#999`
- Arrow : pointing in the movement direction, `#aaa` at 60 %
- Edge connectors : 2×24 px rects at entry/exit (x=0 and x=62)

### Curved belt (`belt_turn_en.svg`)

- Surface : quarter-annulus (arc path), top-right quadrant
- Uses same gray gradient, rollers are radial (pointing to curve center)
- Arrow follows the arc curve

## 9. Multi-tile Examples

### Miner 1×1 (`miner_east.svg`)

```
┌──────────────────┐
│    ╔══════╗      │
│    ║ ▓▓▓▓ ║      │  ← drill rig with cross-bracing
│    ║  ○   ║      │  ← drill bit
│    ║ ▓▓▓▓ ║      │
│    ║ ▓▓▓▓ ║      │
│    ╚══════╝      │
│  ╔══════════╗ ▐▐▐│  ← motor + output port
│  ╚══════════╝    │
└──────────────────┘
```

- Central rig, motor on foundation, port on right edge.

> **Note:** Multi-tile miner variants (`tall`, `2x2`, `3x2`, `3x3`) still use the
> legacy profile view and will be redesigned in top-down later.

## 10. Building Catalog — Top-Down Designs

All buildings use **top-down view** unless noted. Ports are bright yellow (`#ffcc00`).

### Miner 1×1 (`miner_east.svg`)

```
┌──────────────────┐
│ ● ┌──────────┐ ● │
│   │   ╭──╮   │   │
│   │   │◉◉│   │   │  ← drill shaft hole (concentric circles)
│   │   ╰──╯   │   │
│   │ ██ ██ ██ │   │  ← motor blocks (3)
│   └──────────┘   │
│██              ██│  ← yellow ports (left/right)
└──────────────────┘
```

- Square body filling the tile. Large dark circle = drill hole into ground.
- 3 motor blocks below the hole. Yellow ports on left (input) and right (output).
- `owner_color`: stripe across motor. `level_color`: badge top-right.

### Furnace 1×1 (`furnace.svg`)

```
┌──────────────────┐
│ ● ┌──────────┐ ● │
│   │  ▓▓ ▓▓   │   │
│   │  ▓ ██ ▓  │   │  ← refractory frame + glowing hearth
│   │  ▓ ██ ▓  │   │
│   │  ▓▓ ▓▓   │   │
│   └──────────┘   │
│██              ██│  ← yellow ports (left/right)
└──────────────────┘
```

- Square body with refractory inner frame. Orange center = glowing furnace hearth.
- Grate bars across the opening. Yellow ports left (input ore) and right (output plates).
- `owner_color`: stripe top. `level_color`: badge bottom-left.

### Assembler 1×1 (`assembler_east.svg`)

```
┌──────────────────┐
│ ● ┌──────────┐ ● │
│   │  ◯ ╪ ◯  │   │
│   │  ═ ● ═  │   │  ← rotating carousel + cross arm
│   │  ◯ ╪ ◯  │   │
│   └──────────┘   │
│██              ██│  ← yellow ports (left/right)
└──────────────────┘
```

- Square body with central circular assembly platform.
- Concentric rings + cross pattern = robotic arm/carousel mechanism.
- 4 end-effector dots at N/S/E/W. Yellow ports left (input) and right (output).
- `owner_color`: ring highlight. `level_color`: badge bottom-left.

### Storage 1×1 (`storage.svg`)

```
┌──────────────────┐
│██  ┌──────────┐ ██│
│    │ ▓ ▓ ▓ ▓  │   │
│██  │ ▓ ▓ ▓ ▓  │ ██│  ← 4×4 grid of storage cells
│    │ ▓ ▓ ▓ ▓  │   │
│    │ ▓ ▓ ▓ ▓  │   │
│    └──────────┘   │
│██              ██│  ← yellow ports on all 4 sides
└──────────────────┘
```

- Square body with recessed storage chamber. 4×4 grid of cells visible from above.
- Yellow ports on **all 4 edges** (N/S/E/W) — one per face.
- `owner_color`: stripe across top cells. `level_color`: badge bottom-left.

### Splitter 1×1 (`splitter_east.svg`)

```
┌──────────────────┐
│        ██        │  ← yellow top port
│   ═══════        │
│   ═══ ● ════     │  ← T-junction + central gear
│   ═══════        │
│        ●         │
│██              ██│  ← yellow side ports (left/right)
└──────────────────┘
```

- Square body with T-shaped belt surface (horizontal + branch to top).
- Central gear hub with rollers on all 3 arms. Direction arrows.
- Yellow ports on left, right, and top edges. 3-way junction.
- `owner_color`: ring around hub. `level_color`: badge top-right.

### Sorter 1×1 (`sorter_east.svg`)

```
┌──────────────────┐
│        ██        │  ← yellow top port
│   ═════ ╪ ═══    │
│   ═════ ╪ ═══    │  ← scanner/filter bar (blue)
│   ═════ ╪ ═══    │
│         ●        │
│██              ██│  ← yellow side ports (left/right)
└──────────────────┘
```

- Same T-junction layout as splitter + **scanner/filter bar** across center.
- Blue sensor dots and scanner beam (distinctive sorter feature).
- Yellow ports on left, right, and top edges.
- `owner_color`: stripe on scanner housing. `level_color`: badge top-right.

### HQ 2×2 (`hq_east.svg`)

```
┌──────────────────────────────────────┐
│  ┌────────┐  ┌────────┐             │
│  │  ╭──╮  │  │  ║     │             │
│  │  ╰──╯  │  │  ║     │  ← 4 modules + central dome
│  └────────┘  └────────┘             │
│  ┌────────────────────────────────┐ │
│  │       ◯   ◯                   │ │  ← command dome (center)
│  └────────────────────────────────┘ │
│  ┌────────┐  ┌────────┐  ██████████│
│  │        │  │        │  ← yellow ports
│  │        │  │        │             │
│  └────────┘  └────────┘             │
│██                                ██│  ← yellow ports (left/right)
└──────────────────────────────────────┘
```

- 128×128 SVG (2×2 tiles). Large square body filling all 4 tiles.
- Central command dome (concentric circles). 4 corner modules.
- Radar dish arc in top-left module. Yellow ports: 2 per side (left and right).
- `owner_color`: wide stripe across right half. `level_color`: badge top-right.

### Turret 1×1 (`turret_east.svg`)

*Design not yet updated to top-down — currently profile view.*

### Wall 1×1 (`wall_h.svg`, `wall_v.svg`)

*Design not yet updated to top-down — currently profile view.*

## 11. Units

### Soldier (`soldier.svg`)

```
┌──────────────────┐
│      ┌──┐        │  ← helmet
│      │⌠⌡│        │  ← visor
│   ┌──┼──┼──┐     │
│   │  │██│  │     │  ← shoulder pads + torso
│   ├──┴──┴──┤     │
│ ──┤  ████  │     │  ← weapon (rifle)
│   │  ████  │     │
│   └────────┘     │
│   ■         ■    │  ← legs + feet
└──────────────────┘
```

- Humanoid combat unit: helmet with visor, armored torso, shoulder pads.
- Carries a rifle in left hand.
- ~30×40 px character centered in 64×64 canvas.

### Worker (`worker.svg`)

```
┌──────────────────┐
│    ●─▀─●         │  ← antenna on head
│    ┌────┐        │
│    │ ◎◎ │        │  ← head dome + visor
│    └────┘        │
│  ┌──────────┐    │
│──┤  ● ● ●   │──  │  ← body with status lights + tool arm
│  │  ● ● ●   │    │
│  └──────────┘    │
│  ■■■■    ■■■■    │  ← tracks (treads)
└──────────────────┘
```

- Rounded construction robot: spherical body, head dome with antenna.
- Wrench tool in left arm, claw in right arm.
- Tank treads instead of legs (utility vehicle style).

## 12. Level / Owner Tint Zones — Guidelines

| Layer | Purpose | Example uses |
|---|---|---|
| `owner_color` | Team identifier | Stripe on building body, flag, accent panel |
| `level_color` | Upgrade indicator | Badge, indicator light, extra detail |

Design the white areas in these layers so they're **recognizable shapes**
even when fully tinted. Avoid isolated single-pixel dots.

If a building has no owner or level visual, simply omit the `<g>` group.
