use macroquad::{prelude::ImageFormat, texture::Texture2D};
use resvg::usvg::fontdb;
pub fn svg_to_png(svg_str: &str) -> Vec<u8> {
    let opt = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_str, &opt).unwrap();
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();
    let size = tree.size();
    let width = size.width().round() as u32;
    let height = size.height().round() as u32;
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(width, height).unwrap();

    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().unwrap()
}
pub fn svg_to_texture(svg_str: &str) -> Texture2D {
    let png_data = svg_to_png(&svg_str);
    Texture2D::from_file_with_format(&png_data, Some(ImageFormat::Png))
}
