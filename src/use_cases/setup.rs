use crate::domain::Environment;
use tracing::info;

pub struct SetupUseCase {
    env: Box<dyn Environment>,
}

impl SetupUseCase {
    pub fn new(env: Box<dyn Environment>) -> Self {
        Self { env }
    }

    pub fn execute(&self) -> anyhow::Result<()> {
        self.env.ensure_directories()?;
        self.env.ensure_config()?;
        info!("Setup complete.");
        Ok(())
    }
}
