# html_to_epub
This command line converts .html file to .epub file.

![Build](https://github.com/gonejack/html_to_epub/actions/workflows/build.yml/badge.svg)
[![GitHub license](https://img.shields.io/github/license/gonejack/html_to_epub.svg?color=blue)](LICENSE)

### install
```shell
cargo install html_to_epub
```
### Usage
```shell
html_to_epub *.html
```
```
Options:
        --title title   Set epub title
        --author author Set epub author
        --cover cover image
                        Set epub cover
        --output output.epub
                        Set output file
    -v, --verbose       Verbose printing
    -h, --help          Print this help
        --about         Show about

```
