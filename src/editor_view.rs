use cursive::theme::{Color, ColorStyle, Theme, BaseColor};
use cursive::event::{Event, EventResult, Key};
use cursive::{View, Printer, Vec2};
use std::cmp::{min, max};

pub struct EditorView {
    scroll_index: usize,
    cursor_pos: Vec2,
    text: Vec<String>,
    constraint: Vec2
}

impl EditorView {
    pub fn new(text: String) -> EditorView {
        let lines = text.split("\n").map(|x| String::from(x)).collect();
        EditorView { scroll_index: 0, cursor_pos: Vec2::new(0, 0), text: lines, constraint: Vec2::new(0, 0) }
    }

    fn on_input(&mut self, c: char) -> EventResult {
        EventResult::Consumed(None)
    }

    fn on_key(&mut self, key: Key) -> EventResult {
        match key {
            Key::Backspace => {
                EventResult::Consumed(None)
            },
            Key::Enter => {
                EventResult::Consumed(None)
            },
            Key::Up => {
                if self.cursor_pos.y > 0 {
                    self.cursor_pos.y -= 1;

                    if self.cursor_pos.y < self.scroll_index {
                        self.scroll_index -= 1;
                    }
                    if self.text[self.cursor_pos.y].len() < self.cursor_pos.x {
                        self.cursor_pos.x = self.text[self.cursor_pos.y].len() - 2;
                    }
                }   
                EventResult::Consumed(None)
            },
            Key::Down => {
                if self.cursor_pos.y < self.text.len() - 1 {
                    self.cursor_pos.y += 1;

                    if self.cursor_pos.y > self.scroll_index + self.constraint.y - 2 {
                        self.scroll_index += 1;
                    }
                    if self.text[self.cursor_pos.y].len() < self.cursor_pos.x {
                        self.cursor_pos.x = self.text[self.cursor_pos.y].len() - 2;
                    }
                }
                EventResult::Consumed(None)
            },
            Key::Right => {
                if self.cursor_pos.x < self.text[self.cursor_pos.y].len() - 1 { 
                    self.cursor_pos.x += 1;
                }
                EventResult::Consumed(None)
            },
            Key::Left => {
                if self.cursor_pos.x > 0 {
                    self.cursor_pos.x -= 1;
                    if self.cursor_pos.y > self.scroll_index + self.constraint.y {
                        self.scroll_index += 1;
                    }
                }
                EventResult::Consumed(None)
            },
            _ => {
                EventResult::Ignored
            }
        }
    }
}

impl EditorView {
    fn draw_text(&self, printer: &Printer, height: usize) {
        let line_draw_count = min(height, self.text.len());

        for i in 0..line_draw_count {
            let line = &self.text[self.scroll_index + i];
            printer.print((0, i), line);

            // Show cursor
            if self.scroll_index + i == self.cursor_pos.y {
                printer.with_color(ColorStyle::highlight(), |printer| {
                    printer.print((self.cursor_pos.x, self.cursor_pos.y - self.scroll_index), &line[self.cursor_pos.x..self.cursor_pos.x+1]);
                });
            }
        }
    }

    fn draw_bottom_bar(&self, printer: &Printer, pos: usize) {
        let line_count = self.text.len();

        printer.print((0, pos), &format!("Lines: {}", line_count)[..]);
    }
}

impl View for EditorView {
    fn draw(&self, printer: &Printer) {
        let screen_height = printer.size.y;
        
        self.draw_text(printer, screen_height - 1);
        self.draw_bottom_bar(printer, screen_height - 1);
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        self.constraint = constraint;
        constraint
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char(c) => self.on_input(c),
            Event::Key(k) => self.on_key(k),
            _ => EventResult::Ignored
        }
    }
}