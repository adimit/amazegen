use amazegen::maze::{
    feature::{Algorithm, Shape},
    paint::RenderedMaze,
};
use qrcode::QrCode;
use svg::{Node, Parser};

pub struct Metadata {
    algorithm: Algorithm,
    shape: Shape,
    seed: u64,
    maze_url: Option<String>,
}

const SCALE: f64 = 0.2;

impl Metadata {
    pub fn new(algorithm: Algorithm, shape: Shape, seed: u64, maze_url: Option<String>) -> Self {
        Self {
            algorithm,
            shape,
            seed,
            maze_url,
        }
    }

    fn make_text_node(
        &self,
        y: u32,
        family: &Option<String>,
    ) -> (::svg::node::element::Group, u32) {
        let font_size = y / 50;
        let line_spacing = font_size / 3;
        let mut group = svg::node::element::Group::new();
        let font_family = family.as_deref().unwrap_or("sans-serif");
        for (i, text) in [
            format!("Shape: {}", shape_to_str(&self.shape),),
            format!("Algorithm: {:?}", self.algorithm,),
            format!("Seed: {}", self.seed),
        ]
        .iter()
        .enumerate()
        {
            let text_node = svg::node::element::Text::new(text)
                .set("x", 0)
                .set(
                    "y",
                    y + (font_size * 2) + (line_spacing * i as u32 + font_size * i as u32),
                )
                .set("font-size", font_size)
                .set("font-family", font_family);

            group.append(text_node);
        }
        (group, font_size * 4 + line_spacing * 3)
    }

    // returns the y offset that the text will need in the document viewport
    pub fn append_to_svg_document(
        &self,
        doc: &mut svg::Document,
        (x, y): (u32, u32),
        font_family: &Option<String>,
    ) -> f64 {
        let (text_node, text_height) = self.make_text_node(y, font_family);
        doc.append(text_node);

        if let Some(url) = &self.maze_url {
            doc.append(qr_svg_from_url(url, (x, y)));
            y as f64 * SCALE
        } else {
            text_height as f64
        }
    }

    pub fn from_configuration(
        config: &amazegen::maze::feature::Configuration,
        url: Option<String>,
    ) -> Self {
        Self::new(
            config.algorithm.clone(),
            config.shape.clone(),
            config.seed,
            url,
        )
    }

    pub fn metadata_to_render(
        &self,
        mut render: RenderedMaze,
        font_family: &Option<String>,
    ) -> RenderedMaze {
        let offset =
            self.append_to_svg_document(&mut render.document, render.dimensions, font_family);
        let (x, y) = render.dimensions;
        let svg = render.document.set(
            "viewBox",
            format!("0 0 {} {}", x, y + offset.floor() as u32),
        );

        RenderedMaze {
            document: svg,
            dimensions: (x, y + offset.floor() as u32),
        }
    }
}

fn shape_to_str(shape: &Shape) -> String {
    match shape {
        Shape::Rectilinear(x, y) => format!("Rectilinear {}×{}", x, y),
        Shape::Theta(size) => format!("Theta {}", size),
        Shape::Sigma(size) => format!("Sigma {}", size),
    }
    .to_string()
}

fn qr_data(qr_tree: Parser) -> (u32, u32, String) {
    let mut height: u32 = 328;
    let mut width: u32 = 328;
    let mut d: String = String::new();
    for event in qr_tree {
        use svg::node::element::tag;
        use svg::parser::Event;
        match event {
            Event::Tag(tag::Path, _, attributes) => {
                d = attributes
                    .get("d")
                    .expect("Failed to get QR code path")
                    .to_string();
            }
            Event::Tag(tag::Rectangle, _, attributes) => {
                if let Some(h) = attributes.get("height") {
                    height = h.parse().expect("Failed to parse QR code height")
                };
                if let Some(w) = attributes.get("width") {
                    width = w.parse().expect("Failed to parse QR code width")
                };
            }
            _ => {}
        }
    }
    (height, width, d)
}

fn get_qr_path(qr_tree: Parser, (x, y): (u32, u32)) -> ::svg::node::element::Group {
    let (qr_height, qr_width, qr_path) = qr_data(qr_tree);
    let qr_node = svg::node::element::Path::new()
        .set("d", qr_path)
        .set("shape-rendering", "crispEdges")
        .set("stroke", "black");
    let mut qr_group = svg::node::element::Group::new().set(
        "transform",
        format!(
            // order matters
            "translate({},{}), scale({},{})",
            x as f64 * 0.8,
            y,
            x as f64 / qr_width as f64 * SCALE,
            y as f64 / qr_height as f64 * SCALE,
        ),
    );
    qr_group.append(qr_node);
    qr_group
}

fn qr_svg_from_url(url: &str, (x, y): (u32, u32)) -> ::svg::node::element::Group {
    let qrcode = QrCode::new(url.as_bytes()).expect("Failed to create QR code");
    let qr_svg = qrcode.render::<qrcode::render::svg::Color>().build();
    let qr_tree = ::svg::read(&qr_svg).expect("Failed to parse QR code SVG");
    get_qr_path(qr_tree, (x, y))
}
