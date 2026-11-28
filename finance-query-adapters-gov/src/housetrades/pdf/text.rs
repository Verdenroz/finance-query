//! Content-stream text extraction.
//!
//! Characters come from each font's `ToUnicode` CMap and positions from the
//! text matrix composed with the graphics matrix, so no glyph outlines are
//! needed. Reading order is recovered geometrically: spans are bucketed into
//! lines by device `y`, ordered by `x`, and separated by a space only where
//! the measured gap exceeds a fraction of the font size. For a fixed-column
//! table that beats inferring breaks from whitespace.

use std::collections::HashMap;

use super::document::Document;
use super::font::{self, Font};
use super::matrix::Matrix;
use super::object::{Dict, Lexer, Object};

const LINE_TOLERANCE: f64 = 2.5;
const SPACE_FRACTION: f64 = 0.18;
const MAX_FORM_DEPTH: usize = 8;
const MAX_FORMS: usize = 4096;

struct Span {
    x: f64,
    y: f64,
    end: f64,
    size: f64,
    text: String,
}

struct TextState {
    matrix: Matrix,
    line: Matrix,
    leading: f64,
    size: f64,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            matrix: Matrix::IDENTITY,
            line: Matrix::IDENTITY,
            leading: 0.0,
            size: 1.0,
        }
    }
}

impl TextState {
    fn set_matrix(&mut self, m: Matrix) {
        self.matrix = m;
        self.line = m;
    }

    fn next_line(&mut self, tx: f64, ty: f64) {
        self.line = Matrix::new(1.0, 0.0, 0.0, 1.0, tx, ty).then(self.line);
        self.matrix = self.line;
    }

    fn advance(&mut self, dx: f64) {
        self.matrix = Matrix::new(1.0, 0.0, 0.0, 1.0, dx, 0.0).then(self.matrix);
    }
}

fn fonts_for(doc: &Document, resources: Option<&Dict>) -> HashMap<String, Font> {
    let mut out = HashMap::new();
    let Some(fonts) = resources
        .and_then(|r| r.get("Font"))
        .and_then(|o| doc.dict_of(o))
    else {
        return out;
    };
    for (name, value) in fonts.entries() {
        if let Some(loaded) = font::load(doc, value) {
            out.insert(name.to_string(), loaded);
        }
    }
    out
}

struct Ctx<'a> {
    doc: &'a Document,
    spans: Vec<Span>,
    depth: usize,
    forms: usize,
}

fn show(ctx: &mut Ctx<'_>, state: &mut TextState, ctm: Matrix, font: Option<&Font>, bytes: &[u8]) {
    let Some(font) = font else { return };
    let full = state.matrix.then(ctm);
    let scale = full.x_scale();
    let width = font.advance(bytes, state.size);
    let text = font.decode(bytes);
    if !text.trim().is_empty() {
        ctx.spans.push(Span {
            x: full.e,
            y: full.f,
            end: full.e + width * scale,
            size: (state.size * scale).abs().max(f64::EPSILON),
            text,
        });
    }
    state.advance(width);
}

/// Form invocations are bounded on both axes: depth alone leaves a small file
/// able to fan out exponentially by re-invoking the same form at every level.
fn run_stream(ctx: &mut Ctx<'_>, resources: Option<&Dict>, data: &[u8], base: Matrix) {
    if ctx.depth > MAX_FORM_DEPTH || ctx.forms > MAX_FORMS {
        return;
    }
    let fonts = fonts_for(ctx.doc, resources);
    let mut state = TextState::default();
    let mut current: Option<&Font> = None;
    let mut operands: Vec<Object> = Vec::new();
    let mut ctm = base;
    let mut stack: Vec<Matrix> = Vec::new();
    let mut lex = Lexer::new(data, 0);

    loop {
        lex.skip_ws();
        let Some(b) = lex.peek_byte() else { break };
        if b.is_ascii_digit() || matches!(b, b'/' | b'(' | b'<' | b'[' | b'+' | b'-' | b'.') {
            if let Some(obj) = lex.object() {
                operands.push(obj);
            }
            continue;
        }
        let op = lex.operator();
        let n = operands.len();
        let num = |i: usize| operands.get(i).and_then(Object::as_f64).unwrap_or(0.0);

        match op {
            b"q" => stack.push(ctm),
            b"Q" => {
                if let Some(prev) = stack.pop() {
                    ctm = prev;
                }
            }
            b"cm" if n >= 6 => {
                ctm = Matrix::new(
                    num(n - 6),
                    num(n - 5),
                    num(n - 4),
                    num(n - 3),
                    num(n - 2),
                    num(n - 1),
                )
                .then(ctm);
            }
            b"Do" => {
                if let Some(name) = operands.last().and_then(Object::as_name) {
                    let name = name.to_string();
                    run_form(ctx, resources, &name, ctm);
                }
            }
            _ => apply(op, &operands, &mut state, &mut current, &fonts, ctm, ctx),
        }
        operands.clear();
    }
}

fn run_form(ctx: &mut Ctx<'_>, resources: Option<&Dict>, name: &str, ctm: Matrix) {
    let Some(id) = resources
        .and_then(|r| r.get("XObject"))
        .and_then(|o| ctx.doc.dict_of(o))
        .and_then(|x| x.get(name))
        .and_then(Object::as_ref_id)
    else {
        return;
    };
    let Some(dict) = ctx.doc.dicts.get(&id) else {
        return;
    };
    if dict.get("Subtype").and_then(Object::as_name) != Some("Form") {
        return;
    }
    let inner = matrix_of(dict).then(ctm);
    let own = dict
        .get("Resources")
        .and_then(|o| ctx.doc.dict_of(o))
        .or(resources);
    let Some(data) = ctx.doc.stream(id) else {
        return;
    };
    ctx.depth += 1;
    ctx.forms += 1;
    run_stream(ctx, own, &data, inner);
    ctx.depth -= 1;
}

fn matrix_of(dict: &Dict) -> Matrix {
    let Some(m) = dict.get("Matrix").and_then(Object::as_array) else {
        return Matrix::IDENTITY;
    };
    if m.len() < 6 {
        return Matrix::IDENTITY;
    }
    let v: Vec<f64> = m.iter().map(|o| o.as_f64().unwrap_or(0.0)).collect();
    Matrix::new(v[0], v[1], v[2], v[3], v[4], v[5])
}

fn apply<'a>(
    op: &[u8],
    operands: &[Object],
    state: &mut TextState,
    current: &mut Option<&'a Font>,
    fonts: &'a HashMap<String, Font>,
    ctm: Matrix,
    ctx: &mut Ctx<'_>,
) {
    let n = operands.len();
    let num = |i: usize| operands.get(i).and_then(Object::as_f64).unwrap_or(0.0);
    let last_two = || {
        if n >= 2 {
            (num(n - 2), num(n - 1))
        } else {
            (0.0, 0.0)
        }
    };

    match op {
        b"BT" => *state = TextState::default(),
        b"Tf" => {
            if let Some(name) = operands.first().and_then(Object::as_name) {
                *current = fonts.get(name);
            }
            if n >= 2 {
                state.size = num(n - 1);
            }
        }
        b"Tm" if n >= 6 => state.set_matrix(Matrix::new(
            num(n - 6),
            num(n - 5),
            num(n - 4),
            num(n - 3),
            num(n - 2),
            num(n - 1),
        )),
        b"Td" => {
            let (tx, ty) = last_two();
            state.next_line(tx, ty);
        }
        b"TD" => {
            let (tx, ty) = last_two();
            state.leading = -ty;
            state.next_line(tx, ty);
        }
        b"TL" => state.leading = num(0),
        b"T*" => {
            let leading = state.leading;
            state.next_line(0.0, -leading);
        }
        b"Tj" => {
            if let Some(bytes) = operands.first().and_then(Object::as_str_bytes) {
                show(ctx, state, ctm, *current, bytes);
            }
        }
        b"'" | b"\"" => {
            let leading = state.leading;
            state.next_line(0.0, -leading);
            if let Some(bytes) = operands.last().and_then(Object::as_str_bytes) {
                show(ctx, state, ctm, *current, bytes);
            }
        }
        b"TJ" => {
            let Some(items) = operands.first().and_then(Object::as_array) else {
                return;
            };
            for item in items {
                match item {
                    Object::Str(bytes) => show(ctx, state, ctm, *current, bytes),
                    other => {
                        if let Some(adjust) = other.as_f64() {
                            state.advance(-adjust / 1000.0 * state.size);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Every text line in the document, in reading order.
///
/// Lines are grouped per page: two pages routinely place different text at the
/// same `y`, so bucketing across the whole file would interleave them.
pub(super) fn extract_lines(doc: &Document) -> Vec<String> {
    let mut lines = Vec::new();
    for page in doc.page_contents() {
        let mut ctx = Ctx {
            doc,
            spans: Vec::new(),
            depth: 0,
            forms: 0,
        };
        for id in page.streams {
            let Some(data) = doc.stream(id) else { continue };
            run_stream(&mut ctx, page.resources, &data, Matrix::IDENTITY);
        }
        lines.extend(group_lines(ctx.spans));
    }
    lines
}

fn group_lines(mut spans: Vec<Span>) -> Vec<String> {
    spans.sort_by(|a, b| {
        b.y.partial_cmp(&a.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut lines: Vec<String> = Vec::new();
    let mut row: Vec<&Span> = Vec::new();
    let mut anchor = 0.0f64;

    for span in &spans {
        if row.is_empty() {
            anchor = span.y;
        } else if (anchor - span.y).abs() > LINE_TOLERANCE {
            lines.push(join_row(&mut row));
            anchor = span.y;
        }
        row.push(span);
    }
    if !row.is_empty() {
        lines.push(join_row(&mut row));
    }
    lines.retain(|l| !l.trim().is_empty());
    lines
}

fn join_row(row: &mut Vec<&Span>) -> String {
    row.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = String::new();
    let mut cursor = f64::NEG_INFINITY;
    for span in row.iter() {
        let gap = span.x - cursor;
        let needs_space = cursor.is_finite()
            && gap > span.size * SPACE_FRACTION
            && !out.ends_with(' ')
            && !span.text.starts_with(' ');
        if needs_space {
            out.push(' ');
        }
        out.push_str(&span.text);
        cursor = span.end;
    }
    row.clear();
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(x: f64, y: f64, width: f64, text: &str) -> Span {
        Span {
            x,
            y,
            end: x + width,
            size: 10.0,
            text: text.to_string(),
        }
    }

    #[test]
    fn groups_spans_into_lines_by_vertical_position() {
        let lines = group_lines(vec![
            span(100.0, 700.0, 30.0, "second"),
            span(10.0, 700.0, 30.0, "first"),
            span(10.0, 680.0, 40.0, "next line"),
        ]);
        assert_eq!(lines, vec!["first second", "next line"]);
    }

    #[test]
    fn orders_lines_top_to_bottom() {
        let lines = group_lines(vec![
            span(0.0, 100.0, 20.0, "bottom"),
            span(0.0, 500.0, 20.0, "top"),
        ]);
        assert_eq!(lines, vec!["top", "bottom"]);
    }

    #[test]
    fn tolerates_sub_point_baseline_jitter() {
        let lines = group_lines(vec![
            span(0.0, 700.0, 5.0, "a"),
            span(20.0, 698.7, 5.0, "b"),
        ]);
        assert_eq!(lines, vec!["a b"]);
    }

    #[test]
    fn abutting_glyphs_join_without_a_space() {
        let lines = group_lines(vec![
            span(0.0, 700.0, 6.0, "O"),
            span(6.0, 700.0, 6.0, "w"),
            span(12.0, 700.0, 5.0, "n"),
        ]);
        assert_eq!(lines, vec!["Own"]);
    }

    #[test]
    fn separated_columns_get_one_space() {
        let lines = group_lines(vec![
            span(0.0, 700.0, 20.0, "Asset"),
            span(120.0, 700.0, 10.0, "P"),
        ]);
        assert_eq!(lines, vec!["Asset P"]);
    }

    #[test]
    fn drops_blank_rows() {
        let lines = group_lines(vec![
            span(0.0, 700.0, 3.0, "   "),
            span(0.0, 680.0, 10.0, "real"),
        ]);
        assert_eq!(lines, vec!["real"]);
    }
}
