fn main() {
    let (width, height) = get_dim();
    let mut term = Term::new(width, height);
    let mut search_bar = Search::new();
    let mut state = State::Search;

    loop {
        goto_start();
        clear();
        term.draw(matches!(state, State::Term));
        search_bar.draw(matches!(state, State::Search));
        flush();

        match state {
            State::Term => match input() {
                Key::Up => term.go(Direction::Up),
                Key::Down => term.go(Direction::Down),
                Key::Right => term.go(Direction::Right),
                Key::Left => term.go(Direction::Left),
                Key::Enter => term.click(),
                Key::Tab => {
                    state.next();
                }
                Key::Esc => break,
                Key::Size(height, width) => term.update_size(width, height),
                _ => (),
            },
            State::Search => match input() {
                Key::Esc => break,
                Key::Size(height, width) => term.update_size(width, height),

                Key::Backspace => {
                    search_bar.pop();
                }

                Key::Tab => {
                    state.next();
                }
                Key::Char(c) => {
                    search_bar.push(c);
                }
                Key::Enter => {
                    let stations = search_bar.search();
                    term.clear();
                    for s in stations {
                        let mut name = s.name.clone();
                        if name.len() > Term::CELL_SIZE {
                            name = String::from_utf8_lossy(&name.as_bytes()[..Term::CELL_SIZE])
                                .to_string();
                        }

                        match term.push(Button::new(
                            name,
                            Box::new(move || {
                                std::process::Command::new("pkill")
                                    .arg("mpv")
                                    .spawn()
                                    .unwrap()
                                    .wait()
                                    .unwrap();
                                std::process::Command::new("mpv")
                                    .arg(&s.url)
                                    .stdin(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .stdout(std::process::Stdio::null())
                                    .spawn()
                                    .unwrap();
                            }),
                        )) {
                            Ok(()) => (),
                            Err(TermErr::Full) => break,
                        }
                    }
                }
                _ => (),
            },
        }
    }
}
enum State {
    Term,
    Search,
}

impl State {
    fn next(&mut self) {
        match self {
            State::Term => {
                *self = State::Search;
            }
            State::Search => {
                *self = State::Term;
            }
        }
    }
}

struct Search {
    buffer: String,
}
impl Search {
    fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
    fn draw(&self, is_selected: bool) {
        draw_pos_no_modify((0, 100), format!("Search: {}", self.buffer), is_selected);
    }
    fn push(&mut self, c: char) {
        self.buffer.push(c);
    }
    fn pop(&mut self) {
        self.buffer.pop();
    }

    fn search(&mut self) -> Vec<Station> {
        let req = format!("http://91.132.145.114/json/stations/search?{}", self.buffer);
        let stations: Vec<Station> = ureq::get(&req).call().into_json_deserialize().unwrap();
        self.buffer.clear();
        stations
    }
}
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct Station {
    name: String,
    url: String,
}

#[derive(Debug, Default)]
struct Term {
    cells: Vec<Vec<Button>>,
    cursor: (usize, usize),
    width: usize,
    height: usize,
}
impl Drop for Term {
    fn drop(&mut self) {
        clean_term();
    }
}

impl Term {
    const CELL_SIZE: usize = 20;
    const MARGIN: usize = 5;

    fn new(width: usize, height: usize) -> Self {
        preapre_term();
        Self {
            cells: vec![vec![]],
            cursor: (0, 0),
            width: width / (Self::CELL_SIZE + Self::MARGIN),
            height: height - 1,
        }
    }
    fn update_size(&mut self, width: usize, height: usize) {
        self.width = width / (Self::CELL_SIZE + Self::MARGIN);
        self.height = height - 1;
    }
    fn clear(&mut self) {
        self.cells = vec![vec![]];
        self.cursor = (0, 0);
    }
    fn draw(&self, term_is_selected: bool) {
        let limit = std::cmp::min(self.cells.len(), self.height) - 1;
        for (row_idx, row) in self.cells.iter().enumerate().take(limit) {
            for (col_idx, cell) in row.iter().enumerate() {
                let is_selected = if self.cursor == (row_idx, col_idx) {
                    true
                } else {
                    false
                };
                cell.draw(is_selected && term_is_selected);
            }
            movetonextline();
        }
    }
    fn go(&mut self, dir: Direction) {
        let last_row_idx = std::cmp::min(self.cells.len(), self.height) - 2;
        let last_col_idx = self.width - 1;

        use Direction::*;
        match dir {
            Up => {
                if self.cursor.0 == 0 {
                    self.cursor.0 = last_row_idx;
                } else {
                    self.cursor.0 -= 1;
                }
            }
            Down => {
                if self.cursor.0 == last_row_idx {
                    self.cursor.0 = 0;
                } else {
                    self.cursor.0 += 1;
                }
            }
            Right => {
                if self.cursor.1 == last_col_idx {
                    self.cursor.1 = 0;
                    self.go(Down);
                } else {
                    self.cursor.1 += 1;
                }
            }
            Left => {
                if self.cursor.1 == 0 {
                    self.cursor.1 = last_col_idx;
                    self.go(Up);
                } else {
                    self.cursor.1 -= 1;
                }
            }
        }
    }
    fn click(&self) {
        self.cells[self.cursor.0][self.cursor.1].click();
    }
    fn push(&mut self, cell: Button) -> Result<(), TermErr> {
        if self.cells.len() == self.width * self.height {
            return Err(TermErr::Full);
        }

        if self.cells.last().unwrap().len() == self.width {
            self.cells.push(vec![]);
        }
        self.cells.last_mut().unwrap().push(cell);
        Ok(())
    }
}

enum TermErr {
    Full,
}

enum Direction {
    Up,
    Down,
    Right,
    Left,
}

struct Button {
    label: String,
    on_click: Box<dyn Fn()>,
}
impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Button, label = {}", self.label)
    }
}

impl Button {
    fn new(label: String, on_click: Box<dyn Fn()>) -> Self {
        Button { label, on_click }
    }
    fn draw(&self, is_selected: bool) {
        draw(&self.label, is_selected)
    }
    fn click(&self) {
        (self.on_click)();
    }
}

// specific implementation
fn draw<D: std::fmt::Display + Clone>(label: D, is_selected: bool) {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    crossterm::queue!(std::io::stdout(), crossterm::cursor::SavePosition).unwrap();

    if is_selected {
        crossterm::queue!(
            stdout,
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Red)
        )
        .unwrap();
        crossterm::queue!(stdout, crossterm::style::Print(label)).unwrap();
        crossterm::queue!(
            stdout,
            crossterm::style::SetBackgroundColor(crossterm::style::Color::Reset)
        )
        .unwrap();
    } else {
        crossterm::queue!(stdout, crossterm::style::Print(label)).unwrap();
    }
    crossterm::queue!(std::io::stdout(), crossterm::cursor::RestorePosition).unwrap();
    crossterm::queue!(
        std::io::stdout(),
        crossterm::cursor::MoveRight((Term::CELL_SIZE + Term::MARGIN) as u16)
    )
    .unwrap();
}

fn flush() {
    use std::io::Write;
    std::io::stdout().flush().unwrap();
}

fn preapre_term() {
    use crossterm::cursor::Hide;
    use crossterm::terminal::EnterAlternateScreen;
    use std::io::Write;

    crossterm::execute!(std::io::stdout(), Hide).unwrap();
    crossterm::execute!(std::io::stdout(), EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
}
fn clean_term() {
    use crossterm::cursor::Show;
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    use std::io::Write;
    disable_raw_mode().unwrap();
    crossterm::execute!(std::io::stdout(), Show).unwrap();
    crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).unwrap();
    std::process::Command::new("pkill")
        .arg("mpv")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

enum Key {
    Up,
    Down,
    Right,
    Left,
    Enter,
    Esc,
    Unknown,
    Tab,
    Backspace,
    Size(usize, usize),
    Char(char),
}

fn input() -> Key {
    match crossterm::event::read().unwrap() {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Up,
            ..
        }) => Key::Up,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Down,
            ..
        }) => Key::Down,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Right,
            ..
        }) => Key::Right,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Left,
            ..
        }) => Key::Left,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Enter,
            ..
        }) => Key::Enter,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Esc,
            ..
        }) => Key::Esc,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Tab,
            ..
        }) => Key::Tab,
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char(c),
            ..
        }) => Key::Char(c),
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Backspace,
            ..
        }) => Key::Backspace,
        crossterm::event::Event::Resize(col, row) => Key::Size(row as usize, col as usize),
        _ => Key::Unknown,
    }
}

fn goto_start() {
    use std::io::Write;
    crossterm::queue!(std::io::stdout(), crossterm::cursor::MoveTo(0, 0)).unwrap();
}

fn movetonextline() {
    use std::io::Write;
    crossterm::queue!(std::io::stdout(), crossterm::cursor::MoveToNextLine(1)).unwrap();
}

fn get_dim() -> (usize, usize) {
    let dim = crossterm::terminal::size().unwrap();
    (dim.0 as usize, dim.1 as usize)
}

fn clear() {
    use std::io::Write;
    crossterm::queue!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
    )
    .unwrap();
}

fn draw_pos_no_modify(pos: (usize, usize), label: String, is_selected: bool) {
    use std::io::Write;
    crossterm::queue!(std::io::stdout(), crossterm::cursor::SavePosition).unwrap();
    crossterm::queue!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(pos.0 as u16, pos.1 as u16)
    )
    .unwrap();

    draw(label, is_selected);
    crossterm::queue!(std::io::stdout(), crossterm::cursor::RestorePosition).unwrap();
}
