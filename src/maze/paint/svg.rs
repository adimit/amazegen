use svg::Document;

use crate::maze::feature::Svg;

pub fn write_document(doc: &Document) -> Svg {
    let mut strbuf: Vec<u8> = Vec::new();
    svg::write(&mut strbuf, doc).unwrap();
    Svg(String::from_utf8(strbuf).unwrap())
}

pub(crate) fn write(strbuf: &[u8], doc: _) -> _ {
    todo!()
}
