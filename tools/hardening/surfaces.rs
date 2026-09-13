//! Bounded action/keymap/completion/suggestion composition oracle.
use crate::{Action, CompletionAction as C, CompletionCandidate, CompletionItem, CompletionSet, EditAction as E, Editor, Key, KeyMap, MatchCase, Prompt, Suggestion, SuggestionAction, complete_fuzzy, complete_prefix};

pub fn surface_case(data:&[u8]) {
    let mut map=KeyMap::new();
    for (index,chunk) in data.chunks(3).take(128).enumerate() {
        let byte=chunk.first().copied().unwrap_or(0)%32; let key=Key::Control(byte); let action=match chunk.get(1).copied().unwrap_or(0)%8 { 0=>Action::Edit(E::Left),1=>Action::Edit(E::Redo),2=>Action::Edit(E::KillWordForward),3=>Action::Completion(C::PageNext),4=>Action::Completion(C::First),5=>Action::Suggestion(SuggestionAction::Accept),6=>Action::HistorySearchDismiss,_=>Action::Redraw };
        match chunk.get(2).copied().unwrap_or(0)%3 { 0=>{let _=map.bind(key,action);},1=>{let _=map.unbind(key);},_=>{let _=map.reset(key);} }
        assert!(map.custom_len()<=crate::MAX_CUSTOM_BINDINGS); let _=map.get(Key::Control((index%32) as u8));
    }
    let mut engine=crate::engine::Engine::new(Editor::new(8192,8)); engine.start(Prompt::new("fuzz").unwrap(),(20+(data.first().copied().unwrap_or(0)%113) as usize,12)).unwrap();
    let alphabet=["a","b","界","e\u{301}"," "];
    for byte in data.iter().take(128) { let input=match byte%13 { 0..=4=>crate::actions::Input::Text(alphabet[(byte%5) as usize].into()),5=>crate::actions::Input::Edit(E::Left),6=>crate::actions::Input::Edit(E::Right),7=>crate::actions::Input::Edit(E::Undo),8=>crate::actions::Input::Edit(E::Redo),_=>crate::actions::Input::Resize(20+(*byte%113) as usize,12) }; let _=engine.apply(input); }
    let snapshot=engine.editor.analysis_snapshot(); let range=snapshot.cursor()..snapshot.cursor();
    let items=[CompletionItem::new("alpha").unwrap(),CompletionItem::new("alphabet").unwrap(),CompletionItem::new("界面").unwrap()];
    let prefix=complete_prefix(&snapshot,range.clone(),"a",&items,MatchCase::Sensitive).unwrap(); assert!(prefix.candidates().len()<=crate::MAX_COMPLETION_CANDIDATES);
    let fuzzy=complete_fuzzy(&snapshot,range.clone(),"aa",&items,MatchCase::Sensitive).unwrap(); assert!(fuzzy.candidates().len()<=items.len());
    let candidates=(0..usize::from(data.get(1).copied().unwrap_or(0)).min(64)).map(|i|CompletionCandidate::new(range.clone(),&format!("v{i}"),&format!("value {i}")).unwrap()).collect();
    engine.present_completions(CompletionSet::new(snapshot.revision(),candidates).unwrap()).unwrap();
    if engine.completion_selection().is_some() { for action in [C::Next,C::Previous,C::PageNext,C::PagePrevious,C::First,C::Last,C::Dismiss] { let _=engine.completion_action(action).unwrap(); if engine.completion_selection().is_none(){break;} } }
    if engine.editor.cursor()==engine.editor.text().len() { let revision=engine.editor.revision(); let suggestion=Suggestion::new(revision," suffix").unwrap(); engine.present_suggestion(suggestion).unwrap(); if data.get(2).copied().unwrap_or(0)&1==0 { let _=engine.suggestion_action(true); } else { let _=engine.suggestion_action(false); } }
    assert!(engine.editor.cursor()<=engine.editor.text().len()); assert!(engine.editor.text().is_char_boundary(engine.editor.cursor()));
}
