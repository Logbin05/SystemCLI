use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{collections::HashMap, error::Error, time::Duration};
use sysinfo::{Networks, System};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut sys = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();

    let mut prev_network: HashMap<String, (u64, u64)> = HashMap::new();

    loop {
        sys.refresh_all();
        networks.refresh(true);

        let cpu_usage = sys.global_cpu_usage();

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_percent = (used_mem as f64 / total_mem as f64 * 100.0) as u16;

        let mut download_speed = 0u64;
        let mut upload_speed = 0u64;

        for (name, data) in networks.iter() {
            let prev = prev_network
                .get(name)
                .copied()
                .unwrap_or((data.received(), data.transmitted()));
            let recv = data.received().saturating_sub(prev.0);
            let sent = data.transmitted().saturating_sub(prev.1);
            download_speed += recv;
            upload_speed += sent;
            prev_network.insert(name.clone(), (data.received(), data.transmitted()));
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(size);

            let cpu_gauge = Gauge::default()
                .block(Block::default().title("CPU Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(cpu_usage as u16);
            f.render_widget(cpu_gauge, chunks[0]);

            let mem_gauge = Gauge::default()
                .block(Block::default().title("Memory Usage").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .percent(mem_percent);
            f.render_widget(mem_gauge, chunks[1]);

            let download_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title("Download (KB/s)")
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(((download_speed as f64 / 1024.0).min(1000.0) / 10.0) as u16);
            f.render_widget(download_gauge, chunks[2]);

            let upload_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title("Upload (KB/s)")
                        .borders(Borders::ALL),
                )
                .gauge_style(Style::default().fg(Color::Magenta))
                .percent(((upload_speed as f64 / 1024.0).min(1000.0) / 10.0) as u16);
            f.render_widget(upload_gauge, chunks[3]);
        })?;

        if event::poll(Duration::from_millis(1000))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
