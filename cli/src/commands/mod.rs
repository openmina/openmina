pub mod snark;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "openmina", about = "Openmina Cli")]
pub enum Command {
    Snark(snark::Snark),
}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self {
            Self::Snark(v) => v.run(),
        }
    }
}
