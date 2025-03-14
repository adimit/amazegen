use std::collections::HashMap;

use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use svg2pdf::usvg::Options;

use crate::maze::feature::Svg;

pub struct PdfWriter {
    alloc: Ref,
    page_ids: Vec<Ref>,
    options: Options<'static>,
    pdf: Pdf,
    page_tree_id: Ref,
    font_name: String,
}

const A4_WIDTH: f32 = 595.0;
const A4_HEIGHT: f32 = 842.0;

impl PdfWriter {
    pub fn new(font: Option<Vec<u8>>, font_name: &Option<String>) -> Self {
        let mut options = svg2pdf::usvg::Options::default();
        let fontdb = options.fontdb_mut();
        fontdb.load_system_fonts();
        fontdb.faces().for_each(|face| {
            println!("Font: {:?}", face.families);
        });
        let mut alloc = Ref::new(1);
        let catalog_id = alloc.bump();
        let page_tree_id = alloc.bump();
        let mut pdf = Pdf::new();
        pdf.catalog(catalog_id).pages(page_tree_id);

        if let Some(font) = font {
            println!("Loading font");
            let font_id = alloc.bump();
            pdf.stream(font_id, &font);
            fontdb.load_font_data(font);
        }

        PdfWriter {
            alloc,
            page_ids: Vec::new(),
            options,
            pdf,
            page_tree_id,
            font_name: font_name.clone().unwrap_or("Helvetica".to_string()),
        }
    }

    pub fn append_maze(&mut self, maze: &Svg) {
        let mut slice = vec![0; 3 + (usize::BITS / 8) as usize];
        let font_name = Name(self.font_name.as_bytes());
        let svg_name = {
            slice[0..3].copy_from_slice(b"SVG");
            slice[3..].copy_from_slice(&self.page_ids.len().to_le_bytes());
            Name(&slice)
        };
        let page_id = self.alloc.bump();
        self.page_ids.push(page_id);
        let content_id = self.alloc.bump();

        let tree = svg2pdf::usvg::Tree::from_str(&maze.content, &self.options)
            .expect("Failed to parse SVG");
        let (svg_chunk, svg_id) = svg2pdf::to_chunk(&tree, svg2pdf::ConversionOptions::default())
            .expect("Failed to convert SVG to PDF chunk");

        let mut map = HashMap::new();
        let svg_chunk =
            svg_chunk.renumber(|old| *map.entry(old).or_insert_with(|| self.alloc.bump()));
        let svg_id = map
            .get(&svg_id)
            .expect("Failure while embedding SVG in PDF.");

        let mut page = self.pdf.page(page_id);
        let x_margin = 20.0;
        page.media_box(Rect::new(0.0, 0.0, A4_WIDTH, A4_HEIGHT));
        page.parent(self.page_tree_id);
        page.contents(content_id);

        let (width, height) = maze.dimensions;
        let factor = (A4_WIDTH - 2.0 * x_margin) / width as f32;
        let svg_width = width as f32 * factor;
        let svg_height = height as f32 * factor;
        let mut resources = page.resources();
        let font_id = self.alloc.bump();
        resources.x_objects().pair(svg_name, svg_id);
        resources.fonts().pair(font_name, font_id);
        resources.finish();
        page.finish();

        self.pdf
            .type1_font(font_id)
            .base_font(Name(self.font_name.as_bytes()));

        // Add our graphic.
        let mut content = Content::new();
        content
            .transform([
                svg_width,
                0.0,
                0.0,
                svg_height,
                x_margin,
                (822.0 - svg_height) / 2.0,
            ])
            .x_object(svg_name);

        self.pdf.stream(content_id, &content.finish());
        // Write the SVG chunk into the PDF page.
        self.pdf.extend(&svg_chunk);
    }

    pub fn write_to_file(self, filename: &str) {
        std::fs::write(filename, self.write_to_memory()).expect("Failed to write PDF file");
    }

    pub fn write_to_memory(mut self) -> Vec<u8> {
        self.pdf
            .pages(self.page_tree_id)
            .kids(self.page_ids.clone())
            .count(self.page_ids.len().clamp(0, i32::MAX as usize) as i32);
        self.pdf.finish()
    }
}
