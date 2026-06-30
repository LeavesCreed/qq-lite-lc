use qq_core::RichNode;

pub trait RichTextRenderer {
    type Output;

    fn render(nodes: &[RichNode]) -> Self::Output;
}

pub struct HtmlRenderer;
pub struct PlainTextRenderer;

impl RichTextRenderer for HtmlRenderer {
    type Output = String;

    fn render(nodes: &[RichNode]) -> Self::Output {
        nodes.iter().map(render_html_node).collect::<Vec<_>>().join("")
    }
}

impl RichTextRenderer for PlainTextRenderer {
    type Output = String;

    fn render(nodes: &[RichNode]) -> Self::Output {
        nodes.iter().map(render_plain_node).collect::<Vec<_>>().join("")
    }
}

pub fn plain_text_preview(nodes: &[RichNode]) -> String {
    PlainTextRenderer::render(nodes)
}

fn render_plain_node(node: &RichNode) -> String {
    match node {
        RichNode::Text { text } => text.clone(),
        RichNode::Unsupported { summary, .. } => summary.clone(),
    }
}

fn render_html_node(node: &RichNode) -> String {
    match node {
        RichNode::Text { text } => escape_html(text),
        RichNode::Unsupported { kind, summary } => {
            format!(
                r#"<span class="message-unsupported" data-kind="{}">{}</span>"#,
                escape_html(kind),
                escape_html(summary)
            )
        }
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_text() {
        let nodes = [RichNode::Text { text: "<hello>".to_owned() }];
        assert_eq!(HtmlRenderer::render(&nodes), "&lt;hello&gt;");
    }
}
