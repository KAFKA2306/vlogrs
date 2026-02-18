use crate::domain::Environment;

pub struct SetupUseCase {
    env: Box<dyn Environment>,
}

impl SetupUseCase {
    pub fn new(env: Box<dyn Environment>) -> Self {
        Self { env }
    }

    pub fn execute(&self) {
        self.env.ensure_directories();
        self.env.ensure_config();
        println!("Setup complete.");
    }
}
