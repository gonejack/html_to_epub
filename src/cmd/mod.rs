use std::{env, fs};
use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType};
use epub_builder::ZipLibrary;
use futures::executor::block_on;
use futures::future;
use log::info;
use reqwest::Response;
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
        Ok(())
    }

    fn add_html(&mut self, name: &str, html: &str) -> Result<(), Box<dyn Error + 'static>> {
        let data = fs::read_to_string(html)?;

        let doc = Vis::load(&data).unwrap();

        self.save_images(&doc);

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

    fn save_images(&mut self, doc: &Elements) {
        let mut dls: Vec<(String, String)> = Vec::new();

        doc.find("img").each(|i, e| {
            if let Some(src) = e.get_attribute("src") {
                fs::create_dir_all("image").unwrap();
                let mut save = format!("image/{}", i);
                if let Some(ext) = Path::new(&src.to_string()).extension() {
                    save = format!("{}.{}", save, ext.to_str().unwrap());
                }
                e.set_attribute("src", Option::Some(&save));
                dls.push((src.to_string(), save));
            }
            true
        });

        self.download_urls(dls);

        doc.find("img").each(|_i, e| {
            if let Some(src) = e.get_attribute("src") {
                let path = src.to_string();
                self.epub.add_resource(&path, fs::File::open(&path).unwrap(), "image/jpeg").unwrap();
            }
            true
        });
    }

    fn download_urls(&self, mut urls: Vec<(String, String)>) {
        while !urls.is_empty() {
            let mut list = Vec::new();
            for _ in 0..3 {
                if urls.is_empty() {
                    break;
                }
                let (url, save) = urls.remove(0);
                info!("saving {} as {}", url, save);
                list.push(Self::download(url, save));
            }
            block_on(future::join_all(list));
        }
    }

    async fn download(url: String, target: String) -> Result<(), Box<dyn Error>> {
        if let Ok(mut fd) = File::create(target) {
            let resp = Self::do_get(&url).await?;
            let bytes = resp.bytes().await?;
            io::copy(&mut bytes.as_ref(), &mut fd)?;
        }
        Ok(())
    }

    async fn do_get(url: &str) -> Result<Response, reqwest::Error> {
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(120));
        if let Ok(http_proxy) = env::var("http_proxy") {
            builder = builder.proxy(reqwest::Proxy::all(http_proxy)?);
        }
        builder.build()?.get(url)
            .header("user-agent", USER_AGENT)
            .timeout(Duration::new(120, 0))
            .send().await
    }
}

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:93.0) Gecko/20100101 Firefox/93.0";
