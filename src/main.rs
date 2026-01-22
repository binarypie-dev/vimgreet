mod app;
mod error;
mod event;
mod ipc;
mod onboard;
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
    /// Run onboarding wizard for first-boot system setup
    #[arg(long)]
    onboard: bool,

    /// Simulate all operations without making real changes
    #[arg(long)]
    dryrun: bool,

    /// Path to onboard config file (default: /etc/vimgreet/onboard.toml)
    #[arg(long)]
    config: Option<String>,

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

    // Branch to onboard mode if requested
    if args.onboard {
        info!("Starting onboard wizard");
        let result = run_onboard(&mut terminal, args.config.as_deref(), args.dryrun).await;

        match result {
            Ok(OnboardResult::TransitionToLogin) => {
                // Dryrun mode completed, continue to login screen
                info!("Onboard completed in dryrun mode, transitioning to login screen");
            }
            Ok(OnboardResult::Exit) => {
                restore_terminal()?;
                return Ok(());
            }
            Err(e) => {
                restore_terminal()?;
                error!("Onboard error: {}", e);
                return Err(e);
            }
        }
    }

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

/// Result from onboard wizard
pub enum OnboardResult {
    /// Wizard exited normally (cancelled, rebooted, etc.)
    Exit,
    /// Wizard completed in dryrun mode, continue to login screen
    TransitionToLogin,
}

async fn run_onboard(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    config_path: Option<&str>,
    dryrun: bool,
) -> Result<OnboardResult> {
    use onboard::{OnboardApp, OnboardAction};

    let tick_rate = Duration::from_millis(250);
    let mut events = EventHandler::new(tick_rate);

    // Load config from specified path, default path, or use defaults
    let mut config = match config_path {
        Some(path) => onboard::OnboardConfig::load_from(path).unwrap_or_default(),
        None => onboard::OnboardConfig::load().unwrap_or_default(),
    };

    // --dryrun flag overrides config
    if dryrun {
        config.general.dryrun = true;
    }

    let mut app = OnboardApp::new(config);

    loop {
        // Draw UI
        terminal
            .draw(|frame| onboard::ui::draw(frame, &app))
            .map_err(|e| error::VimgreetError::Terminal(e.to_string()))?;

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
                                // Clean up greetd config and exit to login
                                app.finish_setup().await;
                            }
                            OnboardAction::TransitionToLogin => {
                                // Dryrun mode: show fake reboot message and transition to login
                                app.set_info("Simulating reboot... transitioning to login screen.".to_string());
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                return Ok(OnboardResult::TransitionToLogin);
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

    Ok(OnboardResult::Exit)
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
