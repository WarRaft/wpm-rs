use super::*;

/// A 3x2 map: the bottom row walkable, the top one not.
fn small() -> Pathing {
    Pathing {
        version: 0,
        width: 3,
        height: 2,
        cells: vec![0x40, 0x48, 0x40, 0xCA, 0xCE, NO_BUILD | NO_FLY],
    }
}

#[test]
fn reads_and_writes_back() {
    let m = small();
    let bytes = m.to_bytes().unwrap();
    assert_eq!(&bytes[..16], b"MP3W\0\0\0\0\x03\0\0\0\x02\0\0\0");
    assert_eq!(parse(&bytes).unwrap(), m);
    assert_eq!(m.cell(2, 1), Some(NO_BUILD | NO_FLY));
    assert_eq!(m.cell(3, 0), None);
}

#[test]
fn the_picture_is_north_up_red_green_blue() {
    let rgba = small().to_rgba();
    // The top row first: 0xCA can't walk or build; 0xCE nothing.
    assert_eq!(&rgba[..12], &[255, 0, 255, 255, 255, 255, 255, 255, 0, 255, 255, 255]);
    // The bottom row: 0x40 allows all; 0x48 no building.
    assert_eq!(&rgba[12..20], &[0, 0, 0, 255, 0, 0, 255, 255]);
    assert_eq!(flag_names(0xCA), ["no walking", "no building", "no water", "boundary"]);
}

#[test]
fn bad_files() {
    let err = |b: &[u8]| parse(b).unwrap_err().to_string();
    assert!(err(b"W3E!\0\0\0\0").contains("doesn't start with MP3W"));
    assert!(err(b"MP3W\x01\0\0\0").contains("unknown wpm version 1"));
    assert!(err(b"MP3W\0\0\0\0\xff\xff\0\0\xff\xff\0\0").contains("needs"));
    let mut bytes = small().to_bytes().unwrap();
    bytes.push(0);
    assert!(err(&bytes).contains("extra byte"));
    let mut m = small();
    m.cells.pop();
    assert!(m.to_bytes().is_err());
}

/// Every map's pathing reads and writes back to the same bytes:
/// `WPM_CORPUS` is a folder of `<map>/war3map.wpm`. Its size is four
/// cells to each terrain cell, when `war3map.w3e` is beside it.
#[test]
#[ignore]
fn corpus() {
    let dir = std::env::var("WPM_CORPUS").expect("WPM_CORPUS");
    let (mut n, mut bad) = (0, Vec::new());
    for entry in std::fs::read_dir(dir).unwrap() {
        let folder = entry.unwrap().path();
        let Ok(bytes) = std::fs::read(folder.join("war3map.wpm")) else { continue };
        n += 1;
        match parse(&bytes) {
            Ok(m) if m.to_bytes().unwrap() != bytes => bad.push(format!("{}: differs", folder.display())),
            Ok(m) => {
                // war3map.w3e: its width and height, after the lists.
                if let Ok(w3e) = std::fs::read(folder.join("war3map.w3e")) {
                    let u32_at = |i: usize| u32::from_le_bytes(w3e[i..i + 4].try_into().unwrap());
                    let ground = u32_at(13) as usize;
                    let cliffs = u32_at(17 + 4 * ground) as usize;
                    let at = 21 + 4 * ground + 4 * cliffs;
                    let (w, h) = (u32_at(at), u32_at(at + 4));
                    if (m.width, m.height) != ((w - 1) * 4, (h - 1) * 4) {
                        bad.push(format!("{}: {}x{} for terrain {w}x{h}", folder.display(), m.width, m.height));
                    }
                }
            }
            Err(e) => bad.push(format!("{}: {e}", folder.display())),
        }
    }
    println!("{n} files");
    assert!(bad.is_empty(), "{} of {n}:\n{}", bad.len(), bad.join("\n"));
}
