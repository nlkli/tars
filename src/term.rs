use anyhow::Result;
use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use std::{
    collections::VecDeque,
    io::{BufRead, Read, Write},
};

#[derive(Debug, Clone, Default)]
pub struct Execution {
    pub command: String,
    pub raw_input: String,
    pub raw_output: String,
    pub output: String,
}

pub struct Terminal {
    pub executions: VecDeque<Execution>,
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
            executions: VecDeque::new(),
            writer: pair.master.take_writer()?,
            reader: pair.master.try_clone_reader()?,
            child: child,
        })
    }

    pub fn execute(&mut self, command: &str) -> Result<usize> {
        let tcommand = command.trim();
        let mut e = Execution {
            command: tcommand.into(),
            ..Default::default()
        };
        let mut skip = 0;
        for c_line in tcommand.lines() {
            writeln!(self.writer, "{}", c_line.trim())?;
            skip += 1;
        }
        self.writer.flush()?;

        writeln!(self.writer, r#"echo "__EOFEX__""#)?;
        self.writer.flush()?;

        let lines = std::io::BufReader::new(&mut self.reader).lines();
        for (n, line) in lines.skip(skip + 1).enumerate() {
            let line = line?;
            if n < skip {
                e.raw_input.push_str(&line);
                e.raw_input.push('\n');
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
        e.output = strip_ansi_escapes::strip_str(&e.raw_output.replace("\t", "  "));
        println!("-- {:?}", e.output);
        self.executions.push_back(e);
        Ok(self.executions.len() - 1)
    }

    pub fn execute_many(&mut self, commands: &[String]) -> Result<usize> {
        for command in commands.iter() {
            let _ = self.execute(command)?;
        }
        Ok(self.executions.len() - 1)
    }
}
