use std::{env, fs};

use getopts::Options;
use log::{error, LevelFilter};

use html_to_epub::cmd::HtmlToEpub;
use html_to_epub::cmd::HtmlToEpubOption;

fn main() {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut opts = Options::new();
    {
        opts.optopt("", "title", "Set epub title", "title");
        opts.optopt("", "author", "Set epub author", "author");
        opts.optopt("", "cover", "Set epub cover", "cover image");
        opts.optopt("", "output", "Set output file", "output.epub");
        opts.optflag("v", "verbose", "Verbose printing");
        opts.optflag("h", "help", "Print this help");
        opts.optflag("", "about", "Show about");
    }

    let args_raw: Vec<String> = env::args().collect();
    let args = opts.parse(&args_raw[1..]).expect("parse argument failed");

    match () {
        _ if args.opt_present("about") => {
            println!("{}", "Visit https://github.com/gonejack/html_to_epub");
            return;
        }
        _ if args.opt_present("h") => {
            println!("{}", opts.usage("Usage: html_to_epub *.html"));
            return;
        }
        _  if args.free.is_empty() => {
            error!(target: "argument", "No .html files given");
            return;
        }
        _ => {}
    }

    let mut cover = include_bytes!("cover.png").to_vec();
    if let Some(cover_path) = args.opt_str("cover") {
        cover = fs::read(cover_path).unwrap();
    }
    let title = args.opt_str("title").unwrap_or("HTML".to_string());
    let author = args.opt_str("author").unwrap_or("html_to_epub".to_string());
    let output = args.opt_str("output").unwrap_or("output.epub".to_string());

    let opt = HtmlToEpubOption {
        cover: &cover,
        title: &title,
        author: &author,
        output: &output,
    };

    if let Err(err) = HtmlToEpub::new(&args.free, opt).run() {
        error!("failed: {}", err);
    }
}
