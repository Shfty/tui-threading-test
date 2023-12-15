use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io::stdout,
    sync::{mpsc, Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use tui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
};

enum Event<I> {
    Input(I),
    Tick,
}

use tui::{backend::CrosstermBackend, Terminal};
fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || loop {
        if event::poll(Duration::default()).unwrap() {
            if let CEvent::Key(key) = event::read().unwrap() {
                tx.send(Event::Input(key)).unwrap();
            }
        }
    });

    terminal.clear()?;

    let terminal = Arc::new(RwLock::new(terminal));
    let thread_terminal = terminal.clone();

    std::thread::spawn(move || loop {
        thread_terminal
            .write()
            .unwrap()
            .draw(|f| {
                let chart = Chart::new(vec![Dataset::default().data(&[
                    (0.0, 5.0),
                    (1.0, 6.0),
                    (1.5, 6.434),
                ])])
                .block(
                    Block::default()
                        .title(Span::styled(
                            "Chart 1",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("X Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec!["foo".into(), "bar".into(), "baz".into()])
                        .bounds([64.0, 32.0]),
                )
                .y_axis(
                    Axis::default()
                        .title("Y Axis")
                        .style(Style::default().fg(Color::Gray))
                        .labels(vec![
                            Span::styled("-20", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw("0"),
                            Span::styled("20", Style::default().add_modifier(Modifier::BOLD)),
                        ])
                        .bounds([-20.0, 20.0]),
                );
                f.render_widget(chart, f.size());
            })
            .unwrap();

        std::thread::sleep(Duration::from_millis(16));
    });

    loop {
        match rx.recv().unwrap() {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    break;
                }
                _ => {}
            },
            Event::Tick => (),
        }
        std::thread::sleep(Duration::from_millis(16));
    }

    let mut terminal = terminal.write().unwrap();
    disable_raw_mode().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();

    Ok(())
}
