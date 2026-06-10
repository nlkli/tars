use anyhow::Result;
use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use std::io::{BufRead, Read, Write};

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

pub struct Terminal {
    writer: Box<dyn Write + Send>,
    reader: Box<dyn Read + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Terminal {
    pub fn new(cmd: &str) -> Result<Self> {
        let pty = native_pty_system();

        let pair = pty.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let child = pair.slave.spawn_command(CommandBuilder::new(cmd))?;
        drop(pair.slave);

        Ok(Self {
            writer: pair.master.take_writer()?,
            reader: pair.master.try_clone_reader()?,
            child: child,
        })
    }

    pub fn execute(&mut self, command: &str) -> Result<Execution> {
        let command = command.trim();
        let mut e = Execution {
            command: command.into(),
            ..Default::default()
        };
        let mut skip = 0;
        for line in command.lines() {
            writeln!(self.writer, "{}", line.trim())?;
            skip += 1;
        }
        self.writer.flush()?;

        writeln!(self.writer, r#"echo "__EOFEX__""#)?;
        self.writer.flush()?;

        let lines = std::io::BufReader::new(&mut self.reader).lines();
        for (n, line) in lines.skip(skip + 1).enumerate() {
            let line = line?;
            if n < skip {
                e.raw_prompt.push_str(&line);
                e.raw_prompt.push('\n');
                continue;
            }
            if line == "__EOFEX__" {
                e.raw_output.truncate(e.raw_output.len() - 1);
                if let Some(f) = e.raw_output.rfind("\n") {
                    e.raw_output.truncate(f);
                    e.raw_output.push('\n');
                } else {
                    e.raw_output.clear();
                }
                break;
            }
            e.raw_output.push_str(&line);
            e.raw_output.push('\n');
        }
        Ok(e)
    }
}
