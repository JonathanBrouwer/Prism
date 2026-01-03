use annotate_snippets::Level;
use prism_input::span::Span;

pub struct Diag {
    level: Level<'static>,
    title: &'static str,
    id: &'static str,
    primary_span: Span,
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
        self.level.primary_title(self.title).id(self.id);

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
