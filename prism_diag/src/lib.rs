pub mod sugg;

use annotate_snippets::level::ERROR;
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::{AnnotationKind, Group, Renderer, Snippet};
use prism_input::input_table::InputTableInner;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Diag {
    // pub level: Level<'static>,
    pub title: String,
    pub id: String,
    pub groups: Vec<AnnotationGroup>,
}

#[derive(Serialize, Deserialize)]
pub struct AnnotationGroup {
    pub annotations: Vec<Annotation>,
}

#[derive(Serialize, Deserialize)]
pub struct Annotation {
    pub span: Span,
    pub label: Option<String>,
}

#[derive(Copy, Clone, Default)]
pub enum RenderFormat {
    #[default]
    Styled,
    Plain,
}

#[derive(Default)]
pub struct RenderConfig {
    pub format: RenderFormat,
}

impl RenderConfig {
    pub fn uitest() -> Self {
        Self {
            format: RenderFormat::Plain,
        }
    }
}

impl Diag {
    pub fn render(&self, config: &RenderConfig, input: &InputTableInner) -> String {
        let mut diag: Group =
            Group::with_title(ERROR.clone().primary_title(&self.title).id(&self.id));

        for group in &self.groups {
            let file = group.annotations[0].span.start_pos().file();
            let mut snippet = Snippet::<annotate_snippets::Annotation>::source(input.get_str(file))
                .path(Some(input.get_path(file).to_string_lossy()));

            for anno in &group.annotations {
                snippet = snippet.annotation(
                    AnnotationKind::Primary
                        .span(
                            anno.span.start_pos().idx_in_file()..anno.span.end_pos().idx_in_file(),
                        )
                        .label(anno.label.clone()),
                )
            }

            diag = diag.element(snippet);
        }

        let renderer = match config.format {
            RenderFormat::Styled => Renderer::styled().decor_style(DecorStyle::Unicode),
            RenderFormat::Plain => Renderer::plain().decor_style(DecorStyle::Ascii),
        };
        renderer.render(&[diag])
    }
}

pub trait IntoDiag<Env> {
    #[must_use]
    fn into_diag(self, env: &mut Env) -> Diag;
}

impl<Env: Sized> IntoDiag<Env> for Diag {
    fn into_diag(self, _env: &mut Env) -> Diag {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let input_table = InputTable::default();
        let file = input_table
            .inner_mut()
            .get_or_push_file("Helpy helpy helpy".to_string(), "prism.rs".into());
        let span = Span::new(input_table.inner().start_of(file) + 6, 5);

        let diag = Diag {
            // level: Level::ERROR,
            title: "Something is badd",
            id: "baddy",
            groups: vec![AnnotationGroup {
                annotations: vec![Annotation {
                    span,
                    label: "This is wrong".to_string(),
                }],
            }],
        };

        eprintln!(
            "{}",
            diag.render(&RenderConfig::default(), &input_table.inner())
        )
    }
}
