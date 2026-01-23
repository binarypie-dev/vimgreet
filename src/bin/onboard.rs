use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hypercube_utils::error::{HypercubeError, Result};
use hypercube_utils::event::{Event, EventHandler};
use hypercube_utils::onboard::{OnboardApp, OnboardAction, OnboardConfig};
use hypercube_utils::system;
use ratatui::prelude::*;
use std::io::stdout;
use std::panic;
use std::time::Duration;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "hypercube-onboard")]
#[command(author, version, about = "First-boot onboarding wizard for Hypercube Linux")]
struct Args {
    /// Path to onboard config file (default: /etc/hypercube/onboard.toml)
    #[arg(long)]
    config: Option<String>,

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

            info!("Starting hypercube-onboard");
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

    // Run onboard wizard
    let result = run_onboard(&mut terminal, args.config.as_deref(), args.dryrun).await;

    // Restore terminal
    restore_terminal()?;

    if let Err(ref e) = result {
        error!("Onboard error: {}", e);
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

async fn run_onboard(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    config_path: Option<&str>,
    dryrun: bool,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut events = EventHandler::new(tick_rate);

    // Load config from specified path, default path, or use defaults
    let mut config = match config_path {
        Some(path) => OnboardConfig::load_from(path).unwrap_or_default(),
        None => OnboardConfig::load().unwrap_or_default(),
    };

    // --dryrun flag overrides config
    if dryrun {
        config.general.dryrun = true;
    }

    let mut app = OnboardApp::new(config);

    loop {
        // Draw UI
        terminal
            .draw(|frame| hypercube_utils::onboard::ui::draw(frame, &app))
            .map_err(|e| HypercubeError::Terminal(e.to_string()))?;

        // Handle events
        if let Some(event) = events.next().await {
            match event {
                Event::Key(key) => {
                    if let Some(action) = app.handle_key(key) {
                        match action {
                            OnboardAction::LaunchExternal(program, args) => {
                                // Leave alternate screen for external program
                                restore_terminal()?;

                                // Run external program
                                let status = std::process::Command::new(&program)
                                    .args(&args)
                                    .status();

                                // Re-enter alternate screen
                                *terminal = setup_terminal()?;

                                if let Err(e) = status {
                                    app.set_error(format!("Failed to launch {}: {}", program, e));
                                }
                            }
                            OnboardAction::Reboot => {
                                if let Err(e) = system::reboot(app.is_dryrun()) {
                                    app.set_error(format!("Reboot failed: {}", e));
                                }
                            }
                            OnboardAction::Poweroff => {
                                if let Err(e) = system::poweroff(app.is_dryrun()) {
                                    app.set_error(format!("Poweroff failed: {}", e));
                                }
                            }
                            OnboardAction::ExecuteStep => {
                                app.execute_current_step().await;
                            }
                            OnboardAction::ExecuteReview => {
                                app.execute_review().await;
                            }
                            OnboardAction::ExecuteUpdate => {
                                app.execute_update().await;
                            }
                            OnboardAction::ExitToLogin => {
                                app.finish_setup().await;
                            }
                            OnboardAction::TransitionToLogin => {
                                app.set_info("Setup complete!".to_string());
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                return Ok(());
                            }
                        }
                    }
                }
                Event::Mouse => {}
                Event::Resize => {}
                Event::Tick => {
                    app.tick();
                }
            }
        }

        if app.should_exit {
            break;
        }
    }

    Ok(())
}
