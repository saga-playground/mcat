use crossterm::{
    cursor::MoveTo,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseEvent, MouseEventKind,
    },
    execute, queue,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use rasteroid::image_extended::ZoomPanViewport;
use std::{
    io::{self, Stdout, Write},
    time::Duration,
};

struct InteractiveTerminalGuard {
    stdout: Stdout,
}

impl InteractiveTerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture)?;
        Ok(Self { stdout })
    }
}

impl Drop for InteractiveTerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, DisableMouseCapture);
        let _ = disable_raw_mode();
    }
}

pub fn show_help_prompt(
    out: &mut impl Write,
    term_width: u16,
    term_height: u16,
    state: &ZoomPanViewport,
    current_image: u8,
    max_images: u8,
) -> io::Result<()> {
    let current_image = current_image + 1; // 0 based inex to 1
    let help_text = "[Arrow/hjkl/Wheel] Move [n/p] Next/Pre [g/G] Start/End  [+/-] Zoom  [0] Reset  [q/ESC] Quit";
    let status_text = format!(
        "Position: ({}, {}) | Zoom: {}x | image: {current_image}/{max_images}",
        state.pan_x(),
        state.pan_y(),
        state.zoom()
    );

    // Calculate positions (bottom of screen)
    let separator_line = 2; // Lines reserved for status/help
    let status_line = term_height.saturating_sub(separator_line);
    let help_line = term_height.saturating_sub(1);

    // Center the text horizontally
    let help_pos = term_width.saturating_sub(help_text.len() as u16) / 2;
    let status_pos = term_width.saturating_sub(status_text.len() as u16) / 2;

    // Add separator line
    execute!(
        out,
        MoveTo(0, status_line.saturating_sub(1)),
        Print(format!("{:━^width$}", "", width = term_width as usize)),
    )?;

    // Write status text
    execute!(out, MoveTo(status_pos, status_line), Print(&status_text),)?;

    // Write help text
    execute!(out, MoveTo(help_pos, help_line), Print(&help_text),)?;

    Ok(())
}

pub fn clear_screen(
    stdout: &mut impl std::io::Write,
    addon: Option<Vec<u8>>,
) -> std::io::Result<()> {
    let mut buffer: Vec<u8> = Vec::new();
    queue!(buffer, Clear(ClearType::All), MoveTo(0, 0))?;
    if let Some(val) = addon {
        buffer.extend_from_slice(&val);
    }

    stdout.write_all(&buffer)?;
    Ok(())
}

pub fn run_interactive_viewer(
    container_width: u32,
    container_height: u32,
    image_width: u32,
    image_height: u32,
    max_images: u8,
    mut callback: impl FnMut(&mut ZoomPanViewport, u8) -> Option<()>,
) -> std::io::Result<()> {
    let _guard = InteractiveTerminalGuard::enter()?;

    let mut viewport =
        ZoomPanViewport::new(container_width, container_height, image_width, image_height);
    let mut current_image = 0;

    // Initial callback
    let mut should_quit = callback(&mut viewport, current_image);
    let mut last_callback_time = std::time::Instant::now();
    let callback_throttle = std::time::Duration::from_millis(50);

    while should_quit.is_some() {
        if event::poll(Duration::from_millis(16))? {
            // ~60fps
            let mut viewport_changed = false;
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    match key {
                        // Quit (q or ESC)
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Esc, ..
                        } => break,

                        // next image
                        KeyEvent {
                            code: KeyCode::Char('n'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if current_image + 1 < max_images {
                                viewport_changed = true;
                                current_image += 1;
                            }
                        }

                        // previous image
                        KeyEvent {
                            code: KeyCode::Char('p'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if current_image != 0 {
                                viewport_changed = true;
                                current_image -= 1;
                            }
                        }

                        //left
                        KeyEvent {
                            code: KeyCode::Left,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('h'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if viewport.adjust_pan(-50, 0) {
                                viewport_changed = true;
                            }
                        }

                        // right
                        KeyEvent {
                            code: KeyCode::Right,
                            modifiers: KeyModifiers::NONE,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('l'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if viewport.adjust_pan(50, 0) {
                                viewport_changed = true;
                            }
                        }

                        // up
                        KeyEvent {
                            code: KeyCode::Up,
                            modifiers: KeyModifiers::NONE,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('k'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if viewport.adjust_pan(0, -50) {
                                viewport_changed = true;
                            }
                        }

                        // down
                        KeyEvent {
                            code: KeyCode::Down,
                            modifiers: KeyModifiers::NONE,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('j'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if viewport.adjust_pan(0, 50) {
                                viewport_changed = true;
                            }
                        }

                        // stronger up
                        KeyEvent {
                            code: KeyCode::Char('u'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        } => {
                            if viewport.adjust_pan(0, -200) {
                                viewport_changed = true;
                            }
                        }

                        // stronger down
                        KeyEvent {
                            code: KeyCode::Char('d'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        } => {
                            if viewport.adjust_pan(0, 200) {
                                viewport_changed = true;
                            }
                        }

                        // Zoom (+, - or =)
                        KeyEvent {
                            code: KeyCode::Char('+'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('='),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            viewport_changed = true;
                            viewport.set_zoom(viewport.zoom() + 1);
                        }
                        KeyEvent {
                            code: KeyCode::Char('-'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            if viewport.zoom() > 1 {
                                viewport_changed = true;
                                viewport.set_zoom(viewport.zoom() - 1);
                            }
                        }

                        KeyEvent {
                            code: KeyCode::Char('g'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            let (_, _, y, _) = viewport.get_pan_limits();
                            if viewport.pan_y() != y {
                                viewport_changed = true;
                                viewport.set_pan(viewport.pan_x(), y);
                            }
                        }
                        KeyEvent {
                            code: KeyCode::Char('G'),
                            modifiers: KeyModifiers::SHIFT,
                            ..
                        } => {
                            let (_, _, _, y) = viewport.get_pan_limits();
                            if viewport.pan_y() != y {
                                viewport_changed = true;
                                viewport.set_pan(viewport.pan_x(), y);
                            }
                        }

                        // Reset (0)
                        KeyEvent {
                            code: KeyCode::Char('0'),
                            modifiers: KeyModifiers::NONE,
                            ..
                        } => {
                            viewport_changed = true;
                            viewport.set_zoom(1);
                            viewport.set_pan(0, 0);
                        }

                        _ => {}
                    }
                }
                Event::Mouse(MouseEvent { kind, .. }) => match kind {
                    MouseEventKind::ScrollUp => {
                        if viewport.adjust_pan(0, -50) {
                            viewport_changed = true;
                        }
                    }
                    MouseEventKind::ScrollDown => {
                        if viewport.adjust_pan(0, 50) {
                            viewport_changed = true;
                        }
                    }
                    MouseEventKind::ScrollLeft => {
                        if viewport.adjust_pan(-50, 0) {
                            viewport_changed = true;
                        }
                    }
                    MouseEventKind::ScrollRight => {
                        if viewport.adjust_pan(50, 0) {
                            viewport_changed = true;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            // Callback after each viewport change, but throttled.
            if viewport_changed {
                let now = std::time::Instant::now();
                if now.duration_since(last_callback_time) >= callback_throttle {
                    should_quit = callback(&mut viewport, current_image);
                    last_callback_time = now;
                }
            }
        }
    }

    Ok(())
}
