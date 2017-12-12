use Result;
use std::process::Command;

pub struct Link {
    name: String,
}

impl Link {
    pub fn new(name: String) -> Self {
        Self{name}
    }
}

impl ::Pass for Link {
    type Input  = Vec<String>;
    type Output = ();

    fn run(self, object_files: Self::Input) -> Result<Self::Output> {
        let mut command = Command::new("gcc");
        let command = command
            .arg("-o")
            .arg(self.name);
        let _ = object_files.iter()
            .fold( command, |cmd, arg| cmd.arg(arg))
            .output();
        Ok(())
    }
}

