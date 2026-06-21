use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
    time::{Duration, Instant},
};

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub command: String,
    pub raw_prompt: String,
    pub raw_output: String,
    pub duration: Duration,
    plain_output: Option<String>,
}

impl Execution {
    pub fn plain_output(&mut self) -> &str {
        self.plain_output
            .get_or_insert(strip_ansi_escapes::strip_str(
                self.raw_output.replace("\t", "  "),
            ))
    }

    // pub fn write_raw_prompt<W: Write>(&self, mut writer: W) -> Result<()> {
    //     writer.write_all(self.raw_prompt.as_bytes())?;
    //     writer.flush()?;
    //     Ok(())
    // }
    //
    // pub fn write_raw_output<W: Write>(&self, mut writer: W) -> Result<()> {
    //     writer.write_all(self.raw_output.as_bytes())?;
    //     writer.flush()?;
    //     Ok(())
    // }
}

pub struct Terminal {
    pub shell: String,
    writer: Box<dyn Write + Send>,
    line_rx: Receiver<String>,
    _child_thread: std::thread::JoinHandle<()>,
}

const ROWS: u16 = 30;
const COLS: u16 = 90;

impl Terminal {
    pub fn spawn<F>(
        shell: String,
        on_output: F,
        env: Option<HashMap<String, String>>,
    ) -> Result<Self>
    where
        F: Fn(&[u8]) + Send + 'static,
    {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows: ROWS,
            cols: COLS,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let writer = pair.master.take_writer()?;
        let mut reader = pair.master.try_clone_reader()?;

        let (tx, rx) = channel();

        let slave = pair.slave;
        let child_thread = {
            let mut cmd = CommandBuilder::new(&shell);

            std::thread::spawn(move || {
                if let Some(env) = env {
                    for (k, v) in env.into_iter() {
                        cmd.env(k, v);
                    }
                }
                let mut child = match slave.spawn_command(cmd) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[TerminalSession] spawn failed: {e}");
                        return;
                    }
                };
                let mut buf = [0u8; 4096];
                let mut line_buf = Vec::new();
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            on_output(&buf[..n]);
                            for byte in buf[..n].iter() {
                                let byte = *byte;
                                if byte != b'\n' {
                                    line_buf.push(byte);
                                    continue;
                                }
                                if let Some(last) = line_buf.last() {
                                    if *last == b'\r' {
                                        line_buf.truncate(line_buf.len() - 1);
                                    }
                                }
                                if tx
                                    .send(String::from_utf8_lossy(&line_buf).to_string())
                                    .is_err()
                                {
                                    break;
                                };
                                line_buf.truncate(0);
                            }
                        }
                    }
                }
                let _ = child.kill();
                let _ = child.wait();
            })
        };

        Ok(Self {
            shell,
            writer,
            line_rx: rx,
            _child_thread: child_thread,
        })
    }

    fn recv_line_with_timeout(&mut self, timeout: &Option<Duration>) -> Result<String> {
        Ok(if let Some(t) = timeout {
            self.line_rx.recv_timeout(*t)?
        } else {
            self.line_rx.recv()?
        })
    }

    pub fn pwd(&mut self) -> Result<PathBuf> {
        const END_MARKER: &str = "__PWD_END__";
        writeln!(self.writer, "pwd; echo {}", END_MARKER)?;
        self.writer.flush()?;
        let mut path = None;
        loop {
            let line = self.line_rx.recv()?;

            if line == END_MARKER {
                break Ok(PathBuf::from(
                    path.ok_or_else(|| anyhow::anyhow!("pwd output not found"))?,
                ));
            }

            path = Some(line);
        }
    }

    // fn drain_line_rx(&mut self) {
    //     loop {
    //         match self.line_rx.try_recv() {
    //             Ok(_) => {}
    //             Err(TryRecvError::Empty) => break,
    //             Err(TryRecvError::Disconnected) => break,
    //         }
    //     }
    // }

    pub fn execute(&mut self, command: &str, timeout: Option<Duration>) -> Result<Execution> {
        let start = Instant::now();
        let mut e = Execution {
            command: command.into(),
            ..Default::default()
        };
        const EOC_MARKER: &str = "__END_OF_COMMAND__";
        const EOC_PROMPT: &str = "; echo __END_OF_COMMAND__";
        let mut skip = 1;
        let mut lines = command.lines();
        if let Some(line) = lines.next() {
            self.writer.write_all(line.as_bytes())?;
            self.writer.flush()?;
        }
        for line in lines {
            self.writer.write(&[b'\n'])?;
            self.writer.write_all(line.as_bytes())?;
            self.writer.flush()?;
            let _ = self.recv_line_with_timeout(&timeout)?;
            skip += 1;
        }
        self.writer.write_all(EOC_PROMPT.as_bytes())?;
        self.writer.write(&[b'\n'])?;
        self.writer.flush()?;
        let _ = self.recv_line_with_timeout(&timeout)?;
        let mut n = 0;
        loop {
            let line = self.recv_line_with_timeout(&timeout)?;
            if line.ends_with(EOC_MARKER) {
                if line.len() > EOC_MARKER.len() {
                    e.raw_output.push_str(line.trim_end_matches(EOC_MARKER));
                    e.raw_output.push('\n');
                }
                break;
            }
            if n < skip {
                e.raw_prompt.push_str(&line);
                if let Some(n) = e.raw_prompt.rfind(EOC_PROMPT) {
                    e.raw_prompt.truncate(n);
                }
                e.raw_prompt.push('\n');
                n += 1;
                continue;
            }
            e.raw_output.push_str(&line);
            e.raw_output.push('\n');
        }
        e.duration = start.elapsed();
        Ok(e)
    }
}
