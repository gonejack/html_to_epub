use std::fs;
use std::fs::File;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType};
use epub_builder::ZipLibrary;
use log::info;
use visdom::types::Elements;
use visdom::Vis;

pub struct HtmlToEpubOption<'a> {
    pub cover: &'a [u8],
    pub title: &'a str,
    pub author: &'a str,
    pub output: &'a str,
}

pub struct HtmlToEpub<'a> {
    html: &'a Vec<String>,
    option: HtmlToEpubOption<'a>,
    epub: EpubBuilder<ZipLibrary>,
}

impl<'a> HtmlToEpub<'a> {
    pub fn new(html: &'a Vec<String>, option: HtmlToEpubOption<'a>) -> Self {
        Self {
            html,
            option,
            epub: EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap(),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error + 'static>> {
        self.make_book()?;

        for (i, html) in self.html.iter().enumerate() {
            info!("process {}", html);

            self.add_html(&format!("section{}.xhtml", i), html)?;
        }

        let mut output = File::create(self.option.output)?;

        self.epub.generate(&mut output)?;

        Ok(())
    }

    fn make_book(&mut self) -> epub_builder::Result<()> {
        self.epub.metadata("author", self.option.author)?;
        self.epub.metadata("title", self.option.title)?;
        self.epub.add_cover_image("cover.png", self.option.cover, "image/png")?;
        self.epub.add_content(EpubContent::new("cover.xhtml", "".as_bytes())
            .title("Cover")
            .reftype(ReferenceType::Cover))?;

        Ok(())
    }

    fn add_html(&mut self, name: &str, html: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let data = fs::read_to_string(html)?;

        let doc = Vis::load(&data)?;
        let title_node = doc.find("title");
        let title = title_node.text();
        let body = Self::gen_xhtml(doc);
        let content = EpubContent::new(name, body.as_bytes())
            .title(title)
            .reftype(ReferenceType::Text);

        self.epub.add_content(content)?;

        Ok(())
    }

    fn gen_xhtml(doc: Elements) -> String {
        doc.find("html").set_attr("xmlns", Option::from("http://www.w3.org/1999/xhtml"));

        let xhtml = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
"#.to_string() + &doc.outer_html();

        return xhtml.to_owned();
    }
}
