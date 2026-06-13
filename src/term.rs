use anyhow::Result;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, channel},
};

// #[derive(Debug, Clone, Default)]
// pub struct Execution {
//     pub command: String,
//     pub raw_prompt: String,
//     pub raw_output: String,
//     plain_output: Option<String>,
// }
//
// impl Execution {
//     pub fn plain_output(&mut self) -> &str {
//         self.plain_output
//             .get_or_insert(strip_ansi_escapes::strip_str(
//                 self.raw_output.replace("\t", "  "),
//             ))
//     }
//
//     pub fn write_raw_prompt<W: Write>(&self, mut writer: W) -> Result<()> {
//         writer.write_all(self.raw_prompt.as_bytes())?;
//         writer.flush()?;
//         Ok(())
//     }
//
//     pub fn write_raw_output<W: Write>(&self, mut writer: W) -> Result<()> {
//         writer.write_all(self.raw_output.as_bytes())?;
//         writer.flush()?;
//         Ok(())
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub command: String,
    pub raw_prompt: String,
    pub raw_output: String,
    plain_output: Option<String>,
}

impl Execution {
    pub fn plain_output(&mut self) -> &str {
        self.plain_output
            .get_or_insert(strip_ansi_escapes::strip_str(
                self.raw_output.replace("\t", "  "),
            ))
    }

    pub fn write_raw_prompt<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(self.raw_prompt.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub fn write_raw_output<W: Write>(&self, mut writer: W) -> Result<()> {
        writer.write_all(self.raw_output.as_bytes())?;
        writer.flush()?;
        Ok(())
    }
}

// pub struct Terminal {
//     writer: Box<dyn Write + Send>,
//     reader: Box<dyn Read + Send>,
//     child: Box<dyn Child + Send + Sync>,
// }
//
// impl Drop for Terminal {
//     fn drop(&mut self) {
//         let _ = self.child.kill();
//         let _ = self.child.wait();
//     }
// }
//
// impl Terminal {
//     pub fn new(cmd: &str) -> Result<Self> {
//         let pty = native_pty_system();
//
//         let pair = pty.openpty(PtySize {
//             rows: 24,
//             cols: 80,
//             pixel_width: 0,
//             pixel_height: 0,
//         })?;
//
//         let child = pair.slave.spawn_command(CommandBuilder::new(cmd))?;
//         drop(pair.slave);
//
//         Ok(Self {
//             writer: pair.master.take_writer()?,
//             reader: pair.master.try_clone_reader()?,
//             child: child,
//         })
//     }
//
//     fn drain(&mut self) -> Result<()> {
//         let mut buf = [0u8; 4096];
//         loop {
//             match self.reader.read(&mut buf) {
//                 Ok(0) => break,
//                 Ok(_) => continue,
//                 Err(e)
//                     if e.kind() == std::io::ErrorKind::WouldBlock
//                         || e.kind() == std::io::ErrorKind::TimedOut =>
//                 {
//                     break;
//                 }
//                 Err(e) => return Err(e.into()),
//             }
//         }
//         Ok(())
//     }
//
//     pub fn execute_v2(&mut self, command: &str) -> Result<Execution> {
//         let lines = vec!["cat << EOF > ~/Desktop/test.txt", "Hello, World!", "EOF"];
//         for line in lines {
//             writeln!(self.writer, "{}", line)?;
//             self.writer.flush()?;
//             std::thread::sleep(std::time::Duration::from_millis(100));
//         }
//         std::thread::sleep(std::time::Duration::from_millis(500));
//
//         println!("{}", command);
//         let e = Execution {
//             command: command.into(),
//             ..Default::default()
//         };
//
//         // for line in command.lines() {
//         //     writeln!(self.writer, "{}", line)?;
//         //     self.writer.flush()?;
//         // }
//         //
//         // std::thread::sleep(std::time::Duration::from_millis(500));
//
//         Ok(e)
//     }
//
//     pub fn execute(&mut self, command: &str) -> Result<Execution> {
//         println!("------------------\nCOMMAND: {}", command);
//
//         let mut e = Execution {
//             command: command.into(),
//             ..Default::default()
//         };
//         let mut skip = 0;
//
//         println!("START");
//
//         for line in command.lines() {
//             writeln!(self.writer, "{}", line)?;
//             self.writer.flush()?;
//             let _ = self.drain();
//             skip += 1;
//         }
//
//         println!("WRITELINE EOF");
//         writeln!(self.writer, r#"echo "__EOFEX__""#)?;
//         self.writer.flush()?;
//
//         let lines_reader = std::io::BufReader::new(&mut self.reader).lines();
//
//         for (n, line) in lines_reader.skip(1).enumerate() {
//             let line = line?;
//             if n < skip {
//                 e.raw_prompt.push_str(&line);
//                 e.raw_prompt.push('\n');
//                 continue;
//             }
//             if line == "__EOFEX__" {
//                 e.raw_output.truncate(e.raw_output.len() - 1);
//                 if let Some(f) = e.raw_output.rfind("\n") {
//                     e.raw_output.truncate(f);
//                     e.raw_output.push('\n');
//                 } else {
//                     e.raw_output.clear();
//                 }
//                 break;
//             }
//             e.raw_output.push_str(&line);
//             e.raw_output.push('\n');
//         }
//         println!("{}", e.raw_prompt);
//         println!("{}", e.raw_output);
//         Ok(e)
//     }
// }

pub struct Terminal {
    writer: Box<dyn Write + Send>,
    line_rx: Receiver<String>,
    _child_thread: std::thread::JoinHandle<()>,
}

const ROWS: u16 = 30;
const COLS: u16 = 90;

impl Terminal {
    pub fn spawn<F>(shell: &'static str, on_output: F) -> Result<Self>
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
        let child_thread = std::thread::spawn(move || {
            let cmd = CommandBuilder::new(shell);
            // cmd.env("TERM", "xterm-256color");

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
        });

        Ok(Self {
            writer,
            line_rx: rx,
            _child_thread: child_thread,
        })
    }

    pub fn execute(&mut self, command: &str) -> Result<Execution> {
        println!("COMMAND: {}", command);
        let mut e = Execution {
            command: command.into(),
            ..Default::default()
        };
        let command = format!(r#"{command} ; echo "__End_OF_Command__""#);
        let mut skip = 0;
        let total_lines = command.lines().count();
        // println!("TOTAL LINES: {}", total_lines);
        for (n, line) in command.lines().enumerate() {
            writeln!(self.writer, "{}", line)?;
            self.writer.flush()?;
            // println!("WRITELN {n}/{total_lines} LEN {} : {line}", line.len());
            let _ = self.line_rx.recv()?;
            skip += 1;
        }
        // println!("--END--");
        let mut n = 0;
        while let Ok(line) = self.line_rx.recv() {
            println!("{:?}  [LEN {}]", line, line.len());
            if line == "__End_OF_Command__" {
                println!("break");
                break;
            }
            if n < skip {
                println!("SKIP: {} {} = {:?}", n, skip, line);
                e.raw_prompt.push_str(&line);
                e.raw_prompt.push('\n');
                n += 1;
                continue;
            }
            e.raw_output.push_str(&line);
            e.raw_output.push('\n');
        }
        println!("end");
        Ok(e)
    }
}
