use crate::error::{Result, HypercubeError};
use greetd_ipc::{codec::SyncCodec, AuthMessageType, ErrorType, Request, Response};
use std::os::unix::net::UnixStream;
use tracing::{debug, error, info};

pub enum AuthResponse {
    PromptSecret(String),
    PromptVisible(String),
    Info(String),
    Error(String),
    Success,
}

pub struct GreetdClient {
    stream: Option<UnixStream>,
    demo_mode: bool,
}

impl GreetdClient {
    pub async fn connect() -> Result<Self> {
        let socket_path =
            std::env::var("GREETD_SOCK").map_err(|_| HypercubeError::SocketNotFound)?;

        info!("Connecting to greetd socket: {}", socket_path);
        let stream = UnixStream::connect(&socket_path)?;

        Ok(Self {
            stream: Some(stream),
            demo_mode: false,
        })
    }

    pub fn demo() -> Self {
        info!("Running in demo mode");
        Self {
            stream: None,
            demo_mode: true,
        }
    }

    pub async fn create_session(&mut self, username: &str) -> Result<AuthResponse> {
        if self.demo_mode {
            return Ok(AuthResponse::PromptSecret("Password: ".to_string()));
        }

        let response = self.send(Request::CreateSession {
            username: username.to_string(),
        })?;

        self.handle_response(response)
    }

    pub async fn post_auth_response(&mut self, response: Option<String>) -> Result<AuthResponse> {
        if self.demo_mode {
            if response.as_deref() == Some("demo") {
                return Ok(AuthResponse::Success);
            }
            return Ok(AuthResponse::Error(
                "Invalid password (hint: use 'demo')".to_string(),
            ));
        }

        let resp = self.send(Request::PostAuthMessageResponse { response })?;
        self.handle_response(resp)
    }

    pub async fn start_session(&mut self, cmd: Vec<String>, env: Vec<String>) -> Result<()> {
        if self.demo_mode {
            info!(
                "Demo mode: would start session with cmd={:?}, env={:?}",
                cmd, env
            );
            return Ok(());
        }

        let response = self.send(Request::StartSession { cmd, env })?;

        match response {
            Response::Success => Ok(()),
            Response::Error {
                error_type,
                description,
            } => {
                error!("Session start failed: {:?} - {}", error_type, description);
                Err(HypercubeError::SessionFailed(description))
            }
            _ => Err(HypercubeError::SessionFailed(
                "Unexpected response".to_string(),
            )),
        }
    }

    pub async fn cancel_session(&mut self) -> Result<()> {
        if self.demo_mode {
            return Ok(());
        }

        let response = self.send(Request::CancelSession)?;

        match response {
            Response::Success => Ok(()),
            Response::Error { description, .. } => Err(HypercubeError::AuthFailed(description)),
            _ => Ok(()),
        }
    }

    fn send(&mut self, request: Request) -> Result<Response> {
        let stream = self
            .stream
            .as_mut()
            .ok_or(HypercubeError::SocketNotFound)?;

        debug!("Sending request: {:?}", request);
        request.write_to(stream)?;

        let response = Response::read_from(stream)?;
        debug!("Received response: {:?}", response);
        Ok(response)
    }

    fn handle_response(&self, response: Response) -> Result<AuthResponse> {
        match response {
            Response::Success => Ok(AuthResponse::Success),
            Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => match auth_message_type {
                AuthMessageType::Secret => Ok(AuthResponse::PromptSecret(auth_message)),
                AuthMessageType::Visible => Ok(AuthResponse::PromptVisible(auth_message)),
                AuthMessageType::Info => Ok(AuthResponse::Info(auth_message)),
                AuthMessageType::Error => Ok(AuthResponse::Error(auth_message)),
            },
            Response::Error {
                error_type,
                description,
            } => {
                let msg = match error_type {
                    ErrorType::AuthError => "Authentication failed".to_string(),
                    ErrorType::Error => description,
                };
                Ok(AuthResponse::Error(msg))
            }
        }
    }
}
