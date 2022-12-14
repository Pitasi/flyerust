use std::error::Error;

mod gfx;

use gfx::gfx::*;
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;
use rusttype::Font;

fn main() -> Result<(), Box<dyn Error>> {
    let mut canvas = Canvas::new(2000, 2000);

    let bg = Layer::from_file("./imgs/bg.png")?;
    let gfx = Layer::from_file("./imgs/gfx.png")?;
    let logo = Layer::from_file("./imgs/logo.png")?;
    let avatar_circle = Layer::from_file("./imgs/avatar_circle.png")?;

    canvas.add_layer(bg, Position::Coord(0), Position::Coord(0));
    canvas.add_layer(gfx, Position::Coord(0), Position::Coord(0));
    canvas.add_layer(logo, Position::Center, Position::Coord(70));
    canvas.add_layer(avatar_circle, Position::Coord(767), Position::Coord(70));

    let font_medium_data: &[u8] = include_bytes!("./GothamMedium.ttf");
    let font_medium: Font<'static> = Font::try_from_bytes(font_medium_data).unwrap();
    let font_bold_data: &[u8] = include_bytes!("./GothamBold.ttf");
    let font_bold: Font<'static> = Font::try_from_bytes(font_bold_data).unwrap();
    let font_black_data: &[u8] = include_bytes!("./Gotham-Black.otf");
    let font_black: Font<'static> = Font::try_from_bytes(font_black_data).unwrap();

    let black = Rgba([0, 0, 0, 255]);
    let purple = Rgba([196, 40, 198, 255]);
    let speaker_name = TextLayer::new("ROBERTO CLAPIS", &font_bold, 65.0, black);
    canvas.add_layer(
        speaker_name.to_layer(),
        Position::Coord(200),
        Position::Coord(1050),
    );

    let speaker_title = TextLayer::new("SECURITY ENGINEER @ GOOGLE", &font_bold, 41.0, black);
    canvas.add_layer(
        speaker_title.to_layer(),
        Position::Coord(200),
        Position::Coord(1145),
    );

    let title_size = 172.0;
    let title1 = TextLayer::new("WEB", &font_black, title_size, purple);
    let title2 = TextLayer::new("SECURITY", &font_black, title_size, purple);
    canvas.add_layer(
        title1.to_layer(),
        Position::Coord(200),
        Position::Coord(1277),
    );
    canvas.add_layer(
        title2.to_layer(),
        Position::Coord(200),
        Position::Coord(1443),
    );

    let mut date_canvas = DynamicCanvas::new();
    let month = TextLayer::new("DEC", &font_medium, 42.0, black);
    date_canvas.add_layer(month.to_layer(), Position::Coord(0), Position::Coord(0));
    let date = TextLayer::new("16", &font_medium, 70.0, black);
    date_canvas.add_layer(date.to_layer(), Position::Center, Position::Coord(55));

    let hour_pos_y = 1755 + (date_canvas.height() as i32 / 4);
    let hour = TextLayer::new("18:15", &font_medium, 66.0, black);

    canvas.add_dynamic_canvas(date_canvas, Position::Coord(200), Position::Coord(1755));
    canvas.add_layer(
        hour.to_layer(),
        Position::Coord(500),
        Position::Coord(hour_pos_y),
    );

    let mut venue_canvas = DynamicCanvas::new();
    let venue_line1 = TextLayer::new("Aula G", &font_medium, 67.0, black);
    venue_canvas.add_layer(
        venue_line1.to_layer(),
        Position::Coord(0),
        Position::Coord(0),
    );
    let venue_line2 = TextLayer::new("Polo Fibonacci, Pisa", &font_medium, 41.0, black);
    venue_canvas.add_layer(
        venue_line2.to_layer(),
        Position::Center,
        Position::Coord(85),
    );
    canvas.add_dynamic_canvas(venue_canvas, Position::Coord(850), Position::Coord(1755));

    let (line_width, line_length) = (5, 145);
    let mut line_buffer = RgbaImage::new(line_width, line_length);
    for pix in line_buffer.pixels_mut() {
        *pix = black;
    }
    let line = Layer::from_buffer(line_buffer);
    canvas.add_layer(line, Position::Coord(400), Position::Coord(1740));

    let mut line_buffer2 = RgbaImage::new(line_width, line_length);
    for pix in line_buffer2.pixels_mut() {
        *pix = black;
    }
    let line = Layer::from_buffer(line_buffer2);
    canvas.add_layer(line, Position::Coord(750), Position::Coord(1745));

    let avatar_size = 1150;
    let mut avatar_canvas = Canvas::new(avatar_size, avatar_size);

    let mut avatar_image = Layer::from_file("./imgs/roberto-clapis.jpg")?;
    avatar_image.resize(2000);
    avatar_canvas.add_layer(avatar_image, Position::Coord(-560), Position::Coord(-40));

    let mut mask = RgbaImage::new(avatar_size, avatar_size);
    draw_filled_circle_mut(
        &mut mask,
        (avatar_size as i32 / 2, avatar_size as i32 / 2),
        avatar_size as i32 / 2,
        black,
    );
    avatar_canvas.set_mask(mask);

    let avatar = avatar_canvas.rasterize();

    canvas.add_layer(avatar, Position::Coord(1091), Position::Coord(276));

    let img = canvas.rasterize();
    img.save("result.png")?;

    Ok(())
}
