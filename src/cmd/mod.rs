use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;
use std::{env, fs};

use epub_builder::ZipLibrary;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType};
use reqwest::Response;
use tracing::info;
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
    epub: Option<EpubBuilder<ZipLibrary>>,
}

impl<'a> HtmlToEpub<'a> {
    pub fn new(html: &'a Vec<String>, option: HtmlToEpubOption<'a>) -> Self {
        Self {
            html,
            option,
            epub: Some(EpubBuilder::new(ZipLibrary::new().unwrap()).unwrap()),
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error + 'static>> {
        self.make_book()?;

        for (i, html) in self.html.iter().enumerate() {
            info!("process {}", html);
            self.add_html(&format!("section{}.xhtml", i), html).await?;
        }

        let mut output = File::create(self.option.output)?;
        self.epub.take().unwrap().generate(&mut output)?;

        Ok(())
    }

    fn make_book(&mut self) -> epub_builder::Result<()> {
        let epub = self.epub.as_mut().unwrap();
        epub.metadata("author", self.option.author)?;
        epub.metadata("title", self.option.title)?;
        epub.add_cover_image("cover.png", self.option.cover, "image/png")?;
        Ok(())
    }

    async fn add_html(&mut self, name: &str, html: &str) -> Result<(), Box<dyn Error + 'static>> {
        let data = fs::read_to_string(html)?;
        let doc = Vis::load(&data).unwrap();

        self.save_images(&doc).await;

        let title_node = doc.find("title");
        let title = title_node.text();
        let body = Self::gen_xhtml(doc);
        let content = EpubContent::new(name, body.as_bytes())
            .title(title)
            .reftype(ReferenceType::Text);

        self.epub.as_mut().unwrap().add_content(content)?;

        Ok(())
    }

    fn gen_xhtml(doc: Elements) -> String {
        doc.find("html").set_attr("xmlns", Option::from("http://www.w3.org/1999/xhtml"));

        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">
"#.to_string() + &doc.outer_html()
    }

    async fn save_images(&mut self, doc: &Elements<'_>) {
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

        let tasks: Vec<_> = dls.into_iter().map(|(url, save)| {
            info!("saving {} as {}", url, save);
            tokio::spawn(download(url, save))
        }).collect();
        for task in tasks {
            let _ = task.await;
        }

        doc.find("img").each(|_i, e| {
            if let Some(src) = e.get_attribute("src") {
                let path = src.to_string();
                self.epub.as_mut().unwrap()
                    .add_resource(&path, fs::File::open(&path).unwrap(), "image/jpeg")
                    .unwrap();
            }
            true
        });
    }
}

async fn download(url: String, target: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Ok(mut fd) = File::create(target) {
        let resp = do_get(&url).await?;
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

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:93.0) Gecko/20100101 Firefox/93.0";
