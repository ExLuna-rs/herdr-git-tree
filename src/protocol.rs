//! Terminal image protocol detection.

/// Supported graph rendering protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphProtocol {
    /// Kitty terminal graphics protocol.
    KittyGraphics,
    /// iTerm2 inline images protocol.
    Iterm2,
    /// Unicode-only rendering (works everywhere).
    UnicodeOnly,
}

/// Auto-detect the best available image protocol.
pub fn detect_protocol() -> GraphProtocol {
    // Check TERM_PROGRAM
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        let lower = prog.to_lowercase();
        if lower.contains("iterm") {
            return GraphProtocol::Iterm2;
        }
        if lower.contains("wezterm") {
            return GraphProtocol::KittyGraphics;
        }
    }

    // Check TERM
    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") {
            return GraphProtocol::KittyGraphics;
        }
    }

    // Check KITTY_WINDOW_ID
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return GraphProtocol::KittyGraphics;
    }

    // Check KONSOLE_VERSION (Konsole supports kitty protocol)
    if std::env::var("KONSOLE_VERSION").is_ok() {
        return GraphProtocol::KittyGraphics;
    }

    GraphProtocol::UnicodeOnly
}
