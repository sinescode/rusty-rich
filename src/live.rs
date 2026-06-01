//! Live — auto-updating display. Equivalent to Rich's `live.py`.

use std::io::{self, Write};
use std::time::Instant;

use crate::console::{ConsoleOptions, DynRenderable, Renderable};

/// A writer that captures output for live display.
pub struct LiveWriter {
    buffer: Vec<u8>,
}

impl LiveWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn capture(&self) -> &[u8] {
        &self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

impl Write for LiveWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Manages a live-updating region of the terminal.
pub struct Live {
    renderable: Option<DynRenderable>,
    screen: bool,
    auto_refresh: bool,
    refresh_per_second: f64,
    transient: bool,
    started: Option<Instant>,
    previous_line_count: usize,
    redirect_stdout: bool,
    redirect_stderr: bool,
    writers: Vec<LiveWriter>,
}

impl std::fmt::Debug for Live {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Live")
            .field("screen", &self.screen)
            .field("started", &self.started)
            .finish()
    }
}

impl Live {
    pub fn new(renderable: impl Renderable + Send + Sync + 'static) -> Self {
        Self {
            renderable: Some(DynRenderable::new(renderable)),
            screen: false,
            auto_refresh: true,
            refresh_per_second: 4.0,
            transient: false,
            started: None,
            previous_line_count: 0,
            redirect_stdout: true,
            redirect_stderr: true,
            writers: Vec::new(),
        }
    }

    pub fn screen(mut self) -> Self { self.screen = true; self }
    pub fn no_auto_refresh(mut self) -> Self { self.auto_refresh = false; self }
    pub fn refresh_per_second(mut self, rate: f64) -> Self { self.refresh_per_second = rate; self }
    pub fn transient(mut self) -> Self { self.transient = true; self }
    pub fn redirect_stdout(mut self, redirect: bool) -> Self { self.redirect_stdout = redirect; self }
    pub fn redirect_stderr(mut self, redirect: bool) -> Self { self.redirect_stderr = redirect; self }

    /// Register a writer whose captured content will be rendered during refresh.
    pub fn add_writer(&mut self, writer: LiveWriter) { self.writers.push(writer); }

    /// Create a LiveWriter that captures output while Live is active.
    pub fn create_writer() -> LiveWriter {
        LiveWriter::new()
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.started = Some(Instant::now());
        if self.screen {
            write!(io::stdout(), "\x1b[?1049h")?;
        }
        write!(io::stdout(), "\x1b[?25l")?;
        self.refresh()
    }

    pub fn stop(&mut self) -> io::Result<()> {
        if self.transient {
            for _ in 0..self.previous_line_count {
                write!(io::stdout(), "\x1b[1A\x1b[2K")?;
            }
        }
        if self.screen {
            write!(io::stdout(), "\x1b[?1049l")?;
        }
        write!(io::stdout(), "\x1b[?25h")?;
        io::stdout().flush()?;
        self.started = None;
        Ok(())
    }

    pub fn update(&mut self, renderable: impl Renderable + Send + Sync + 'static) -> io::Result<()> {
        self.renderable = Some(DynRenderable::new(renderable));
        self.refresh()
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        if let Some(ref renderable) = self.renderable {
            let opts = ConsoleOptions::default();
            let result = renderable.render(&opts);

            if self.previous_line_count > 0 {
                write!(io::stdout(), "\x1b[{}F", self.previous_line_count)?;
            }

            let ansi = result.to_ansi();
            let line_count = ansi.lines().count();

            write!(io::stdout(), "{ansi}")?;
            if line_count < self.previous_line_count {
                for _ in line_count..self.previous_line_count {
                    write!(io::stdout(), "\x1b[2K\n")?;
                }
            }

            self.previous_line_count = line_count;

            // Write captured writer content
            for writer in &self.writers {
                let captured = String::from_utf8_lossy(writer.capture());
                if !captured.is_empty() {
                    write!(io::stdout(), "{}", captured)?;
                }
            }

            io::stdout().flush()?;
        }
        Ok(())
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
