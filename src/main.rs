mod editor_view;

use std::path::Path;
use std::process::exit;
use std::fs;
use std::env;
use cursive::views::{Dialog, TextView};
use cursive::theme::{Color, ColorStyle, Theme, BaseColor};
use crate::editor_view::EditorView;


fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Filename not specified");
        exit(1);
    }

    let file_path = &args[1];
    if !Path::new(&file_path).exists() {
        println!("File {} does not exist", file_path);
        exit(1);
    }
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Failed to read file {}", file_path);
            exit(1);
        }
    };
    
    show_editor(content)
}

fn show_editor(file_content: String) {
    let mut siv = cursive::default();

    siv.add_fullscreen_layer(EditorView::new(String::from(file_content)));
    
    let mut theme = Theme::default();
    theme.palette.set_color("primary", Color::TerminalDefault);
    theme.palette.set_color("secondary", Color::Dark(BaseColor::Blue));
    theme.palette.set_color("background", Color::TerminalDefault);
    theme.palette.set_color("view", Color::TerminalDefault);
    theme.palette.set_color("highlight", Color::Dark(BaseColor::White));
    theme.shadow = false;


    siv.set_theme(theme);

    siv.run();
}