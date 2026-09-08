//! Structured layout/encoding/writer characterization using the existing harness.
use super::{
    Document, Harness, Role, Text, Theme,
    document::{Alignment, Block, Column, ListItem, Severity},
    presentation::Run,
    spec,
};
use serde_json::json;
use unicode_width::UnicodeWidthStr;
fn t(s: &str) -> Text {
    Text::new(s).unwrap()
}
#[derive(Default)]
struct Writer {
    calls: usize,
    bytes: Vec<u8>,
}
impl std::io::Write for Writer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.calls += 1;
        self.bytes.extend(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn measure(h: &Harness) {
    let text = "Readable information with café, 界 and 👩‍💻. ".repeat(8);
    let cases = vec![
        ("paragraph", vec![Block::Paragraph(t(&text))]),
        (
            "facts",
            vec![Block::KeyValue(
                (0..16)
                    .map(|i| (t(&format!("Field {i}")), t("a longer value 界")))
                    .collect(),
            )],
        ),
        (
            "list",
            vec![Block::List {
                ordered: true,
                items: (0..32)
                    .map(|i| ListItem {
                        depth: (i % 3) as u8,
                        text: t("A descriptive entry 界"),
                    })
                    .collect(),
            }],
        ),
        (
            "table",
            vec![Block::Table {
                columns: vec![
                    Column {
                        heading: t("Name"),
                        alignment: Alignment::Left,
                    },
                    Column {
                        heading: t("Count"),
                        alignment: Alignment::Right,
                    },
                    Column {
                        heading: t("Details"),
                        alignment: Alignment::Left,
                    },
                ],
                rows: (0..32)
                    .map(|i| vec![t("café 界"), t(&i.to_string()), t("a longer explanation")])
                    .collect(),
            }],
        ),
        (
            "status",
            vec![Block::Status {
                severity: Severity::Warning,
                text: t("Operation unavailable: retry later"),
            }],
        ),
        ("large", vec![Block::Paragraph(t(&text)); 128]),
    ];
    for (name, blocks) in cases {
        let document = Document::new(blocks).unwrap();
        for columns in [8, 20, 80, 160] {
            let plain = Theme::new(true, true, None);
            let expected = document.render(columns, plain).unwrap();
            h.measure(
                spec("document", "layout", expected.len(), name, columns),
                || (),
                |_| document.layout(columns).unwrap(),
                |_, lines| {
                    assert!(lines.iter().all(|l| {
                        l.0.iter()
                            .map(|r| match r {
                                Run::Text(t) => t.width(),
                                _ => 0,
                            })
                            .sum::<usize>()
                            <= columns
                    }));
                    json!({"rendered_rows":lines.len(),"encoded_bytes":0,"writer_calls":0})
                },
            );
            for (label, theme) in [("plain", plain), ("styled", Theme::new(true, false, None))] {
                let reference = document.render(columns, theme).unwrap();
                h.measure(spec("document",label,expected.len(),name,columns),Writer::default,|writer|document.write_to(writer,columns,theme).unwrap(),|writer,_|{
                    assert_eq!(writer.bytes,reference.as_bytes());assert_eq!(writer.calls,1);
                    if label=="plain" {assert!(!writer.bytes.contains(&27));} else {assert!(!theme.sequence(Role::Strong).is_empty());}
                    json!({"encoded_bytes":writer.bytes.len(),"writer_calls":writer.calls,"scope":"layout + encoding + one logical writer batch; not syscall timing"})
                });
            }
        }
    }
}
