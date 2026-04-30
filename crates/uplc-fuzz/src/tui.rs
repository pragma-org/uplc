use std::{
    io::{self, Stdout},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::{divergence::DivergenceKind, stats::Stats};

const SPARKLINE_LEN: usize = 120;

pub struct TuiState {
    /// Rolling throughput samples for sparkline (programs/sec per tick).
    throughput_history: Vec<u64>,
    /// Last iteration count for computing delta.
    last_iterations: u64,
    /// Last sample time.
    last_sample: Instant,
    /// Recent divergence messages (ring buffer).
    recent_divergences: Vec<String>,
    /// Divergence kind breakdown.
    pub result_mismatches: u64,
    pub budget_mismatches: u64,
    pub result_and_budget: u64,
    pub panics_count: u64,
    pub external_mismatches: u64,
    /// Config display.
    pub workers: usize,
    pub plutus_version: String,
    pub output_dir: String,
    pub base_seed: u64,
    /// Peak throughput.
    peak_throughput: u64,
}

impl TuiState {
    pub fn new(workers: usize, plutus_version: &str, output_dir: &str, base_seed: u64) -> Self {
        Self {
            throughput_history: vec![0; SPARKLINE_LEN],
            last_iterations: 0,
            last_sample: Instant::now(),
            recent_divergences: Vec::new(),
            result_mismatches: 0,
            budget_mismatches: 0,
            result_and_budget: 0,
            panics_count: 0,
            external_mismatches: 0,
            workers,
            plutus_version: plutus_version.to_string(),
            output_dir: output_dir.to_string(),
            base_seed,
            peak_throughput: 0,
        }
    }

    pub fn record_divergence_kind(&mut self, kind: &DivergenceKind) {
        match kind {
            DivergenceKind::ResultMismatch => self.result_mismatches += 1,
            DivergenceKind::BudgetMismatch => self.budget_mismatches += 1,
            DivergenceKind::ResultAndBudgetMismatch => self.result_and_budget += 1,
            DivergenceKind::Panic(_) => self.panics_count += 1,
            DivergenceKind::ExternalMismatch { .. } => self.external_mismatches += 1,
        }
    }

    pub fn push_divergence_message(&mut self, msg: String) {
        self.recent_divergences.push(msg);
        if self.recent_divergences.len() > 100 {
            self.recent_divergences.remove(0);
        }
    }

    /// Sample current stats to update throughput sparkline.
    pub fn sample_throughput(&mut self, stats: &Stats) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_sample).as_secs_f64();
        if dt < 0.001 {
            return;
        }
        let current = stats.iterations.load(Ordering::Relaxed);
        let delta = current.saturating_sub(self.last_iterations);
        let rate = (delta as f64 / dt) as u64;
        self.throughput_history.push(rate);
        if self.throughput_history.len() > SPARKLINE_LEN {
            self.throughput_history.remove(0);
        }
        if rate > self.peak_throughput {
            self.peak_throughput = rate;
        }
        self.last_iterations = current;
        self.last_sample = now;
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    /// Returns true if user pressed q or Ctrl-C.
    pub fn poll_quit(&self) -> bool {
        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                return key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
            }
        }
        false
    }

    pub fn draw(
        &mut self,
        stats: &Stats,
        state: &TuiState,
        catalog_count: usize,
    ) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Main layout: header, sparkline, stats row, divergence breakdown, log
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // header
                    Constraint::Length(6), // sparkline
                    Constraint::Length(5), // big numbers
                    Constraint::Length(9), // divergence breakdown
                    Constraint::Min(4),    // recent log
                ])
                .split(area);

            draw_header(frame, chunks[0], state);
            draw_sparkline(frame, chunks[1], state);
            draw_big_numbers(frame, chunks[2], stats, state, catalog_count);
            draw_breakdown(frame, chunks[3], state, catalog_count);
            draw_log(frame, chunks[4], state);
        })?;
        Ok(())
    }
}

fn draw_header(frame: &mut Frame, area: Rect, state: &TuiState) {
    let text = vec![Line::from(vec![
        Span::styled(
            " uplc-fuzz ",
            Style::default().fg(Color::Black).bg(Color::Cyan).bold(),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} workers", state.workers),
            Style::default().fg(Color::White),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("Plutus {}", state.plutus_version),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("seed {}", state.base_seed),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("output: {}", state.output_dir),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("  "),
        Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
    ])];
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn draw_sparkline(frame: &mut Frame, area: Rect, state: &TuiState) {
    let block = Block::default()
        .title(Span::styled(
            format!(
                " Throughput (peak: {}/s) ",
                format_number(state.peak_throughput)
            ),
            Style::default().fg(Color::Cyan).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let data = &state.throughput_history;
    let max_val = data.iter().copied().max().unwrap_or(1).max(1);

    let sparkline = Sparkline::default()
        .block(block)
        .data(data)
        .max(max_val)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(sparkline, area);
}

fn draw_big_numbers(
    frame: &mut Frame,
    area: Rect,
    stats: &Stats,
    _state: &TuiState,
    catalog_count: usize,
) {
    let elapsed = stats.start_time.elapsed().as_secs_f64();
    let iters = stats.iterations.load(Ordering::Relaxed);
    let rate = if elapsed > 0.0 {
        iters as f64 / elapsed
    } else {
        0.0
    };

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .split(area);

    let items: Vec<(&str, String, Color)> = vec![
        ("Programs", format_number(iters), Color::White),
        (
            "Rate",
            format!("{}/s", format_number(rate as u64)),
            Color::Green,
        ),
        (
            "Divergences",
            format_number(catalog_count as u64),
            if catalog_count > 0 {
                Color::Red
            } else {
                Color::Green
            },
        ),
        (
            "Success",
            format_number(stats.successes.load(Ordering::Relaxed)),
            Color::Cyan,
        ),
        ("Elapsed", format_duration(elapsed), Color::Yellow),
    ];

    for (i, (label, value, color)) in items.iter().enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                format!(" {label} "),
                Style::default().fg(Color::DarkGray),
            ));
        let text = Paragraph::new(Line::from(Span::styled(
            value.clone(),
            Style::default().fg(*color).bold(),
        )))
        .block(block)
        .alignment(Alignment::Center);
        frame.render_widget(text, cols[i]);
    }
}

fn draw_breakdown(frame: &mut Frame, area: Rect, state: &TuiState, catalog_count: usize) {
    let block = Block::default()
        .title(Span::styled(
            " Divergence Breakdown ",
            Style::default().fg(Color::Red).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let rows = vec![
        Row::new(vec![
            Cell::from("Result Mismatch").style(Style::default().fg(Color::Red)),
            Cell::from(format_number(state.result_mismatches))
                .style(Style::default().fg(Color::White)),
            Cell::from(bar(state.result_mismatches, catalog_count))
                .style(Style::default().fg(Color::Red)),
        ]),
        Row::new(vec![
            Cell::from("Budget Mismatch").style(Style::default().fg(Color::Yellow)),
            Cell::from(format_number(state.budget_mismatches))
                .style(Style::default().fg(Color::White)),
            Cell::from(bar(state.budget_mismatches, catalog_count))
                .style(Style::default().fg(Color::Yellow)),
        ]),
        Row::new(vec![
            Cell::from("Result+Budget").style(Style::default().fg(Color::Magenta)),
            Cell::from(format_number(state.result_and_budget))
                .style(Style::default().fg(Color::White)),
            Cell::from(bar(state.result_and_budget, catalog_count))
                .style(Style::default().fg(Color::Magenta)),
        ]),
        Row::new(vec![
            Cell::from("Panics").style(Style::default().fg(Color::LightRed)),
            Cell::from(format_number(state.panics_count)).style(Style::default().fg(Color::White)),
            Cell::from(bar(state.panics_count, catalog_count))
                .style(Style::default().fg(Color::LightRed)),
        ]),
        Row::new(vec![
            Cell::from("External").style(Style::default().fg(Color::Blue)),
            Cell::from(format_number(state.external_mismatches))
                .style(Style::default().fg(Color::White)),
            Cell::from(bar(state.external_mismatches, catalog_count))
                .style(Style::default().fg(Color::Blue)),
        ]),
    ];

    let widths = [
        Constraint::Length(18),
        Constraint::Length(10),
        Constraint::Min(10),
    ];
    let table = Table::new(rows, widths).block(block);
    frame.render_widget(table, area);
}

fn draw_log(frame: &mut Frame, area: Rect, state: &TuiState) {
    let block = Block::default()
        .title(Span::styled(
            " Recent Divergences ",
            Style::default().fg(Color::Yellow).bold(),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let visible = area.height.saturating_sub(2) as usize;
    let start = state.recent_divergences.len().saturating_sub(visible);
    let lines: Vec<Line> = state.recent_divergences[start..]
        .iter()
        .map(|msg| {
            let color = if msg.contains("Panic") {
                Color::LightRed
            } else if msg.contains("Result") {
                Color::Red
            } else if msg.contains("Budget") {
                Color::Yellow
            } else {
                Color::White
            };
            Line::from(Span::styled(msg.clone(), Style::default().fg(color)))
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn bar(count: u64, total: usize) -> String {
    if total == 0 {
        return String::new();
    }
    let pct = (count as f64 / total as f64 * 100.0) as usize;
    let filled = (count as f64 / total as f64 * 30.0) as usize;
    format!("{} {:>3}%", "█".repeat(filled), pct)
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}

fn format_duration(secs: f64) -> String {
    let s = secs as u64;
    if s >= 3600 {
        format!("{}h {:02}m {:02}s", s / 3600, (s % 3600) / 60, s % 60)
    } else if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{:.1}s", secs)
    }
}
