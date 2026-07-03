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

## 2. Output Ports

Every production building has an **output port** : a dark recess on the tile
edge where a belt connects.

### Port position

The port is always at the **center of the output face**, aligned with the
belt's edge connector area:

```
  belt surface y=8..56    belt connector y=20..44 (h=24)
```

⇒ Port SVG rect: `y=20..44, h=24` **relative to the tile containing the port**.

### Port alignment table

| Building size | Port Y (in SVG) | Port H | Notes |
|---|---|---|---|---|
| 1×1 | 20–44 | 24 | Single port, center-right |
| 1×2 (tall) | 84–108 (bottom tile) | 24 | Single port, bottom tile |
| 2×2 | 84–108 (bottom-right tile) | 24 | Single port, bottom-right tile |
| 3×2 | 20–44 (top-right tile) | 24 | 2 ports (top + bottom) |
| 3×2 | 84–108 (bottom-right tile) | 24 | |
| 3×3 | 84–108 (middle-right tile) | 24 | 2 ports (middle + bottom) |
| 3×3 | 148–172 (bottom-right tile) | 24 | |

From 6 tiles onward, buildings may have **multiple ports** on different faces.

### Visual design of a port

```svg
<rect x="W-4" y="20" width="4" height="24" rx="1" fill="#2e2e2e"/>   <!-- deep recess -->
<rect x="W-3" y="22" width="3" height="20" fill="#3a3a3a"/>          <!-- inner shadow -->
<line x1="W-4" y1="20" x2="W-4" y2="44" stroke="#5a5a5a" stroke-width="0.5"/>  <!-- top highlight -->
```

Where `W` = SVG canvas width (64 for 1×1, 128 for 2×2, etc.).

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
| Straight belt | 4 (or 1 + code rotation) | Symmetrical, rotation is fine |
| Curved belt | 1 + code rotation | Same shape, just arrow direction |
| Miner | 4 (east done) | Distinct front face (output) + multi-tile variants |
| Assembler | 4 (east done) | Industrial unit, input left / output right |
| Turret | 4 (east done) | Barrel direction, base + rotating body |
| Wall | 2 (h, v) | Rectangle, 90° rotation |
| Storage | 1 | No front face, pentagonal container |
| Splitter / Sorter | 4 (east done) | 3-way junction, input left / output right + up |
| HQ | 4 (east done) | 128×128 (2×2), command center |
| Soldier / Worker | 1 | Small entity, ~30-40px

## 7. Rules

### Do
- One SVG = one building type, one orientation, one variant
- Use `<g id="...">` for the 3 color layers
- Use gradients in `#base`, solids in `#owner_color` / `#level_color`
- Keep connector edges aligned (belts: entry/exit at same Y or X as straight belts)
- Use **same stroke widths** for all canvas sizes (stroke consistency)

### Don't
- No game data (items, ore, ammo, HP bars, labels) — those are separate entities in code
- No direction arrows on non-transport buildings (miner, assembler, turret — the front face IS the direction)
- No gradients in `owner_color` or `level_color` layers
- No SVG animations (Bevy renders SVGs as static textures)

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

### Miner 1×2 (`miner_east_tall.svg`)

```
┌──────────────────┐
│    ╔══════╗      │  ← upper mast (1 tile)
│    ║ ▓▓▓▓ ║      │
│    ║ ▓▓▓▓ ║      │
│    ║  ○   ║      │
│    ║ ▓▓▓▓ ║      │
│    ╚══════╝      │
│  ╔══════════╗     │  ← deck (seam)
│  ║ ▓▓▓▓▓▓▓ ║ ▐▐▐│  ← lower processing + port
│  ║ ╔══════╗║     │  ← motor
│  ╚══════════╝    │
└──────────────────┘
```

- Taller mast (more cross-bracing), processing unit below deck, port in bottom tile.

### Miner 2×2 (`miner_east_2x2.svg`)

```
┌──────────────────────────────────────┐
│  ║ ▓▓ ▓▓ ║    ║ ▓▓ ▓▓ ║            │  ← dual-column headframe
│  ║  ○    ║    ║    ○  ║            │  ← 2 drill bits
│  ║ ▓▓ ▓▓ ║    ║ ▓▓ ▓▓ ║            │
│  ╚══════════════════════╝           │
│  ╔══════════════════════════╗ ▐▐▐▐▐▐│  ← deck + processing + port
│  ║ ▓▓ ▓▓ ▓▓ ┃ ▓▓ ▓▓ ▓▓ ▓▓ ║       │
│  ║ ╔══════════════════╗    ║       │  ← motor
│  ╚══════════════════════════╝       │
└──────────────────────────────────────┘
```

- Dual drill columns, wide processing, port at bottom-right tile.

### Miner 3×2 (`miner_east_3x2.svg`)

```
┌──────────────────────────────────────────────────────────────────┐
│  ┌──────┐   ┌──────┐   ┌──────┐                                 │
│  │ ▓▓▓▓ │   │ ▓▓▓▓ │   │ ▓▓▓▓ │  ← 3 drill columns             │
│  │  ○   │   │  ○   │   │  ○   │                                 │
│  └──────┘   └──────┘   └──────┘                                 │
│  ╔═══════════════════════════════════════╗ ▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐    │
│  ║          CONVOYEUR PRINCIPAL          ║  ← conveyor + port 1 │
│  ╚═══════════════════════════════════════╝                       │
│  ┌──────────────────────────────────────┐                        │
│  │      ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓              │                        │
│  │      ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓              │  ← processing          │
│  │      ╔══════════════════╗            │                        │
│  └──────────────────────────────────────┘ ▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐    │
│  ╔═══════════════════════════════════════╗  ← motor + port 2     │
└──────────────────────────────────────────────────────────────────┘
```

- 3 drill columns across top, conveyor, processing body, motor.
- **2 output ports** (from 6 tiles onward): top-right tile and bottom-right tile.

### Miner 3×3 (`miner_east_3x3.svg`)

```
┌──────────────────────────────────────────────────────────────────┐
│  ┌──────┐   ┌──────┐   ┌──────┐                                 │
│  │ ▓▓▓▓ │   │ ▓▓▓▓ │   │ ▓▓▓▓ │  ← 3 drill columns             │
│  │  ○   │   │  ○   │   │  ○   │                                 │
│  └──────┘   └──────┘   └──────┘                                 │
│  ╔═══════════════════════════════════════╗                       │
│  ║          CONVOYEUR PRINCIPAL          ║  ← conveyor deck      │
│  ╚═══════════════════════════════════════╝                       │
│  ┌────────┐ ┌──────────────────┐ ┌────────┐                      │
│  │ ▓ ▓ ▓  │ │  ▓▓ ▓▓ ▓▓ ▓▓   │ │ ▓ ▓ ▓  │  ← processing hall   │
│  │  ○○○   │ │  ▓▓ ▓▓ ▓▓ ▓▓   │ │  ○○○   │  with 3 units        │
│  └────────┘ └──────────────────┘ └────────┘ ▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐   │
│  ╔══════════════════════════════╗           ← deck + port 1      │
│  ║   ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓    ║                                 │
│  ║   ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓ ▓▓    ║  ← motor hall (3 blocks)        │
│  ║   ╔════════════════════════╗║                                 │
│  ╚══════════════════════════════╝ ▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐▐              │
│  ╔═══════════════════════════════════════╗  ← foundation + port 2│
└──────────────────────────────────────────────────────────────────┘
```

- 3 drill columns, processing hall with 3 units (crushers + control panel),
  motor hall with 3 large motor blocks.
- **2 output ports**: middle-right tile and bottom-right tile.
- External pipes connect the decks vertically.

## 10. Building Catalog — Other Types

### HQ 2×2 (`hq_east.svg`)

```
┌──────────────────────────────────────────────────┐
│   ╔═══════════════════╗  ┌─ radar ─┐             │
│   ║  COMMAND TOWER    ║  │         │             │
│   ║  [■■■] [■■■]     ║  └─────────┘             │
│   ╚═══════════════════╝                          │
│  ┌──────────┐   ┌────────────┐                   │
│  │ WINDOWS  │   │  GARAGE    │  ▐▐▐▐▐▐▐▐        │
│  │ WINDOWS  │   │  DOOR      │  ← output port    │
│  │ WINDOWS  │   │  VENTS     │                   │
│  └──────────┘   └────────────┘                   │
│  ═══════════════════════════════════════════════  │
│   ●       ●       ●       ●       ●       ●      │
└──────────────────────────────────────────────────┘
```

- Central tower with command windows + radar dish.
- Left wing (admin, 3 window rows), right wing (garage + vents).
- Output port on right wing, input port on left.
- 128×128 SVG (2×2 tiles).

### Assembler 1×1 (`assembler_east.svg`)

```
┌──────────────────┐
│    ┌──┐          │  ← chimney + exhaust puff
│    │  │          │
│   ╔══════════╗   │
│   ║ ┌──────┐ ║   │  ← assembly window + robot arm
│   ║ │  ◇   │ ║   │
│   ║ └──────┘ ║   │
│   ║ ● ● ●   ║   │  ← status lights on panel
│   ╚══════════╝   │
│  ▐▐▐▐▐    ▐▐▐▐▐│  ← input port / output port
└──────────────────┘
```

- Factory body with chimney, viewport showing internal arm, control panel.
- Input port on left edge, output port on right edge.

### Turret 1×1 (`turret_east.svg`)

```
┌──────────────────┐
│    ┌────┐        │
│    │ ◎  │        │  ← targeting camera / sensor
│    ╔══════╗      │
│    ║ ▓▓▓▓ ║ ▐▐▐▐│  ← rotating turret body + barrel →
│    ╚══════╝      │
│   ╔══════════╗   │  ← hexagonal base
│   ╚══════════╝   │
│  ▐▐▐▐  ●  ▐▐▐▐  │  ← ammo feed (left) + bolts
└──────────────────┘
```

- Hexagonal fixed base, rotating upper body, long barrel pointing east.
- Targeting sensor on top, ammo feed on left.

### Storage 1×1 (`storage.svg`)

```
┌──────────────────┐
│    ╔══════╗      │
│    ║  lid  ║      │  ← pentagonal container
│   ╔══════════╗   │
│   ║ ┌──────┐ ║   │
│   ║ │ DOOR │ ║   │  ← access door with handle
│   ║ │      │ ║   │
│   ║ └──────┘ ║   │
│   ╚══════════╝   │
│  ▐▐          ▐▐  │  ← input / output ports
└──────────────────┘
```

- Pentagonal container with lid seam, reinforced corners, center door.
- Input port on left, output port on right.
- Ventilation slots on right side.

### Wall 1×1 (`wall_h.svg`, `wall_v.svg`)

```
Horizontal (H):           Vertical (V):
┌──────────────────┐     ┌──────────────────┐
│ ▓▓  ▓▓  ▓▓  ▓▓  │     │ ▓▓               │
│ ▓▓  ▓▓  ▓▓  ▓▓  │     │ ▓▓               │
│ ════════════════ │     │ ▓▓   battlements  │
│ ■■■■■■■■■■■■■■ │     │ ▓▓               │
│ ■■■■■■■■■■■■■■ │     │ ▓▓               │
│                  │     │ ▓▓               │
│                  │     │ ▓▓   wall body   │
│                  │     │ ▓▓               │
│                  │     │ ▓▓               │
│                  │     │ ▓▓               │
│                  │     │ ▓▓               │
└──────────────────┘     │ ▓▓               │
                          │ ▓▓               │
                          └──────────────────┘
```

- Stone/concrete wall with battlements on top.
- Horizontal (full width, short) and vertical (full height, narrow).
- Stone block texture lines.

### Splitter 1×1 (`splitter_east.svg`)

```
┌──────────────────┐
│          ↑       │  ← lateral output (to top)
│          ▐▐      │
│          ▐▐      │
│  → → → ● ● → → →│  ← input left, straight output right
│  ▐▐     ●●   ▐▐  │
│          ▲       │
│          │       │
└──────────────────┘
```

- 3-way conveyor junction: input left, output right (straight) + output up (lateral).
- Belt rollers on all 3 arms, direction arrows, central gear hub.
- Edge connectors on left, right, and top edges.

### Sorter 1×1 (`sorter_east.svg`)

```
┌──────────────────┐
│          ↑       │  ← lateral output
│          ▐▐      │
│   ═══════════    │  ← scanner beam across belt
│  → → → [◎] → →  │  ← filter sensor (blue glow)
│  ▐▐     ●●   ▐▐  │
│          ▲       │
│          │       │
└──────────────────┘
```

- Same 3-way layout as splitter.
- Additional filter sensor/scanner unit across the belt path.
- Blue indicator lights and selection dial.

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
