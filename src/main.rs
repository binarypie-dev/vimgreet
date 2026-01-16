mod app;
mod error;
mod event;
mod ipc;
mod system;
mod ui;
mod vim;

use app::{App, AppAction};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Result;
use event::{Event, EventHandler};
use ipc::GreetdClient;
use ratatui::prelude::*;
use std::io::stdout;
use std::panic;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "vimgreet")]
#[command(author, version, about = "A neovim-inspired greeter for greetd")]
struct Args {
    /// Run in demo mode without connecting to greetd
    #[arg(long)]
    demo: bool,

    /// Log file path (logging disabled if not specified)
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Set up logging only if log file is specified
    if let Some(ref log_path) = args.log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();

        if let Some(file) = file {
            let filter = EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"));

            tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_writer(file)
                .with_ansi(false)
                .init();

            info!("Starting vimgreet");
        }
    }

    // Set up panic handler to restore terminal
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Create greetd client
    let mut client = if args.demo {
        GreetdClient::demo()
    } else {
        match GreetdClient::connect().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to greetd: {}. Use --demo to test without greetd.", e);
                restore_terminal()?;
                return Err(e);
            }
        }
    };

    // Create app state
    let mut app = App::new(args.demo);

    // Run the app
    let result = run(&mut terminal, &mut app, &mut client).await;

    // Restore terminal
    restore_terminal()?;

    if let Err(ref e) = result {
        error!("Application error: {}", e);
    }

    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode().map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let terminal =
        Terminal::new(backend).map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;
    Ok(())
}

async fn run(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    client: &mut GreetdClient,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut events = EventHandler::new(tick_rate);

    loop {
        // Draw UI
        terminal
            .draw(|frame| ui::draw(frame, app))
            .map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    if let Some(action) = app.handle_key(key) {
                        match action {
                            AppAction::Login => {
                                app.login(client).await;
                            }
                            AppAction::Cancel => {
                                let _ = client.cancel_session().await;
                                app.password.clear();
                                app.working = false;
                            }
                            AppAction::Reboot => {
                                if let Err(e) = system::reboot() {
                                    app.set_error(format!("Reboot failed: {}", e));
                                }
                            }
                            AppAction::Poweroff => {
                                if let Err(e) = system::poweroff() {
                                    app.set_error(format!("Poweroff failed: {}", e));
                                }
                            }
                        }
                    }
                }
                Event::Mouse => {
                    // Mouse support for future use (cage compatibility)
                }
                Event::Resize => {
                    // Terminal will redraw on next tick
                }
                Event::Tick => {
                    // Periodic tick for animations/updates
                }
            }
        }

        if app.should_exit {
            break;
        }
    }

    Ok(())
}
