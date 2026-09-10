//! Derived editor display cost, with no parser or terminal acquisition in timing.
use super::*;
fn presentation(e: &Engine, count: usize, hint: &str) -> AnalysisPresentation {
    let step = (e.editor.text().len() / count).max(1);
    let spans = (0..count)
        .map(|i| AnalysisSpan::new(i * step..(i * step + 1), Role::Accent).unwrap())
        .collect();
    AnalysisPresentation::new(
        e.editor.revision(),
        spans,
        Some(Hint::new(hint, Role::Dim).unwrap()),
    )
    .unwrap()
}
pub(super) fn measure(h: &Harness) {
    for (size, lines) in [
        (1024, 0),
        (65_536, 0),
        (1_048_576, 0),
        (690, 10),
        (6900, 100),
        (69000, 1000),
    ] {
        let text = if lines == 0 {
            "x".repeat(size)
        } else {
            ("x".repeat(68) + "\n").repeat(lines)
        };
        for count in [10, 100, 1000] {
            if count > size {
                continue;
            }
            for hint in ["short", "unicode", "long"] {
                let hint_text = match hint {
                    "unicode" => "e\u{301}界👩‍💻".to_owned(),
                    "long" => "x".repeat(4096),
                    _ => "suffix".to_owned(),
                };
                for operation in ["install", "render", "resize", "clear"] {
                    let mut case = spec("analysis_display", operation, size, hint, 80);
                    case["id"] = json!(format!("{}/spans-{count}", case["id"].as_str().unwrap()));
                    case["span_count"] = json!(count);
                    case["lines"] = json!(lines);
                    h.measure(
                        case,
                        || {
                            let mut e = Engine::new(editor(&text, text.len()));
                            e.start(prompt(), (80, 24)).unwrap();
                            let p = presentation(&e, count, &hint_text);
                            if operation != "install" {
                                e.present_analysis(p.clone()).unwrap();
                            }
                            (e, Some(p))
                        },
                        |(e, p)| match operation {
                            "install" => e.present_analysis(p.take().unwrap()).unwrap().1,
                            "render" => e.apply(Input::Request(Request::Redraw)).unwrap(),
                            "resize" => e.apply(Input::Resize(40, 24)).unwrap(),
                            _ => {
                                e.present_analysis(
                                    AnalysisPresentation::new(e.editor.revision(), vec![], None)
                                        .unwrap(),
                                )
                                .unwrap()
                                .1
                            }
                        },
                        |_, fx| mutation_counts(&fx.mutations),
                    );
                }
            }
        }
    }
    h.measure(spec("analysis_display","sizes",0,"none",0),||(),|_|(),|_,_|json!({"editor":std::mem::size_of::<Editor>(),"interaction":std::mem::size_of::<Interaction>(),"span":std::mem::size_of::<AnalysisSpan>(),"presentation":std::mem::size_of::<AnalysisPresentation>(),"hint":std::mem::size_of::<Hint>()}));
}
