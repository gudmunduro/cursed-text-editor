use cursive::event::{Event, EventResult, Key};
use cursive::theme::{BaseColor, Color, ColorStyle, Theme};
use cursive::{Printer, Vec2, View};
use std::cmp::{max, min};

pub struct EditorView {
    scroll_index: usize,
    cursor_pos: Vec2,
    text: Vec<String>,
    constraint: Vec2,
    invalidated_data_changed: bool,
    invalidated_resize: bool,
}

impl EditorView {
    pub fn new(text: String) -> EditorView {
        let lines = text.split("\n")
                        .map(|x| String::from(x.trim()))
                        .collect();

        EditorView {
            scroll_index: 0,
            cursor_pos: Vec2::new(0, 0),
            text: lines,
            constraint: Vec2::new(0, 0),
            invalidated_data_changed: false,
            invalidated_resize: false,
        }
    }

    fn curr_line(&self) -> &String {
        &self.text[self.cursor_pos.y]
    }
}

// MARK: Input

impl EditorView {
    fn on_input(&mut self, c: char) -> EventResult {
        self.text[self.cursor_pos.y].insert(self.cursor_pos.x, c);
        self.cursor_pos.x += 1;

        self.invalidated_data_changed = true;
        EventResult::Consumed(None)
    }

    fn on_backspace(&mut self) {
        let line = self.curr_line();

        match self.cursor_pos.x {
            0 => {
                if self.text.len() == 1 {
                    return;
                }

                let line_copy = line.clone();
                self.text.remove(self.cursor_pos.y);
                self.move_cursor_up();

                if !line_copy.is_empty() {
                    self.text[self.cursor_pos.y] += &line_copy;
                }
            }
            _ => {
                self.text[self.cursor_pos.y].remove(self.cursor_pos.x - 1);
                self.cursor_pos.x -= 1;
            }
        }
        if self.cursor_pos.x == 0 {
            return;
        }
    }

    fn on_newline(&mut self) {
        let line = String::from(self.curr_line().clone());
        let (before_cursor, after_cursor) = line.split_at(self.cursor_pos.x);

        self.text[self.cursor_pos.y] = String::from(before_cursor.to_owned());
        // self.text.insert(self.cursor_pos.y + 1, after_cursor.into());
    }

    fn move_cursor_up(&mut self) {
        if self.cursor_pos.y == 0 {
            return;
        }

        self.cursor_pos.y -= 1;

        if self.cursor_pos.y < self.scroll_index {
            self.scroll_index -= 1;
        }
        if self.curr_line().len() < self.cursor_pos.x {
            self.cursor_pos.x = self.curr_line().len() - 2;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_pos.y >= self.text.len() - 1 {
            return;
        }

        self.cursor_pos.y += 1;

        if self.cursor_pos.y > self.scroll_index + self.constraint.y - 2 {
            self.scroll_index += 1;
        }
        if self.text[self.cursor_pos.y].len() < self.cursor_pos.x {
            self.cursor_pos.x = self.curr_line().len() - 2;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_pos.x >= self.curr_line().len() - 1 {
            return;
        }
        self.cursor_pos.x += 1;
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_pos.x == 0 {
            return;
        }

        self.cursor_pos.x -= 1;
        if self.cursor_pos.y > self.scroll_index + self.constraint.y {
            self.scroll_index += 1;
        }
    }

    fn on_key(&mut self, key: Key) -> EventResult {
        match key {
            Key::Backspace => {
                self.on_backspace();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            }
            Key::Enter => {
                self.on_newline();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            }
            Key::Up => {
                self.move_cursor_up();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            }
            Key::Down => {
                self.move_cursor_down();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            }
            Key::Right => {
                self.move_cursor_right();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            }
            Key::Left => {
                self.move_cursor_left();
                self.invalidated_data_changed = true;
                EventResult::Consumed(None)
            },
            Key::Home => {
                self.cursor_pos.x = 0;
                EventResult::Consumed(None)
            },
            Key::End => {
                self.cursor_pos.x = self.curr_line().len() - 1;
                EventResult::Consumed(None)
            },
            _ => EventResult::Ignored,
        }
    }
}

// MARK: Draw

impl EditorView {
    fn draw_text(&self, printer: &Printer, height: usize) {
        let line_draw_count = min(height, self.text.len());

        for i in 0..line_draw_count {
            let line = &self.text[self.scroll_index + i];
            printer.print((0, i), &format!("{} ", line));

            // Show cursor
            if self.scroll_index + i == self.cursor_pos.y {
                printer.with_color(ColorStyle::highlight(), |printer| {
                    printer.print(
                        (self.cursor_pos.x, self.cursor_pos.y - self.scroll_index),
                        &line[self.cursor_pos.x..self.cursor_pos.x + 1],
                    );
                });
            }
        }
    }

    fn draw_bottom_bar(&self, printer: &Printer, pos: usize) {
        let line_count = self.text.len();

        printer.print((0, pos), &format!("Lines: {}", line_count)[..]);
    }
}

// MARK: Implement view

impl View for EditorView {
    fn draw(&self, printer: &Printer) {
        let screen_height = printer.size.y;
        self.draw_text(printer, screen_height - 1);
        self.draw_bottom_bar(printer, screen_height - 1);
    }

    fn layout(&mut self, constraint: Vec2) {
        self.constraint = constraint;
        self.invalidated_data_changed = false;
        self.invalidated_resize = false;
    }

    fn needs_relayout(&self) -> bool {
        self.invalidated_data_changed || self.invalidated_resize
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::WindowResize => {
                self.invalidated_resize = true;
                EventResult::Consumed(None)
            },
            Event::Char(c) => self.on_input(c),
            Event::Key(k) => self.on_key(k),
            _ => EventResult::Ignored,
        }
    }
}
