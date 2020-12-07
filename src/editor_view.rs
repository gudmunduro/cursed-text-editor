use cursive::event::{Event, EventResult, Key};
use cursive::theme::{BaseColor, Color, ColorStyle, Theme};
use cursive::{Printer, Vec2, View, Cursive};
use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use std::cmp::{max, min};
use std::fs;
use std::process::exit;

const TAB_SIZE: usize = 4;

pub struct EditorView {
    scroll_index: usize,
    cursor_pos: Vec2,
    file_path: String,
    text: Vec<String>,
    constraint: Vec2,
    invalidated_data_changed: bool,
    invalidated_resize: bool,
    is_insert_mode_active: bool,
    last_input: Option<char>,
    bottom_message: Option<String>,
    curr_line_graphemes: Vec<String>
}

impl EditorView {
    pub fn new(file_path: String, text: String) -> EditorView {
        let mut lines: Vec<String> = text.split("\n")
                        .map(|l| {
                            let mut line = String::from(l);
                            if line.ends_with("\n") {
                                line.pop();
                            }
                            line
                        })
                        .collect();
        
        if lines.len() == 0 {
            lines.push(String::new());
        }

        let first_line_graphemes = lines.first()
                                        .unwrap()
                                        .graphemes(true)
                                        .map(|x| String::from(x))
                                        .collect();

        EditorView {
            scroll_index: 0,
            cursor_pos: Vec2::new(0, 0),
            file_path: file_path,
            text: lines,
            constraint: Vec2::new(0, 0),
            invalidated_data_changed: false,
            invalidated_resize: false,
            is_insert_mode_active: false,
            last_input: None,
            bottom_message: None,
            curr_line_graphemes: first_line_graphemes,
        }
    }
}

// MARK: Misc functions

impl EditorView {
    fn save(&self) -> bool {
        let content = self.text.join("\n");
        
        if let Ok(_) = fs::write(&self.file_path, content) {
            true
        } else {
            false
        }
    }

    fn clear_bottom_message(&mut self) {
        self.bottom_message = Some(String::new());
    }

    fn real_line_char_index(&self, pos: usize) -> usize {
        self.curr_line_graphemes.iter()
                                .take(pos)
                                .fold(0, |acc, x| acc + x.len())
    }

    fn line_char_size(&self, pos: usize) -> usize {
        self.curr_line_graphemes[pos].len()
    }

    fn next_line_char_index(&self, pos: usize) -> usize {
        return self.real_line_char_index(pos) + self.line_char_size(pos);
    }

    fn update_curr_line_graphemes(&mut self) {
        self.curr_line_graphemes = self.curr_line().graphemes(true)
                                                    .map(|x| String::from(x))
                                                    .collect();
    }

    fn curr_line(&self) -> &String {
        &self.text[self.cursor_pos.y.clone()]
    }
}

// MARK: Events

impl EditorView {
    fn on_input_insert_mode(&mut self, c: char) -> EventResult {
        let insert_index = if self.curr_line().len() != 0 {
            self.real_line_char_index(self.cursor_pos.x)
        } else {
            0
        };

        self.text[self.cursor_pos.y].insert(insert_index, c);
        self.cursor_pos.x += 1;

        self.invalidated_data_changed = true;
        EventResult::Consumed(None)
    }

    fn on_input_normal_mode(&mut self, c: char) -> EventResult {
        match c {
            'i' => {
                self.is_insert_mode_active = true;
                EventResult::Consumed(None)
            }
            'd' if self.last_input == Some('d') && self.text.len() == 1 => {
                self.text[0] = "".into();
                self.last_input = None;
                
                EventResult::Consumed(None)
            }
            'd' if self.last_input == Some('d') => {
                self.text.remove(self.cursor_pos.y);
                self.last_input = None;

                if self.text.len() <= self.cursor_pos.y {
                    self.cursor_pos.y -= 1;
                }
                
                EventResult::Consumed(None)
            }
            's' => {
                if self.save() {
                    self.bottom_message = Some("File saved".into());
                } else {
                    self.bottom_message = Some("Failed to save file".into());
                }

                EventResult::Consumed(None)
            },
            _ => {
                self.last_input = Some('d');
                EventResult::Ignored
            }
        }
    }

    fn on_backspace(&mut self) {
        let line = self.curr_line();

        match self.cursor_pos.x {
            0 => {
                if self.text.len() == 1 || self.cursor_pos.y == 0 {
                    return;
                }

                let line_copy = line.clone();
                self.text.remove(self.cursor_pos.y);

                self.cursor_pos.y -= 1;

                if !line_copy.is_empty() {
                    self.text[self.cursor_pos.y] += &line_copy;
                }

                self.update_curr_line_graphemes();
                self.cursor_pos.x = self.real_line_char_index(self.curr_line().len() - line_copy.len());
            }
            _ => {
                for _ in 0..self.line_char_size(self.cursor_pos.x - 1) {
                    let index = self.next_line_char_index(self.cursor_pos.x - 1);
                    self.text[self.cursor_pos.y].remove(index);
                }
                self.cursor_pos.x -= 1;
            }
        }

    }

    fn on_newline(&mut self) {
        let line = String::from(self.curr_line().clone());
        let (before_cursor, after_cursor) = line.split_at(self.cursor_pos.x);

        self.text[self.cursor_pos.y] = before_cursor.into();
        self.text.insert(self.cursor_pos.y + 1, after_cursor.into());

        self.cursor_pos.x = 0;
        self.cursor_pos.y += 1;
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
            self.cursor_pos.x = self.curr_line().len();
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
            self.cursor_pos.x = self.curr_line().len();
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

    fn on_key_normal_mode(&mut self, key: Key) -> EventResult {
        match key {
            _ => self.on_key_shared(key)
        }
    }

    fn on_key_insert_mode(&mut self, key: Key) -> EventResult {
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
            Key::Tab => {
                self.text[self.cursor_pos.y].insert_str(self.cursor_pos.x, &" ".repeat(TAB_SIZE));
                self.cursor_pos.x += TAB_SIZE;
                EventResult::Consumed(None)
            }
            Key::Esc => {
                self.is_insert_mode_active = false;
                EventResult::Consumed(None)
            }
            _ => self.on_key_shared(key)
        }
    }

    fn on_key_shared(&mut self, key: Key) -> EventResult {
        match key {
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
            }
            Key::Home => {
                self.cursor_pos.x = 0;
                EventResult::Consumed(None)
            }
            Key::End => {
                self.cursor_pos.x = self.curr_line().len();
                EventResult::Consumed(None)
            }
            _ => EventResult::Ignored
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
                    if self.cursor_pos.x == line.len() {
                        printer.print(
                            (self.cursor_pos.x, self.cursor_pos.y - self.scroll_index),
                            " ",
                        );
                    }
                    else {
                        let selected_char_index = self.real_line_char_index(self.cursor_pos.x);
                        let selected_char_size = self.line_char_size(self.cursor_pos.x);

                        printer.print(
                            (self.cursor_pos.x, self.cursor_pos.y - self.scroll_index),
                            &line[selected_char_index..selected_char_index + selected_char_size],
                        );
                    }


                });
            }
        }
    }

    fn draw_bottom_bar(&self, printer: &Printer, pos: usize) {
        let line_count = self.text.len();
        let insert_mode_text = if self.is_insert_mode_active {
            ", INSERT MODE"
        } else {
            ""
        };
        let bottom_message = if let Some(msg) = &self.bottom_message {
            String::from(", ") + msg
        } else {
            String::new()
        };

        printer.print((0, pos), &format!("Lines: {}{}{}", line_count, insert_mode_text, &bottom_message)[..]);
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
            Event::Char(c) => {
                self.clear_bottom_message();

                let event_res = if self.is_insert_mode_active {
                    self.on_input_insert_mode(c)
                } else {
                    self.on_input_normal_mode(c)
                };

                if let EventResult::Consumed(callback) = event_res {
                    self.update_curr_line_graphemes();
                    EventResult::Consumed(callback)
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(k) => {
                self.clear_bottom_message();

                let event_res = if self.is_insert_mode_active {
                    self.on_key_insert_mode(k)
                } else {
                    self.on_key_normal_mode(k)
                };

                if let EventResult::Consumed(callback) = event_res {
                    self.update_curr_line_graphemes();
                    EventResult::Consumed(callback)
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }
}
