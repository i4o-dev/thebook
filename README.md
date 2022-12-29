# TheBook

<!--- ![test workflow](https://github.com/0xhiro/thebook/actions/workflows/test.yml/badge.svg) -->
[![Discord](https://img.shields.io/discord/1018936651612967043)](https://discord.gg/yMEKS2hk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Crates.io](https://img.shields.io/crates/d/thebook)
![Crates.io](https://img.shields.io/crates/v/thebook)
![GitHub top language](https://img.shields.io/github/languages/top/0xhiro/thebook)

TheBook is a command line utility that allows you to SEARCH and READ [The Rust Programming Language](https://doc.rust-lang.org/book/) (popularly known as 'The Book' ) from the terminal.
Ever wanted to quickly look up how to spawn threads, or how to declare Structs in Rust? TheBook allows you to do so by simply typing `thebook spawn threads` or `thebook structs`.
TheBook renders markdown in the terminal and provides a browser-like experience. It is geared towards Rust beginners who are not quite familiar with the Rust syntax, and Rust experts who want the luxury of typing a few commands in the terminal to look up a certain Rust concept.

If you still prefer the graphical experience of a real web browser, you can use TheBook as a simple 'The Book' launcher. Just run `thebook` in the terminal without any arguments, and 'The Book' will automatically open in your web browser.

TheBook borrows the IMPOSTER_DETECTION_ALGORITHM from [AmongRust](https://github.com/0xhiro/amongrust) for advanced search processing and intelligent query parsing hehe! ඞ :). 

Note: This crate is still new, the results may not be perfect, and the code is a little messy, but hey! it gets the job done :) Please give this project a star on [Github](https://github.com/0xhiro/thebook) and subscribe to my new [YouTube channel](https://www.youtube.com/channel/UCv3SId-GfOT7MCaHVl88SQQ) for fun programming videos.

# USAGE

`cargo install thebook`

That's it. you're done. Now run `thebook what you want to search` to search for something or just `thebook` to open 'The Book' in your web browser.

![screenshot](https://github.com/0xhiro/thebook/blob/main/img/Screenshot.png?raw=true)

# COMMAND FLAGS

| Syntax      | Description |
| ----------- | ----------- |
| thebook <search query>      | Searches the book for pages that mention <search query and renders markdown in the terminal>       |
| thebook   | Opens the book in the default browser        |


Made with ❤️  by [Hiro](https://twitter.com/0x1hiro) 