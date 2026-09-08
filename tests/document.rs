//! Structured output contracts through the public native API.
use replai::{
    Alignment, Block, Column, Document, EditError, Foreground, ListItem, Prompt, Role, Severity,
    Span, Style, Text, Theme,
};
use unicode_width::UnicodeWidthStr;
fn t(s: &str) -> Text {
    Text::new(s).unwrap()
}
fn plain() -> Theme {
    Theme::new(false, false, None)
}
fn doc(blocks: Vec<Block>) -> Document {
    Document::new(blocks).unwrap()
}
fn matrix() -> Document {
    doc(vec![Block::Table {
        columns: vec![
            Column {
                heading: t("Name"),
                alignment: Alignment::Left,
            },
            Column {
                heading: t("Count"),
                alignment: Alignment::Right,
            },
        ],
        rows: vec![vec![t("café 界"), t("12")], vec![t("joined 👩‍💻"), t("2\n3")]],
    }])
}
#[test]
fn aligned_and_stacked_tables_preserve_every_value() {
    let wide = matrix().render(30, plain()).unwrap();
    assert_eq!(
        wide,
        "Name       Count\n---------  -----\ncafé 界       12\njoined 👩‍💻      2\n               3\n"
    );
    let narrow = matrix().render(8, plain()).unwrap();
    assert_eq!(
        narrow,
        "Name\n  café \n  界\nCount\n  12\n\nName\n  joined\n   👩‍💻\nCount\n  2\n  3\n"
    );
    for width in 2..80 {
        let s = matrix().render(width, plain()).unwrap();
        assert!(s.lines().all(|l| l.width() <= width), "{width}: {s}");
    }
}
#[test]
fn plain_structure_roles_and_unicode_boundaries_are_independent() {
    let mixed = Text::from_spans(vec![
        Span::new(Role::Strong, "e").unwrap(),
        Span::new(Role::Error, "\u{301}界").unwrap(),
        Span::new(Role::Default, "!").unwrap(),
    ])
    .unwrap();
    let d = doc(vec![
        Block::Heading {
            level: 2,
            text: t("Details"),
        },
        Block::Paragraph(mixed),
        Block::Spacer,
        Block::Status {
            severity: Severity::Warning,
            text: t("Unavailable"),
        },
        Block::Literal(t("a\tb\r\n<safe>")),
    ]);
    let expected = "## Details\né界!\n\n[!] Unavailable\n| a   b\n  <safe>\n";
    for theme in [
        plain(),
        Theme::new(true, true, None),
        Theme::new(true, false, Some("dumb")),
    ] {
        assert_eq!(d.render(40, theme).unwrap(), expected);
    }
    let styled = d.render(40, Theme::new(true, false, None)).unwrap();
    let mut vt = vt100::Parser::new(20, 40, 0);
    vt.process(styled.replace('\n', "\r\n").as_bytes());
    assert!(vt.screen().cell(1, 0).unwrap().bold());
    assert!(!vt.screen().cell(1, 1).unwrap().bold());
    assert_eq!(
        vt.screen().cell(1, 1).unwrap().fgcolor(),
        vt100::Color::Idx(203)
    );
    assert_eq!(
        vt.screen().cell(1, 3).unwrap().fgcolor(),
        vt100::Color::Default
    );
    assert!(!styled.contains("\x1b[48"));
}
#[test]
fn lists_facts_and_status_keep_hierarchy_without_color() {
    let d = doc(vec![
        Block::KeyValue(vec![(t("x"), t("1")), (t("long"), t("value"))]),
        Block::List {
            ordered: true,
            items: vec![
                ListItem {
                    depth: 0,
                    text: t("first"),
                },
                ListItem {
                    depth: 1,
                    text: t("nested"),
                },
                ListItem {
                    depth: 0,
                    text: t("last"),
                },
            ],
        },
        Block::Status {
            severity: Severity::Success,
            text: t("ready"),
        },
        Block::Status {
            severity: Severity::Info,
            text: t("note"),
        },
        Block::Status {
            severity: Severity::Error,
            text: t("failed"),
        },
    ]);
    assert_eq!(
        d.render(40, plain()).unwrap(),
        "x     1\nlong  value\n1. first\n  1. nested\n2. last\n[ok] ready\n[i] note\n[error] failed\n"
    );
    for width in 2..40 {
        assert!(
            d.render(width, plain())
                .unwrap()
                .lines()
                .all(|l| l.width() <= width)
        );
    }
}
#[test]
fn safety_and_work_rejection_happen_before_output() {
    for bad in [
        "\x1b[2J",
        "\x1b]52;c;a\x07",
        "\x1bPbad\x1b\\",
        "\r",
        "\0",
        "\x7f",
        "\u{85}",
    ] {
        assert!(matches!(Text::new(bad), Err(EditError::InvalidText)));
    }
    assert!(matches!(
        Text::new(&"x".repeat(16385)),
        Err(EditError::Capacity)
    ));
    assert!(
        Document::new(vec![Block::Heading {
            level: 0,
            text: t("x")
        }])
        .is_err()
    );
    assert!(
        Document::new(vec![Block::List {
            ordered: false,
            items: vec![ListItem {
                depth: 9,
                text: t("x")
            }]
        }])
        .is_err()
    );
    assert!(
        Document::new(vec![Block::Table {
            columns: vec![],
            rows: vec![]
        }])
        .is_err()
    );
    assert!(
        Document::new(vec![Block::Table {
            columns: vec![Column {
                heading: t("x"),
                alignment: Alignment::Left
            }],
            rows: vec![vec![]]
        }])
        .is_err()
    );
    let huge = doc(vec![Block::Paragraph(t(&"x".repeat(16384))); 64]);
    let mut bytes = b"sentinel".to_vec();
    assert!(huge.write_to(&mut bytes, 2, plain()).is_err());
    assert_eq!(bytes, b"sentinel");
    for width in [0, 1, 4097, usize::MAX] {
        assert!(matrix().render(width, plain()).is_err());
    }
    struct Broken;
    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("broken"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(matches!(
        matrix().write_to(&mut Broken, 40, plain()),
        Err(replai::Error::Io(_))
    ));
}
#[test]
fn prompt_and_theme_are_safe_composable_values() {
    let prompt = Prompt::composed(
        Text::from_spans(vec![
            Span::new(Role::Strong, "db").unwrap(),
            Span::new(Role::Dim, "(prod)").unwrap(),
            Span::new(Role::Accent, "> ").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert!(prompt.clone().with_state("ignored").is_err());
    assert!(
        prompt
            .with_continuation_text(Text::styled(Role::Dim, ".. ").unwrap())
            .is_ok()
    );
    assert!(Prompt::composed(t("bad\n")).is_err());
    assert!(Prompt::composed(t(&"x".repeat(3073))).is_err());
    let theme = Theme::new(true, false, None)
        .with_style(
            Role::Accent,
            Style {
                foreground: Foreground::Green,
                bold: true,
            },
        )
        .unwrap();
    assert_eq!(theme.sequence(Role::Accent), "\x1b[1;38;5;114m");
    assert!(
        theme
            .with_style(
                Role::Default,
                Style {
                    foreground: Foreground::Red,
                    bold: true
                }
            )
            .is_err()
    );
    let plain = Theme::new(true, true, None)
        .with_style(Role::Accent, theme.style(Role::Accent))
        .unwrap();
    assert_eq!(plain.sequence(Role::Accent), "");
}
#[test]
fn batch_writer_and_empty_table_remain_meaningful() {
    struct Count(usize, Vec<u8>);
    impl std::io::Write for Count {
        fn write(&mut self, s: &[u8]) -> std::io::Result<usize> {
            self.0 += 1;
            self.1.extend(s);
            Ok(s.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count(0, vec![]);
    matrix().write_to(&mut count, 40, plain()).unwrap();
    assert_eq!(count.0, 1);
    let empty = doc(vec![Block::Table {
        columns: vec![Column {
            heading: t("Empty"),
            alignment: Alignment::Left,
        }],
        rows: vec![],
    }]);
    assert_eq!(empty.render(3, plain()).unwrap(), "Emp\nty\n");
}

#[test]
fn structure_bounds_include_empty_fields_and_split_joined_emoji() {
    let column = Column {
        heading: t("x"),
        alignment: Alignment::Left,
    };
    for (columns, rows) in [
        (vec![column.clone(); 33], vec![]),
        (vec![column.clone()], vec![vec![t("")]; 4097]),
    ] {
        assert!(matches!(
            Document::new(vec![Block::Table { columns, rows }]),
            Err(EditError::Capacity)
        ));
    }
    let empty = Text::from_spans(vec![]).unwrap();
    assert!(matches!(
        Document::new(vec![Block::Table {
            columns: vec![column; 32],
            rows: vec![vec![empty; 32]; 513]
        }]),
        Err(EditError::Capacity)
    ));
    assert!(matches!(
        Text::from_spans(vec![Span::new(Role::Dim, "").unwrap(); 1025]),
        Err(EditError::Capacity)
    ));
    let joined = Text::from_spans(vec![
        Span::new(Role::Accent, "👩").unwrap(),
        Span::new(Role::Error, "\u{200d}💻").unwrap(),
    ])
    .unwrap();
    assert_eq!(
        doc(vec![Block::Paragraph(joined)])
            .render(2, plain())
            .unwrap(),
        "👩‍💻\n"
    );
    assert_eq!(doc(vec![]).render(2, plain()).unwrap(), "");
}
