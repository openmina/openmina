pub mod snark;
pub mod snarker;

pub type CommandError = Box<dyn std::error::Error>;

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "openmina", about = "Openmina Cli")]
pub enum Command {
    Snark(snark::Snark),
    Snarker(snarker::Snarker),
}

impl Command {
    pub fn run(self) -> Result<(), crate::CommandError> {
        match self {
            Self::Snark(v) => v.run(),
            Self::Snarker(v) => v.run(),
        }
    }
}
