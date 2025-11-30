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
    tiles: Tiles,
    frame: usize,
}
pub struct TileMap {
    data: Box<[[u16; 15]; 20]>,
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
                self.data[y][x] = self.map[offset];
            }
        }
        self.cursor = 15 * 20;
    }

    pub fn next(&mut self) {
        let mut data = Box::new([[0u16; 15]; 20]);

        for y in 0 .. 19 {
            for x in 0 .. 13 {
                data[y + 1][x] = self.data[y][x + 2];
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

        self.data = data;
    }
    pub fn new() -> Self {
        let map_source = source("./graphics/tilemap.inc");
        
        let mut data = Box::new([[0u16; 15]; 20]);
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
            data[row][column] = n;
        }
        
        Self {
            data,
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
}

impl FetchColor {
    pub fn new() -> Self {
        Self {
            map: TileMap::new(),
            tiles: Tiles::new(),
            frame: 0,
        }
    }
    pub fn next_frame(&mut self) {
        if self.frame > 160 {
            self.map.reset();
            self.frame = 0;
            return;
        }
        self.map.next();
        self.frame += 1;
    }
    pub fn skip(&mut self, n: usize) {
        for _ in 0 .. n {
            self.next_frame();
        }
    }
    pub fn skip_to(&mut self, frame: usize) {
        self.skip(frame - self.frame);
    }

    pub fn get_color(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        let tile_pos_x = x / 8;
        let tile_pos_y = y / 8;
        
        if tile_pos_x >= 15 || tile_pos_y >= 20 {
            return None;
        }
        let tile_x = x % 8;
        let tile_y = y % 8;

        let tile_id = self.map.data[tile_pos_y][tile_pos_x] as usize;

        let tile = &self.tiles.data[tile_id];

        Some(tile[tile_y][tile_x])
    }
}


pub fn create_bitmap(
    tiles: &[u8],
    tiles2: &[u8],
    fetch_color: &FetchColor,
) -> Vec<u8> {
    let mut rgb = Vec::<u8>::new();

    for y in 0..32 * 8 + 160 {
        let is_extra_space = y >= 32 * 8;
        for x in 0..32 * 8 * 2 {
            if is_extra_space {
                let animation_x = (x + 129) % 130;
                let color = fetch_color.get_color(animation_x, y % (32 * 8)).unwrap_or((255, 255, 255));
                rgb.extend(&[color.0, color.1, color.2]);
                continue;
            }
            let tile_data = if x < 32 * 8 { tiles } else { tiles2 };
            let i = x % (32 * 8) / 32;
            let j = y / 32;
            let n = i * 8 + j;
            let data = get_image(n, tile_data);
            let offset = y % 32 * 32 * 3 + x % 32 * 3;
            rgb.extend(&[data[offset], data[offset + 1], data[offset + 2]]);
        }
    }

    rgb
}
