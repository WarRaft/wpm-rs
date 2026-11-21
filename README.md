# wpm-rs

Reader and writer for Warcraft III pathing maps, `war3map.wpm`: for every
cell of a map's grid, four to a terrain cell (32 world units), whether
units can walk, fly, build or sail there, whether it's blighted, and
whether it's the map's boundary. A library and a small CLI. There's no
text in it, so its form for people is a picture: the
[wc3-rs](../wc3-rs) editor shows it in its texture viewer, and the
[wc3-mcp](../wc3-mcp) server's `wpm_image` gives it to a model as a PNG.

Reading and writing are exact: `to_bytes` gives back the bytes `parse`
read, on every map of the game, a corpus of custom maps and the maps in
`maps/` (208 files). Each is four times its terrain's size in cells
(`war3map.w3e`'s points less one, times four), across and up.

## Format

Little-endian.

| type | field |
|---|---|
| char[4] | `MP3W` |
| u32 | format version: 0, the only one |
| u32 | width: cells across, the map's terrain cells × 4 |
| u32 | height: cells up, × 4 too |
| u8 × width × height | each cell's flags, row by row from the bottom left, as the terrain's points are |

A set flag forbids:

| bit | flag |
|---|---|
| 0x01 | unused (kept as it is) |
| 0x02 | no walking |
| 0x04 | no flying |
| 0x08 | no building |
| 0x10 | unused (kept as it is) |
| 0x20 | blight |
| 0x40 | no water: ships can't sail |
| 0x80 | boundary |

## Library

```rust
let map = wpm_rs::parse(&std::fs::read("war3map.wpm")?)?;
let walkable = map.cell(10, 20).is_some_and(|c| c & wpm_rs::NO_WALK == 0);
let rgba = map.to_rgba(); // width * height pixels, north up
```

`Pathing::to_rgba` draws it as the game's pathing textures draw pathing:
red where units can't walk, green where they can't fly, blue where
nothing can be built (white: none of the three; black: all of them),
rows top to bottom with north up. `flag_names` names a cell's flags.

## CLI

```sh
wpm-rs war3map.wpm          # read and write back: a check
wpm-rs --info war3map.wpm   # the size, and how many cells have each flag
```

## Tests

```sh
cargo test
WPM_CORPUS=<folder of map folders> cargo test --release corpus -- --ignored
```

The corpus test also checks each map's size against its `war3map.w3e`.
