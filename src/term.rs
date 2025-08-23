use std::env;

pub fn should_use_color() -> bool {
    // Check environment variables that control color output
    if let Ok(val) = env::var("NO_COLOR") {
        if !val.is_empty() {
            return false;
        }
    }

    if let Ok(val) = env::var("CARGO_TERM_COLOR") {
        return match val.as_str() {
            "always" => true,
            "never" => false,
            "auto" => is_terminal(),
            _ => is_terminal(),
        };
    }

    if let Ok(val) = env::var("TERM") {
        if val == "dumb" {
            return false;
        }
    }

    // Default to TTY detection
    is_terminal()
}

fn is_terminal() -> bool {
    // Check if stderr is a terminal (since that's where we output errors)
    atty::is(atty::Stream::Stderr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_no_color_env_var() {
        env::set_var("NO_COLOR", "1");
        assert_eq!(should_use_color(), false);
        env::remove_var("NO_COLOR");
    }

    #[test]
    fn test_cargo_term_color_always() {
        env::set_var("CARGO_TERM_COLOR", "always");
        assert_eq!(should_use_color(), true);
        env::remove_var("CARGO_TERM_COLOR");
    }

    #[test]
    fn test_cargo_term_color_never() {
        env::set_var("CARGO_TERM_COLOR", "never");
        assert_eq!(should_use_color(), false);
        env::remove_var("CARGO_TERM_COLOR");
    }

    #[test]
    fn test_term_dumb() {
        // Temporarily remove other color vars
        let no_color = env::var("NO_COLOR");
        let cargo_color = env::var("CARGO_TERM_COLOR");
        
        env::remove_var("NO_COLOR");
        env::remove_var("CARGO_TERM_COLOR");
        
        env::set_var("TERM", "dumb");
        assert_eq!(should_use_color(), false);
        
        env::remove_var("TERM");
        
        // Restore previous values
        if let Ok(val) = no_color {
            env::set_var("NO_COLOR", val);
        }
        if let Ok(val) = cargo_color {
            env::set_var("CARGO_TERM_COLOR", val);
        }
    }
}