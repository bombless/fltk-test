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
pub struct TileMap {
    raw: Vec<u8>,
    map: Vec<usize>,
    map_width: usize,   // 地图宽度（tile 数）
    map_height: usize,  // 地图高度（tile 数）
    tile_size: usize,   // 每个 tile 的像素大小
}

impl TileMap {
    pub fn new() -> Self {
        let raw = source("./graphics/bgTiles.inc");
        let map_source = source("./graphics/tilemap.inc");
        
        let mut map = Vec::new();
        for i in 0..map_source.len() / 2 {
            let lo = map_source[i * 2] as usize;
            let hi = map_source[i * 2 + 1] as usize;
            let n = lo + (hi << 8);
            map.push(n);
        }
        
        Self {
            raw,
            map,
            map_width: 15,
            map_height: 20,
            tile_size: 8,
        }
    }

}

impl TileMap {
    fn get_color(&self, frame: usize, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        let tile_size = self.tile_size;
        
        // ========================================
        // 根据汇编逻辑计算滚动偏移
        // ========================================
        // INC SCROLLX              ; 每帧 +1
        // TEST RAX, 01H
        // JNZ JUSTXSCROLL
        // INC SCROLLY              ; 偶数帧才 +1（每2帧+1）
        
        let total_scroll_x = frame;           // X: 每帧滚动1像素
        let total_scroll_y = frame / 2;       // Y: 每2帧滚动1像素
        
        // ========================================
        // 计算地图坐标（屏幕坐标 + 滚动偏移）
        // ========================================
        // 屏幕向左上滚动 = 地图坐标向右下移动
        let map_x = x + total_scroll_x;
        let map_y = y + total_scroll_y;
        
        // ========================================
        // 转换为 tile 坐标
        // ========================================
        let tile_x = map_x / tile_size;
        let tile_y = map_y / tile_size;
        
        // 边界检查
        if tile_x >= self.map_width || tile_y >= self.map_height {
            return None;
        }
        
        // 计算 tile 索引（行优先）
        let tile_idx = tile_y * self.map_width + tile_x;
        if tile_idx >= self.map.len() {
            return None;
        }
        let tile_id = self.map[tile_idx];
        
        // ========================================
        // 计算 tile 内像素位置
        // ========================================
        let px = map_x % tile_size;
        let py = map_y % tile_size;
        
        // tile 数据偏移（每个 tile = tile_size × tile_size × 4 字节）
        let bytes_per_tile = tile_size * tile_size * 4;
        let offset = tile_id * bytes_per_tile;
        let pixel_offset = (py * tile_size + px) * 4;
        
        let total_offset = offset + pixel_offset;
        if total_offset + 2 >= self.raw.len() {
            return None;
        }
        
        let buffer = &self.raw[total_offset..];
        Some((buffer[2], buffer[1], buffer[0]))  // BGR -> RGB
    }
}

pub fn create_bitmap(
    index: usize, 
    tiles: &[u8],
    tiles2: &[u8],
    bg_tiles: &TileMap,
) -> Vec<u8> {
    let mut rgb = Vec::<u8>::new();

    for y in 0..32 * 8 * 2 {
        let is_extra_space = y < 32 * 8;
        for x in 0..32 * 8 * 2 {
            if is_extra_space {
                if x >= 120 {
                    rgb.extend(&[255, 255, 255]);
                    continue;
                }
                let color = bg_tiles.get_color(index, x % 120, y).unwrap_or((0, 0, 0));
                rgb.extend(&[color.0, color.1, color.2]);
                continue;
            }
            let tile_data = if x < 32 * 8 { tiles } else { tiles2 };
            let i = x % (32 * 8) / 32;
            let j = y % (32 * 8) / 32;
            let n = i * 8 + j;
            let data = get_image(n, tile_data);
            let offset = y % 32 * 32 * 3 + x % 32 * 3;
            rgb.extend(&[data[offset], data[offset + 1], data[offset + 2]]);
        }
    }

    rgb
}
