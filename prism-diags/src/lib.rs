use annotate_snippets::Level;

pub struct Diag {
    level: Level<'static>,
    title: String,
}

#[derive(Copy, Clone)]
pub enum RenderFormat {
    Fancy,
    Simple,
}

pub struct RenderConfig {
    format: RenderFormat,
}

impl Diag {
    fn render(&self, config: &RenderConfig) -> String {
        match config.format {
            RenderFormat::Fancy => {
                todo!()
            }
            RenderFormat::Simple => {
                todo!()
            }
        }
    }
}
