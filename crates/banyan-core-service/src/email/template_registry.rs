use handlebars::Handlebars;
use serde::Serialize;

use super::error::EmailError;

const TEMPLATE_EXT: &str = ".hbs";
const TEMPLATE_DIR: &str = "./templates/email";

pub struct TemplateRegistry(Handlebars<'static>);

impl Default for TemplateRegistry {
    fn default() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_templates_directory(TEMPLATE_EXT, TEMPLATE_DIR)
            .expect("Failed to register templates directory");
        TemplateRegistry(handlebars)
    }
}

impl TemplateRegistry {
    pub fn render<T>(&self, template_name: &str, data: &T) -> Result<String, EmailError>
    where
        T: Serialize,
    {
        self.0
            .render(template_name, data)
            .map_err(EmailError::render_error)
    }
}
