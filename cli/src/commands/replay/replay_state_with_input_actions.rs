use node::BuildEnv;
use openmina_node_native::replay_state_with_input_actions;

#[derive(Debug, clap::Args)]
/// Replay node using initial state and input actions.
pub struct ReplayStateWithInputActions {
    #[arg(long, short, default_value = "~/.openmina/recorder")]
    pub dir: String,

    #[arg(long, default_value = "./target/release/libreplay_dynamic_effects.so")]
    pub dynamic_effects_lib: String,

    /// Verbosity level
    #[arg(long, short, env, default_value = "info")]
    pub verbosity: tracing::Level,
}

impl ReplayStateWithInputActions {
    pub fn run(self) -> anyhow::Result<()> {
        openmina_node_native::tracing::initialize(self.verbosity);

        let dir = shellexpand::full(&self.dir)?.into_owned();
        let dynamic_effects_lib = shellexpand::full(&self.dynamic_effects_lib)?.into_owned();

        let dynamic_effects_lib = match std::path::Path::new(&dynamic_effects_lib).exists() {
            true => Some(dynamic_effects_lib),
            false => {
                eprintln!("dynamic effects compiled lib not found! try running `cargo build --release -p replay_dynamic_effects`");
                None
            }
        };

        replay_state_with_input_actions(&dir, dynamic_effects_lib, check_build_env)?;

        Ok(())
    }
}

pub fn check_build_env(record_env: &BuildEnv, replay_env: &BuildEnv) -> anyhow::Result<()> {
    let is_git_same = record_env.git.commit_hash == replay_env.git.commit_hash;
    let is_cargo_same = record_env.cargo == replay_env.cargo;
    let is_rustc_same = record_env.rustc == replay_env.rustc;

    if !is_git_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.git, replay_env.git
        );
        let msg = format!("git build env mismatch!\n{diff}");
        if console::user_attended() {
            use dialoguer::Confirm;

            let prompt = format!("{msg}\nDo you want to continue?");
            if Confirm::new().with_prompt(prompt).interact().unwrap() {
            } else {
                anyhow::bail!("mismatch rejected");
            }
        } else {
            anyhow::bail!("mismatch rejected automatically");
        }
    }

    if !is_cargo_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.cargo, replay_env.cargo
        );
        let msg = format!("cargo build env mismatch!\n{diff}");
        eprintln!("{msg}");
    }

    if !is_rustc_same {
        let diff = format!(
            "recorded:\n{:?}\n\ncurrent:\n{:?}",
            record_env.rustc, replay_env.rustc
        );
        let msg = format!("rustc build env mismatch!\n{diff}");
        eprintln!("{msg}");
    }

    Ok(())
}
