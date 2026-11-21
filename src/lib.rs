//! Reader and writer for Warcraft III pathing maps, `war3map.wpm`: for
//! every cell of a map's grid, four to a terrain cell (32 world units),
//! whether units can walk, fly, build or swim there, and whether it's
//! blighted or the map's boundary.
//!
//! ```no_run
//! let bytes = std::fs::read("war3map.wpm").unwrap();
//! let map = wpm_rs::parse(&bytes).unwrap();
//! let rgba = map.to_rgba();
//! assert_eq!(rgba.len(), (map.width * map.height * 4) as usize);
//! assert_eq!(map.to_bytes().unwrap(), bytes);
//! ```
//!
//! Reading and writing are exact. There's no text in it, so its form for
//! people is a picture ([`Pathing::to_rgba`]), as the game's own pathing
//! textures draw pathing.

pub use parser_rs::{BinError, BinResult};
use parser_rs::bin::{Reader, Writer};

/// `MP3W`.
pub const MAGIC: [u8; 4] = *b"MP3W";

/// The format version this crate reads and writes: the only one there is.
pub const VERSION: u32 = 0;

/// Cells per terrain cell, across and up.
pub const PER_TERRAIN_CELL: u32 = 4;

/// The size of a cell in world units.
pub const CELL: f32 = 32.0;

/// The flags of a cell, by bit, with their names. A set flag forbids:
/// `0x02` is "can't walk here". `0x01` and `0x10` are kept as they are;
/// nothing uses them.
pub const FLAGS: [(u8, &str); 8] = [
    (0x01, "unused 0x01"),
    (0x02, "no walking"),
    (0x04, "no flying"),
    (0x08, "no building"),
    (0x10, "unused 0x10"),
    (0x20, "blight"),
    (0x40, "no water"),
    (0x80, "boundary"),
];

/// Units can't walk here.
pub const NO_WALK: u8 = 0x02;
/// Units can't fly here.
pub const NO_FLY: u8 = 0x04;
/// Nothing can be built here.
pub const NO_BUILD: u8 = 0x08;
/// Blighted ground.
pub const BLIGHT: u8 = 0x20;
/// Not water: ships can't sail here.
pub const NO_WATER: u8 = 0x40;
/// The map's boundary.
pub const BOUNDARY: u8 = 0x80;

/// The names of the flags set in `cell`, in bit order.
pub fn flag_names(cell: u8) -> Vec<&'static str> {
    FLAGS
        .iter()
        .filter(|(bit, _)| cell & bit != 0)
        .map(|(_, name)| *name)
        .collect()
}

/// A whole `war3map.wpm`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pathing {
    /// Always [`VERSION`].
    pub version: u32,
    /// Cells across and up: four per terrain cell.
    pub width: u32,
    pub height: u32,
    /// `width * height` cells, row by row from the bottom, as the terrain's
    /// points are.
    pub cells: Vec<u8>,
}

impl Pathing {
    /// The cell at column `x`, row `y` (from the bottom left).
    pub fn cell(&self, x: u32, y: u32) -> Option<u8> {
        (x < self.width && y < self.height).then(|| self.cells[(y * self.width + x) as usize])
    }

    /// A picture of it, `width * height` RGBA pixels, rows top to bottom
    /// (north up), the way the game's pathing textures draw pathing: red
    /// where units can't walk, green where they can't fly, blue where
    /// nothing can be built, so white forbids all three and black none.
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.cells.len() * 4);
        for row in self.cells.chunks(self.width.max(1) as usize).rev() {
            for &c in row {
                let on = |bit: u8| if c & bit != 0 { 255 } else { 0 };
                out.extend([on(NO_WALK), on(NO_FLY), on(NO_BUILD), 255]);
            }
        }
        out
    }

    /// The file's bytes.
    pub fn to_bytes(&self) -> BinResult<Vec<u8>> {
        let count = self.width as usize * self.height as usize;
        if self.cells.len() != count {
            return Err(BinError::new(
                0,
                format!(
                    "{} cells for a {}x{} grid, which has {count}",
                    self.cells.len(),
                    self.width,
                    self.height
                ),
            ));
        }
        let mut w = Writer::new();
        w.bytes(&MAGIC);
        w.u32(self.version);
        w.u32(self.width);
        w.u32(self.height);
        w.bytes(&self.cells);
        Ok(w.into_bytes())
    }
}

/// Reads a `war3map.wpm`. Every byte must be read.
pub fn parse(bytes: &[u8]) -> BinResult<Pathing> {
    let mut r = Reader::new(bytes);
    if r.array::<4>()? != MAGIC {
        return Err(BinError::new(0, "not a pathing map: it doesn't start with MP3W"));
    }
    let version = r.u32()?;
    if version != VERSION {
        return Err(BinError::new(
            4,
            format!("unknown wpm version {version} (0 is the only one)"),
        ));
    }
    let width = r.u32()?;
    let height = r.u32()?;
    let count = width as usize * height as usize;
    if count > r.remaining() {
        return Err(r.error(format!(
            "a {width}x{height} grid needs {count} bytes, {} are left",
            r.remaining()
        )));
    }
    let cells = r.bytes(count)?.to_vec();
    r.finish()?;
    Ok(Pathing {
        version,
        width,
        height,
        cells,
    })
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
