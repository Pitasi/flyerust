pub mod gfx {
    use image::imageops::{overlay, resize, FilterType};
    use image::io::Reader as ImageReader;
    use image::{GenericImageView, ImageError, Rgba, RgbaImage};
    use imageproc::drawing::{draw_text_mut, text_size};
    use rusttype::Font;

    pub struct DynamicCanvas {
        layers: Vec<PositionedLayer>,
    }

    impl DynamicCanvas {
        pub fn new() -> Self {
            Self { layers: Vec::new() }
        }

        pub fn add_layer(self: &mut Self, layer: Layer, pos_x: Position, pos_y: Position) {
            self.layers.push(PositionedLayer::new(layer, pos_x, pos_y));
        }

        pub fn width(self: &Self) -> u32 {
            let mut width = 0;
            for layer in &self.layers {
                let layer_width = match layer.pos_x {
                    Position::Coord(x) => x + layer.layer.buffer.width() as i32,
                    Position::Center => layer.layer.buffer.width() as i32,
                };
                if layer_width > width {
                    width = layer_width;
                }
            }

            if width < 0 {
                0
            } else {
                width as u32
            }
        }

        pub fn height(self: &Self) -> u32 {
            let mut height = 0;
            for layer in &self.layers {
                let layer_height = match layer.pos_y {
                    Position::Coord(y) => y + layer.layer.buffer.height() as i32,
                    Position::Center => layer.layer.buffer.height() as i32,
                };
                if layer_height > height {
                    height = layer_height;
                }
            }

            if height < 0 {
                0
            } else {
                height as u32
            }
        }

        pub fn dimensions(self: &Self) -> (u32, u32) {
            (self.width(), self.height())
        }
    }

    pub struct Canvas {
        base: Layer,
        layers: Vec<PositionedLayer>,
        mask: Option<RgbaImage>,
    }

    impl Canvas {
        pub fn new(width: u32, height: u32) -> Self {
            let image = RgbaImage::new(width, height);
            let base = Layer::from_buffer(image);
            Self {
                base,
                layers: Vec::new(),
                mask: None,
            }
        }

        pub fn add_layer(self: &mut Self, layer: Layer, pos_x: Position, pos_y: Position) {
            self.layers.push(PositionedLayer::new(layer, pos_x, pos_y));
        }

        pub fn set_mask(self: &mut Self, mask: RgbaImage) {
            self.mask = Some(mask);
        }

        pub fn add_dynamic_canvas(
            self: &mut Self,
            canvas: DynamicCanvas,
            pos_x: Position,
            pos_y: Position,
        ) {
            let (width, height) = canvas.dimensions();

            let abs_pos_x = pos_x.to_coord((self.base.buffer.width() as i32 - width as i32) / 2);
            let abs_pos_y = pos_y.to_coord((self.base.buffer.height() as i32 - height as i32) / 2);

            for layer in canvas.layers {
                let (layer_width, layer_height) = layer.layer.buffer.dimensions();
                let layer_abs_pos_x = layer
                    .pos_x
                    .to_coord((width as i32 - layer_width as i32) / 2);
                let layer_abs_pos_y = layer
                    .pos_y
                    .to_coord((height as i32 - layer_height as i32) / 2);
                self.add_layer(
                    layer.layer,
                    Position::Coord(abs_pos_x + layer_abs_pos_x),
                    Position::Coord(abs_pos_y + layer_abs_pos_y),
                );
            }
        }

        pub fn rasterize(self: Self) -> Layer {
            let mut base = self.base;
            for mut layer in self.layers {
                let pos_x: i64 = match layer.pos_x {
                    Position::Coord(x) => x.into(),
                    Position::Center => {
                        (base.buffer.width() as i64 - layer.layer.buffer.width() as i64) / 2
                    }
                };
                let pos_y: i64 = match layer.pos_y {
                    Position::Coord(y) => y.into(),
                    Position::Center => {
                        (base.buffer.height() as i64 - layer.layer.buffer.height() as i64) / 2
                    }
                };

                // merge_layers(&mut base, layer); not sure why my implementation works but grays are slightly different
                overlay(
                    &mut base.buffer,
                    &mut layer.layer.buffer,
                    pos_x as i64,
                    pos_y as i64,
                )
            }

            match self.mask {
                Some(mask) => apply_mask(base, mask),
                None => base,
            }
        }
    }

    pub enum Position {
        Coord(i32),
        Center,
    }

    impl Position {
        pub fn to_coord(self: &Self, size: i32) -> i32 {
            match self {
                Position::Coord(x) => *x,
                Position::Center => size / 2,
            }
        }
    }

    pub struct PositionedLayer {
        layer: Layer,
        pos_x: Position,
        pos_y: Position,
    }

    impl PositionedLayer {
        pub fn new(layer: Layer, pos_x: Position, pos_y: Position) -> Self {
            Self {
                layer,
                pos_x,
                pos_y,
            }
        }
    }

    pub struct Layer {
        buffer: RgbaImage,
    }

    impl Layer {
        pub fn from_file(path: &str) -> Result<Self, ImageError> {
            let buffer = ImageReader::open(path)?.decode()?.to_rgba8();
            Ok(Self { buffer })
        }

        pub fn from_buffer(buffer: RgbaImage) -> Self {
            Self { buffer }
        }

        pub fn resize(self: &mut Self, nwidth: u32) {
            let nheight = self.buffer.height() * nwidth / self.buffer.width();
            self.buffer = resize(&self.buffer, nwidth, nheight, FilterType::Gaussian);
        }

        pub fn save(&self, path: &str) -> Result<(), ImageError> {
            self.buffer.save(path)
        }
    }

    pub struct TextLayer<'a> {
        text: String,
        font: &'a Font<'a>,
        size: f32,
        color: Rgba<u8>,
    }

    impl<'a> TextLayer<'a> {
        pub fn new(text: &str, font: &'a Font<'a>, size: f32, color: Rgba<u8>) -> Self {
            Self {
                text: text.to_string(),
                font,
                size,
                color,
            }
        }

        pub fn to_layer(self: Self) -> Layer {
            let text = self.text.as_str();
            let scale = rusttype::Scale {
                x: self.size,
                y: self.size,
            };

            let (size_x, size_y) = text_size(scale, &self.font, text);

            let mut layer = Layer::from_buffer(RgbaImage::new(size_x as u32, size_y as u32));
            draw_text_mut(
                &mut layer.buffer,
                self.color,
                0,
                0,
                scale,
                &self.font,
                self.text.as_str(),
            );

            layer
        }
    }

    fn apply_mask(image: Layer, mask: RgbaImage) -> Layer {
        let mut result = RgbaImage::new(mask.width(), mask.height());

        for pix in result.pixels_mut() {
            *pix = Rgba([255, 255, 255, 0]);
        }

        for (x, y, pixel) in result.enumerate_pixels_mut() {
            if !image.buffer.in_bounds(x, y) {
                continue;
            }

            let mask_pixel = mask.get_pixel(x, y);
            let alpha = mask_pixel[3] as f32 / 255.0;

            let image_pixel = image.buffer.get_pixel(x, y);
            let red = (image_pixel[0] as f32 * alpha) as u8;
            let green = (image_pixel[1] as f32 * alpha) as u8;
            let blue = (image_pixel[2] as f32 * alpha) as u8;
            let original_alpha = (image_pixel[3] as f32 * alpha) as u8;
            *pixel = Rgba([red, green, blue, original_alpha]);
        }

        Layer::from_buffer(result)
    }
}
