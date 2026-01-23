use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hypercube_utils::error::{HypercubeError, Result};
use hypercube_utils::event::{Event, EventHandler};
use hypercube_utils::greeter::{App, AppAction};
use hypercube_utils::ipc::GreetdClient;
use hypercube_utils::system;
use ratatui::prelude::*;
use std::io::stdout;
use std::panic;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "hypercube-greeter")]
#[command(author, version, about = "A vim-inspired greeter for greetd")]
struct Args {
    /// Simulate all operations without making real changes
    #[arg(long)]
    dryrun: bool,

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

            info!("Starting hypercube-greeter");
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
    let mut client = if args.dryrun {
        GreetdClient::demo()
    } else {
        match GreetdClient::connect().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to greetd: {}. Use --dryrun to test without greetd.", e);
                restore_terminal()?;
                return Err(e);
            }
        }
    };

    // Create app state
    let mut app = App::new(args.dryrun);

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
    enable_raw_mode().map_err(|e| HypercubeError::Terminal(e.to_string()))?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| HypercubeError::Terminal(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let terminal =
        Terminal::new(backend).map_err(|e| HypercubeError::Terminal(e.to_string()))?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode().map_err(|e| HypercubeError::Terminal(e.to_string()))?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)
        .map_err(|e| HypercubeError::Terminal(e.to_string()))?;
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
            .draw(|frame| hypercube_utils::greeter::ui::draw(frame, app))
            .map_err(|e| HypercubeError::Terminal(e.to_string()))?;

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
                                if let Err(e) = system::reboot(app.demo_mode) {
                                    app.set_error(format!("Reboot failed: {}", e));
                                }
                            }
                            AppAction::Poweroff => {
                                if let Err(e) = system::poweroff(app.demo_mode) {
                                    app.set_error(format!("Poweroff failed: {}", e));
                                }
                            }
                        }
                    }
                }
                Event::Mouse => {}
                Event::Resize => {}
                Event::Tick => {}
            }
        }

        if app.should_exit {
            break;
        }
    }

    Ok(())
}
