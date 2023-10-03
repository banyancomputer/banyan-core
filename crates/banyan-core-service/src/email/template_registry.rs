use handlebars::Handlebars;
use serde::Serialize;

const TEMPLATE_EXT: &str = ".hbs";
const TEMPLATE_DIR: &str = "./src/email/templates";

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
    pub fn render<T>(
        &self,
        template_name: &str,
        data: &T,
    ) -> Result<String, handlebars::RenderError>
    where
        T: Serialize,
    {
        self.0.render(template_name, data)
    }
}
