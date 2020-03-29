use rect_packer::DensePacker;

#[inline]
fn next_pow2(n: i32) -> i32 {
    2i32.pow((n as f32).log2().ceil() as _)
}

pub struct AtlasBuilder {
    max_size: i32,
    packers: Vec<DensePacker>,
    textures: Vec<(AtlasRef, Box<[u8]>)>,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct AtlasRef {
    pub atlas_id: u32,

    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,

    // Normalized to 0-1 by texture width and height
    pub origin_x: f32,
    pub origin_y: f32,
}

impl AtlasBuilder {
    pub fn new(max_size: i32) -> Self {
        assert_eq!(max_size, next_pow2(max_size));
        AtlasBuilder { max_size, packers: Vec::new(), textures: Vec::new() }
    }

    pub fn texture(
        &mut self,
        width: i32,
        height: i32,
        origin_x: i32,
        origin_y: i32,
        data: Box<[u8]>,
    ) -> Option<AtlasRef> {
        fn to_texture(
            id: u32,
            rect: rect_packer::Rect,
            data: Box<[u8]>,
            origin_x: i32,
            origin_y: i32,
        ) -> (AtlasRef, Box<[u8]>) {
            (
                AtlasRef {
                    atlas_id: id,
                    w: rect.width,
                    h: rect.height,
                    x: rect.x,
                    y: rect.y,
                    origin_x: (origin_x as f32 / rect.width as f32),
                    origin_y: (origin_y as f32 / rect.height as f32),
                },
                data,
            )
        }

        if width > self.max_size || height > self.max_size {
            return None
        }

        for (id, packer) in self.packers.iter_mut().enumerate() {
            if let Some(rect) = packer.pack(width, height, false) {
                let (atlas_ref, data) = to_texture(id as _, rect, data, origin_x, origin_y);
                self.textures.push((atlas_ref.clone(), data));
                return Some(atlas_ref)
            } else {
                loop {
                    let (pwidth, pheight) = packer.size();
                    if pwidth <= pheight && (pwidth * 2) <= self.max_size {
                        packer.resize(pwidth * 2, pheight);
                    } else if (pheight * 2) <= self.max_size {
                        packer.resize(pwidth, pheight * 2);
                    } else {
                        break
                    }

                    if let Some(rect) = packer.pack(width, height, false) {
                        let (atlas_ref, data) = to_texture(id as _, rect, data, origin_x, origin_y);
                        self.textures.push((atlas_ref.clone(), data));
                        return Some(atlas_ref)
                    }
                }
            }
        }

        let size = 4096.min(self.max_size);
        self.packers.push(DensePacker::new(size, size));
        self.texture(width, height, origin_x, origin_y, data)
    }

    #[allow(clippy::type_complexity)] // It's for the Renderer only.
    pub fn into_inner(self) -> (Vec<DensePacker>, Vec<(AtlasRef, Box<[u8]>)>) {
        (self.packers, self.textures)
    }
}
