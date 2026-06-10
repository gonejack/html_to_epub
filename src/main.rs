use std::fs;

use clap::Parser;
use nu_ansi_term::Color;
use time::macros::format_description;
use time::UtcOffset;
use tracing::error;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::fmt::FormatEvent;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::EnvFilter;

use html_to_epub::cmd::HtmlToEpub;
use html_to_epub::cmd::HtmlToEpubOption;

struct MyFormatter;

impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
        let timer = OffsetTime::new(offset, format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
        let meta = event.metadata();

        let level = *meta.level();
        let level_str = match level {
            tracing::Level::ERROR => Color::Red.bold().paint("ERRO"),
            tracing::Level::WARN  => Color::Yellow.bold().paint("WARN"),
            tracing::Level::INFO  => Color::Green.bold().paint("INFO"),
            tracing::Level::DEBUG => Color::Blue.bold().paint("DBUG"),
            tracing::Level::TRACE => Color::Purple.bold().paint("TRAC"),
        };

        write!(writer, "[")?;
        timer.format_time(&mut writer)?;
        write!(writer, "][{}][{}] ", level_str, meta.target())?;
        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

#[derive(Parser)]
#[command(name = "html_to_epub", about = "This command line converts .html files to .epub")]
struct Args {
    /// Set epub title
    #[arg(long, default_value = "HTML")]
    title: String,

    /// Set epub author
    #[arg(long, default_value = "html_to_epub")]
    author: String,

    /// Set epub cover image path
    #[arg(long)]
    cover: Option<String>,

    /// Set output file
    #[arg(long, default_value = "output.epub")]
    output: String,

    /// Verbose printing
    #[arg(short, long)]
    verbose: bool,

    /// Show about
    #[arg(long)]
    about: bool,

    /// HTML files to convert
    files: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.about {
        println!("Visit https://github.com/gonejack/html_to_epub");
        return;
    }

    let directives = if args.verbose { "debug,hyper=info,hyper_util=info,h2=info,rustls=info,reqwest=info" } else { "info" };
    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .parse_lossy(directives);
    tracing_subscriber::fmt()
        .event_format(MyFormatter)
        .with_env_filter(filter)
        .init();

    if args.files.is_empty() {
        error!(target: "argument", "No .html files given");
        return;
    }

    let cover = match args.cover {
        Some(ref p) => fs::read(p).unwrap_or_else(|e| {
            error!(target: "argument", "Failed to read cover image '{}': {}", p, e);
            std::process::exit(1);
        }),
        None => include_bytes!("cover.png").to_vec(),
    };

    let opt = HtmlToEpubOption {
        cover: &cover,
        title: &args.title,
        author: &args.author,
        output: &args.output,
    };

    if let Err(err) = HtmlToEpub::new(&args.files, opt).run().await {
        error!("failed: {}", err);
    }
}
