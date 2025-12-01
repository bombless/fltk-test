use std::fs;

pub fn source(path: &str) -> Vec<u8> {
    let content = fs::read_to_string(path).unwrap();
    let mut result = Vec::new();

    for line in content.lines() {
        let after_db = line.split("db ").nth(1).unwrap_or("");
        for item in after_db.split(',') {
            let trimmed = item.trim();
            if trimmed.len() >= 4 {
                let c1 = trimmed.chars().nth(1).unwrap();
                let c2 = trimmed.chars().nth(2).unwrap();
                if let (Some(v1), Some(v2)) = (c1.to_digit(16), c2.to_digit(16)) {
                    result.push((v1 * 16 + v2) as u8);
                }
            }
        }
    }
    result
}

fn get_image(n: usize, data: &[u8]) -> Vec<u8> {
    let mut img = Vec::new();
    let offset = n * 32 * 32 * 4;
    for i in 0 .. 32 {
        for j in 0 .. 32 {
            let idx = offset + (i * 32 + j) * 4;
            img.extend_from_slice(&[data[idx + 2], data[idx + 1], data[idx]]);
        }
    }
    img
}

pub struct FetchColor {
    map: TileMap,
    cache: Box<[[u16; 17]; 32]>,
    tiles: Tiles,
    frame: usize,
}
pub struct TileMap {
    tilemap0: Box<[[u16; 15]; 20]>,
    tilemap1: Box<[[u16; 32]; 32]>,
    map: Vec<u16>,
    cursor: usize,
}

pub struct Tiles {
    data: Vec<[[(u8, u8, u8); 8]; 8]>,
}

impl TileMap {

    fn reset(&mut self) {
        for y in 0 .. 20 {
            for x in 0 .. 15 {
                let offset = y * 15 + x;
                self.tilemap0[y][x] = self.map[offset];
            }
        }
        self.cursor = 15 * 20;
    }

    pub fn next(&mut self) {
        let mut data = Box::new([[0u16; 15]; 20]);

        for y in 0 .. 19 {
            for x in 0 .. 13 {
                data[y + 1][x] = self.tilemap0[y][x + 2];
            }
        }

        for offset in 0 .. 15 {
            data[0][offset] = self.map[self.cursor];
            self.cursor += 1;
        }
        for y in 1 .. 20 {
            for x in 13 .. 15 {
                data[y][x] = self.map[self.cursor];
                self.cursor += 1;
            }

        }

        self.tilemap0 = data;
    }
    pub fn new() -> Self {
        let map_source = source("./graphics/tilemap.inc");
        
        let mut tilemap0 = Box::new([[0u16; 15]; 20]);
        
        let mut tilemap1 = Box::new([[0u16; 32]; 32]);

        let mut map = Vec::new();
        for i in 0..map_source.len() / 2 {
            let lo = map_source[i * 2] as u16;
            let hi = map_source[i * 2 + 1] as u16;
            let n = lo + (hi << 8);
            map.push(n);

            if i >= 15 * 20 {
                continue;
            }
            let column = i % 15;
            let row = i / 15;
            tilemap0[row][column] = n;
        }

        // TITLECARD		DW	0612H, 060FH, 0600H, 0602H, 0604H, 05EDH, 0606H, 0600H, 060CH, 0604H
		//		DW	05EDH, 0605H, 060EH, 0611H, 05EDH, 0617H, 0620H, 061EH

        let tile_card = &[0x0612, 0x060f, 0x0600, 0x0602, 0x0604, 0x05ed, 0x0606, 0x0600, 0x060c, 0x0604, 0x05ed, 0x0605, 0x060e, 0x0611, 0x05ed, 0x0617, 0x0620, 0x061e];
        let mut cursor = 0x18 / 2;
        for tile in tile_card {
            let x = cursor % 32;
            let y = cursor / 32;
            tilemap1[y][x] = *tile;
            cursor += 1;
        }

        let mut offset = 0x0204 / 2;
        let offset_x = offset % 32;
        let offset_y = offset / 32;
        tilemap1[offset_y][offset_x] = 0x064d;

        offset += 0x0140 / 2;
        let offset_x = offset % 32;
        let offset_y = offset / 32;
        tilemap1[offset_y][offset_x] = 0x0643;



        
        Self {
            tilemap0,
            tilemap1,
            map,
            cursor: 15 * 20,
        }
    }

}

impl Tiles {
    pub fn new() -> Self {
        let raw = source("./graphics/bgTiles.inc");
        let mut data = Vec::new();
        for slice in raw.chunks(8 * 8 * 4) {
            let mut tile = Box::new([[(0, 0, 0); 8]; 8]);
            for (y, line) in slice.chunks(8 * 4).enumerate() {
                for (x, pixel) in line.chunks(4).enumerate() {
                    tile[y][x] = (pixel[2], pixel[1], pixel[0]); // BGR -> RGB
                }
            }
            data.push(*tile);
        }
        Self { data }
    }

    pub fn data(&self) -> &[[[(u8, u8, u8); 8]; 8]] {
        &self.data
    }

    pub fn alphabet(&self, n: usize) -> &[[(u8, u8, u8); 8]; 8] {
        &self.data[0x600 + n]
    }

    pub fn dec_digit(&self, n: usize) -> &[[(u8, u8, u8); 8]; 8] {
        &self.data[0x600 + 26 + n]
    }

    pub fn hex_digit(&self, n: usize) -> &[[(u8, u8, u8); 8]; 8] {
        if n < 10 {
            self.dec_digit(n)
        } else {
            self.alphabet(n - 10)
        }
    }
}

impl FetchColor {
    pub fn new() -> Self {
        Self {
            map: TileMap::new(),
            tiles: Tiles::new(),
            frame: 0,
            cache: Default::default(),
        }
    }
    pub fn next_frame(&mut self) {
        if self.frame > 160 {
            self.map.reset();
            self.frame = 0;
            return;
        }
        for y in (0 .. 31).rev() {
            for x in 0 .. 15 {
                self.cache[y + 1][x] = self.cache[y][x + 2];
            }
        }
        for y in 0 .. 20 {
            self.cache[y + 2][15] = self.map.tilemap0[y][0];
            self.cache[y + 2][16] = self.map.tilemap0[y][1];
        }
        self.map.next();
        self.frame += 1;
    }

    pub fn skip_to(&mut self, target_frame: usize) {
        for _ in self.frame .. target_frame {
            self.next_frame();
        }
    }

    pub fn get_color(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        let tile_pos_x = x / 8;
        let tile_pos_y = y / 8;
        
        if tile_pos_x >= 32 || tile_pos_y >= 32 {
            return None;
        }
        let tile_x = x % 8;
        let tile_y = y % 8;

        let tile1_id = self.map.tilemap1[tile_pos_y][tile_pos_x] as usize;

        let tile_id = if tile1_id > 0 {
            tile1_id
        } else {
            if tile_pos_x < 17 && tile_pos_y < 32 {
                self.cache[tile_pos_y][tile_pos_x] as usize
            } else if tile_pos_x >= 17 {
                let tile_pos_x = tile_pos_x - 17;
                if tile_pos_y < 1 {
                    return None;
                }
                let tile_pos_y = tile_pos_y - 1;
                if tile_pos_x >= 15 || tile_pos_y >= 20 {
                    return None;
                }
                self.map.tilemap0[tile_pos_y][tile_pos_x] as usize
            } else {
                return None;
            }
        };

        let tile = &self.tiles.data[tile_id];

        Some(tile[tile_y][tile_x])
    }
}


pub fn create_bitmap(
    tiles: &[u8],
    tiles2: &[u8],
) -> Box<[[(u8, u8, u8); 512]; 256]> {
    let mut rgb = Box::new([[(0, 0, 0); 512]; 256]);

    for y in 0..32 * 8 {
        for x in 0..32 * 8 * 2 {
            let tile_data = if x < 32 * 8 { tiles } else { tiles2 };
            let i = x % (32 * 8) / 32;
            let j = y / 32;
            let n = i * 8 + j;
            let data = get_image(n, tile_data);
            let offset = y % 32 * 32 * 3 + x % 32 * 3;
            rgb[y][x] = (data[offset], data[offset + 1], data[offset + 2]);
        }
    }

    rgb
}
