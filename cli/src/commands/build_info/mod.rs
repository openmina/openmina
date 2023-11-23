use node::BuildEnv;

/// Displays openmina version, commit etc.
#[derive(Debug, clap::Args)]
pub struct Command;

impl Command {
    pub fn run(&self) -> Result<(), crate::CommandError> {
        let build_env = BuildEnv::get();
        println!(
            r#"
Build time:    {}
Commit SHA:    {}
Commit time:   {}
Commit branch: {}
Rustc channel: {}
Rustc version: {}
"#,
            build_env.time,
            build_env.git.commit_hash,
            build_env.git.commit_time,
            build_env.git.branch,
            build_env.rustc.channel,
            build_env.rustc.version,
        );
        Ok(())
    }
}
