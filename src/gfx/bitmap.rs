//! Indexed 8-bit bitmaps and 2D drawing primitives.
//! Index 0 is transparent when blitting.

#[derive(Clone, Debug, Default)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Bitmap {
    pub fn new(w: i32, h: i32) -> Bitmap {
        Bitmap { w, h, px: vec![0; (w.max(0) * h.max(0)) as usize] }
    }

    pub fn filled(w: i32, h: i32, c: u8) -> Bitmap {
        Bitmap { w, h, px: vec![c; (w.max(0) * h.max(0)) as usize] }
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if self.in_bounds(x, y) { self.px[(y * self.w + x) as usize] } else { 0 }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if self.in_bounds(x, y) {
            let i = (y * self.w + x) as usize;
            self.px[i] = c;
        }
    }

    pub fn fill(&mut self, c: u8) {
        self.px.iter_mut().for_each(|p| *p = c);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.w);
        let y1 = (y + h).min(self.h);
        for yy in y0..y1 {
            let row = (yy * self.w) as usize;
            for xx in x0..x1 {
                self.px[row + xx as usize] = c;
            }
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: u8) {
        let (a, b) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
        for x in a..=b {
            self.set(x, y, c);
        }
    }

    pub fn vline(&mut self, x: i32, y0: i32, y1: i32, c: u8) {
        let (a, b) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
        for y in a..=b {
            self.set(x, y, c);
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.hline(x, x + w - 1, y, c);
        self.hline(x, x + w - 1, y + h - 1, c);
        self.vline(x, y, y + h - 1, c);
        self.vline(x + w - 1, y, y + h - 1, c);
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn fill_circle_f(&mut self, cx: f32, cy: f32, r: f32, c: u8) {
        let x0 = (cx - r).floor() as i32;
        let x1 = (cx + r).ceil() as i32;
        let y0 = (cy - r).floor() as i32;
        let y1 = (cy + r).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                if dx * dx + dy * dy <= r * r {
                    self.set(x, y, c);
                }
            }
        }
    }

    pub fn fill_ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: u8) {
        let x0 = (cx - rx).floor() as i32;
        let x1 = (cx + rx).ceil() as i32;
        let y0 = (cy - ry).floor() as i32;
        let y1 = (cy + ry).ceil() as i32;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let dx = (x as f32 + 0.5 - cx) / rx;
                let dy = (y as f32 + 0.5 - cy) / ry;
                if dx * dx + dy * dy <= 1.0 {
                    self.set(x, y, c);
                }
            }
        }
    }

    pub fn ellipse_outline(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, c: u8) {
        let x0 = (cx - rx - 1.0).floor() as i32;
        let x1 = (cx + rx + 1.0).ceil() as i32;
        let y0 = (cy - ry - 1.0).floor() as i32;
        let y1 = (cy + ry + 1.0).ceil() as i32;
        let inside = |x: i32, y: i32| {
            let dx = (x as f32 + 0.5 - cx) / rx;
            let dy = (y as f32 + 0.5 - cy) / ry;
            dx * dx + dy * dy <= 1.0
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                if inside(x, y)
                    && (!inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1))
                {
                    self.set(x, y, c);
                }
            }
        }
    }

    /// Fill an arbitrary (possibly concave) polygon using even-odd scanlines.
    pub fn fill_polygon(&mut self, pts: &[(f32, f32)], c: u8) {
        if pts.len() < 3 {
            return;
        }
        let miny = pts.iter().map(|p| p.1).fold(f32::MAX, f32::min).floor() as i32;
        let maxy = pts.iter().map(|p| p.1).fold(f32::MIN, f32::max).ceil() as i32;
        let mut xs: Vec<f32> = Vec::new();
        for y in miny..=maxy {
            let sy = y as f32 + 0.5;
            xs.clear();
            for i in 0..pts.len() {
                let (x0, y0) = pts[i];
                let (x1, y1) = pts[(i + 1) % pts.len()];
                if (y0 <= sy && y1 > sy) || (y1 <= sy && y0 > sy) {
                    xs.push(x0 + (sy - y0) / (y1 - y0) * (x1 - x0));
                }
            }
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for pair in xs.chunks(2) {
                if pair.len() == 2 {
                    let xa = (pair[0] - 0.5).ceil() as i32;
                    let xb = (pair[1] - 0.5).floor() as i32;
                    for x in xa..=xb {
                        self.set(x, y, c);
                    }
                }
            }
        }
    }

    /// Copy with transparency (index 0 skipped).
    pub fn blit(&mut self, src: &Bitmap, x: i32, y: i32) {
        self.blit_region(src, 0, 0, src.w, src.h, x, y);
    }

    pub fn blit_region(&mut self, src: &Bitmap, sx: i32, sy: i32, w: i32, h: i32, dx: i32, dy: i32) {
        for yy in 0..h {
            let ty = dy + yy;
            if ty < 0 || ty >= self.h {
                continue;
            }
            for xx in 0..w {
                let tx = dx + xx;
                if tx < 0 || tx >= self.w {
                    continue;
                }
                let c = src.get(sx + xx, sy + yy);
                if c != 0 {
                    self.px[(ty * self.w + tx) as usize] = c;
                }
            }
        }
    }

    /// Copy everything including index 0.
    pub fn blit_opaque(&mut self, src: &Bitmap, x: i32, y: i32) {
        for yy in 0..src.h {
            let ty = y + yy;
            if ty < 0 || ty >= self.h {
                continue;
            }
            for xx in 0..src.w {
                let tx = x + xx;
                if tx < 0 || tx >= self.w {
                    continue;
                }
                self.px[(ty * self.w + tx) as usize] = src.px[(yy * src.w + xx) as usize];
            }
        }
    }

    pub fn blit_flip_x(&mut self, src: &Bitmap, x: i32, y: i32) {
        for yy in 0..src.h {
            for xx in 0..src.w {
                let c = src.get(src.w - 1 - xx, yy);
                if c != 0 {
                    self.set(x + xx, y + yy, c);
                }
            }
        }
    }

    /// Blit through a colour remap table (e.g. a shade table).
    pub fn blit_mapped(&mut self, src: &Bitmap, x: i32, y: i32, map: &[u8; 256]) {
        for yy in 0..src.h {
            for xx in 0..src.w {
                let c = src.get(xx, yy);
                if c != 0 {
                    self.set(x + xx, y + yy, map[c as usize]);
                }
            }
        }
    }

    /// Blit with every opaque pixel drawn in a single colour (silhouettes, shadows).
    pub fn blit_silhouette(&mut self, src: &Bitmap, x: i32, y: i32, col: u8) {
        for yy in 0..src.h {
            for xx in 0..src.w {
                if src.get(xx, yy) != 0 {
                    self.set(x + xx, y + yy, col);
                }
            }
        }
    }

    /// Nearest-neighbour scaled blit into the rectangle (x, y, w, h).
    pub fn blit_scaled(&mut self, src: &Bitmap, x: i32, y: i32, w: i32, h: i32) {
        if w <= 0 || h <= 0 {
            return;
        }
        for yy in 0..h {
            let sy = yy * src.h / h;
            for xx in 0..w {
                let sx = xx * src.w / w;
                let c = src.get(sx, sy);
                if c != 0 {
                    self.set(x + xx, y + yy, c);
                }
            }
        }
    }

    pub fn sub(&self, x: i32, y: i32, w: i32, h: i32) -> Bitmap {
        let mut b = Bitmap::new(w, h);
        for yy in 0..h {
            for xx in 0..w {
                b.px[(yy * w + xx) as usize] = self.get(x + xx, y + yy);
            }
        }
        b
    }

    pub fn flipped_x(&self) -> Bitmap {
        let mut b = Bitmap::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                b.px[(y * self.w + x) as usize] = self.get(self.w - 1 - x, y);
            }
        }
        b
    }

    /// Add a 1-pixel outline of colour `c` around all opaque pixels
    /// (outside the shape). Grows the bitmap by 1 pixel on every side if `grow`.
    pub fn outlined(&self, c: u8, grow: bool) -> Bitmap {
        let g = if grow { 1 } else { 0 };
        let mut b = Bitmap::new(self.w + 2 * g, self.h + 2 * g);
        b.blit(self, g, g);
        let src = b.clone();
        for y in 0..b.h {
            for x in 0..b.w {
                if src.get(x, y) == 0
                    && (src.get(x - 1, y) != 0
                        || src.get(x + 1, y) != 0
                        || src.get(x, y - 1) != 0
                        || src.get(x, y + 1) != 0)
                {
                    b.set(x, y, c);
                }
            }
        }
        b
    }

    /// Replace colours via a map.
    pub fn remap(&mut self, map: &[u8; 256]) {
        for p in self.px.iter_mut() {
            *p = map[*p as usize];
        }
    }

    /// Parse ASCII pixel art. Each char maps to a palette index via `key`;
    /// chars not in the key (and ' ' / '.') are transparent.
    pub fn from_ascii(rows: &[&str], key: &[(char, u8)]) -> Bitmap {
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        let mut b = Bitmap::new(w, h);
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(&(_, c)) = key.iter().find(|(k, _)| *k == ch) {
                    b.set(x as i32, y as i32, c);
                }
            }
        }
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blit_respects_transparency_and_clipping() {
        let mut dst = Bitmap::filled(4, 4, 9);
        let mut src = Bitmap::new(2, 2);
        src.set(0, 0, 5);
        dst.blit(&src, 3, 3);
        assert_eq!(dst.get(3, 3), 5);
        assert_eq!(dst.get(2, 2), 9);
        dst.blit(&src, -1, -1);
        assert_eq!(dst.get(0, 0), 9); // (1,1) of src is transparent
    }

    #[test]
    fn outline_grows() {
        let mut b = Bitmap::new(1, 1);
        b.set(0, 0, 7);
        let o = b.outlined(1, true);
        assert_eq!((o.w, o.h), (3, 3));
        assert_eq!(o.get(1, 1), 7);
        assert_eq!(o.get(0, 1), 1);
        assert_eq!(o.get(0, 0), 0);
    }

    #[test]
    fn polygon_fill_square() {
        let mut b = Bitmap::new(10, 10);
        b.fill_polygon(&[(2.0, 2.0), (6.0, 2.0), (6.0, 6.0), (2.0, 6.0)], 3);
        let n = b.px.iter().filter(|&&p| p == 3).count();
        assert_eq!(n, 16);
    }
}
