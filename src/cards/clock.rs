use chrono::{Local, Timelike};

pub fn current_user_name() -> String {
    whoami::username().unwrap_or_else(|_| "User".to_string())
}

pub fn user_greeting() -> String {
    if Local::now().hour() < 12 {
        format!("Good morning, {}!", current_user_name())
    } else if Local::now().hour() < 18 {
        format!("Good afternoon, {}!", current_user_name())
    } else {
        format!("Good evening, {}!", current_user_name())
    }
}

pub fn current_time() -> String {
    Local::now().format("%H:%M").to_string()
}

pub fn current_date() -> String {
    Local::now().format("%d %B, %A").to_string()
}