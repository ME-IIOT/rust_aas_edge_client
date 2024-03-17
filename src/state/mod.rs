use std::sync::Mutex;

pub struct AppState {
    pub health_check_response: Mutex<String>,
}
