use std::fmt::{self, Debug};
use std::ops::Range;
use std::time::Duration;
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

use kas::draw::TextClass;
use kas::event::{self, ControlKey, GrabMode, PressSource, ScrollDelta};
use kas::geom::Vec2;
use kas::prelude::*;
use kas::text::SelectionHelper;

use std::sync;
use parking_lot::Mutex;

use crate::session::{Session, OutputMode};
use crate::builtin_types::*;
use ruffbox_synth::ruffbox::Ruffbox;

use crate::parser;
use crate::interpreter;

#[derive(Clone, Debug, PartialEq)]
enum LastEdit {
    None,
    Insert,
    Delete,
    Paste,
}

impl Default for LastEdit {
    fn default() -> Self {
        LastEdit::None
    }
}

enum EditAction {
    None,
    Unhandled,
    Activate,
    Edit,
}

const TOUCH_DUR: Duration = Duration::from_secs(1);

#[derive(Clone, Debug, PartialEq)]
enum TouchPhase {
    None,
    Start(u64, Coord), // id, coord
    Pan(u64),          // id
    Cursor(u64),       // id
}

impl Default for TouchPhase {
    fn default() -> Self {
        TouchPhase::None
    }
}

/// An editable, single-line text box.
///
/// This widget is intended for use with short input strings. Internally it
/// uses a [`String`], for which edits have `O(n)` cost.
///
/// Optionally, [`EditBox::multi_line`] mode can be activated (enabling
/// line-wrapping and a larger vertical height). This mode is only recommended
/// for short texts for performance reasons.
#[widget(config(key_nav = true, cursor_icon = event::CursorIcon::Text))]
#[handler(handle=noauto, generics = <>)]
#[derive(Clone, Widget)]
pub struct EditBox<const BUFSIZE:usize, const NCHAN:usize> {
    #[widget_core]
    core: CoreData,
    frame_offset: Coord,
    frame_size: Size,
    text_pos: Coord,
    view_offset: Coord,    
    text: Text<String>,
    required: Vec2,
    selection: SelectionHelper,
    edit_x_coord: Option<f32>,
    old_state: Option<(String, usize, usize)>,
    last_edit: LastEdit,
    error_state: bool,
    touch_phase: TouchPhase,
    // megra stuff
    ruffbox: sync::Arc<Mutex<Ruffbox<BUFSIZE,NCHAN>>>,
    session: sync::Arc<Mutex<Session<BUFSIZE,NCHAN>>>,
    sample_set: SampleSet,
    parts_store: PartsStore,
    mode: OutputMode
}

impl<const BUFSIZE:usize, const NCHAN:usize> Debug for EditBox<BUFSIZE, NCHAN> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "EditBox {{ core: {:?}, text: {:?}, ... }}",
            self.core,
            self.text.text()
        )
    }
}

impl <const BUFSIZE:usize, const NCHAN:usize> Layout for EditBox<BUFSIZE, NCHAN> {
    fn size_rules(&mut self, size_handle: &mut dyn SizeHandle, axis: AxisInfo) -> SizeRules {
        let frame_sides = size_handle.edit_surround();
        let inner = size_handle.inner_margin();
        let frame_offset = frame_sides.0 + inner;
        let frame_size = frame_offset + frame_sides.1 + inner;

        let margins = size_handle.outer_margins();
        let frame_rules = SizeRules::extract_fixed(axis.is_vertical(), frame_size, margins);

        let content_rules = size_handle.text_bound(&mut self.text, TextClass::EditMulti, axis);
        let m = content_rules.margins();

        // Note: we do not allocate space for the edit marker (size_handle.edit_marker_width());
        // instead we simply draw it in the margin (inner_margin() should be sufficient).

        let rules = content_rules.surrounded_by(frame_rules, true);
        if axis.is_horizontal() {
            self.core.rect.size.0 = rules.ideal_size();
            self.frame_offset.0 = frame_offset.0 as i32 + m.0 as i32;
            self.frame_size.0 = frame_size.0 + (m.0 + m.1) as u32;
        } else {
            self.core.rect.size.1 = rules.ideal_size();
            self.frame_offset.1 = frame_offset.1 as i32 + m.0 as i32;
            self.frame_size.1 = frame_size.1 + (m.0 + m.1) as u32;
        }
        rules
    }

    fn set_rect(&mut self, rect: Rect, align: AlignHints) {
        let rect = align
            .complete(Align::Stretch, Align::Stretch, self.rect().size)
            .apply(rect);

        self.core.rect = rect;
        self.text_pos = rect.pos + self.frame_offset;
        let size = rect.size - self.frame_size;
        self.required = self
            .text
            .update_env(|env| {
                env.set_bounds(size.into());
                env.set_wrap(true);
            })
            .into();
        self.set_view_offset_from_edit_pos();
    }

    fn draw(&self, draw_handle: &mut dyn DrawHandle, mgr: &event::ManagerState, disabled: bool) {
        
        let mut input_state = self.input_state(mgr, disabled);
        input_state.error = self.error_state;
        draw_handle.edit_box(self.core.rect, input_state);
        let bounds = self.text.env().bounds.into();
        if self.selection.is_empty() {
            draw_handle.text_offset(
                self.text_pos,
                bounds,
                self.view_offset,
                self.text.as_ref(),
                TextClass::EditMulti,
            );
        } else {
            // TODO(opt): we could cache the selection rectangles here to make
            // drawing more efficient (self.text.highlight_lines(range) output).
            // The same applies to the edit marker below.
            draw_handle.text_selected(
                self.text_pos,
                bounds,
                self.view_offset,
                &self.text,
                self.selection.range(),
		TextClass::EditMulti,
            );
        }
        if input_state.char_focus {
            draw_handle.edit_marker(
                self.text_pos,
                bounds,
                self.view_offset,
                self.text.as_ref(),
		TextClass::EditMulti,
                self.selection.edit_pos(),
            );
        }
    }
}

impl <const BUFSIZE:usize, const NCHAN:usize> EditBox<BUFSIZE, NCHAN> {
    /// Construct an `EditBox` with the given inital `text`.
    pub fn new<S: ToString>(text: S,
			    ruffbox: &sync::Arc<Mutex<Ruffbox<BUFSIZE,NCHAN>>>,
			    mode: OutputMode) -> Self {
        let text = text.to_string();
        let len = text.len();
        EditBox {
            core: Default::default(),
            frame_offset: Default::default(),
            frame_size: Default::default(),
            text_pos: Default::default(),
            view_offset: Default::default(),            
            text: Text::new(Default::default(), text.into()),
            required: Vec2::ZERO,
            selection: SelectionHelper::new(len, len),
            edit_x_coord: None,
            old_state: None,
            last_edit: LastEdit::None,
            error_state: false,
            touch_phase: TouchPhase::None,
	    ruffbox: sync::Arc::clone(ruffbox),
	    session: sync::Arc::new(Mutex::new(Session::<BUFSIZE, NCHAN>::with_mode(mode))),
	    sample_set: SampleSet::new(),
	    parts_store: PartsStore::new(),
	    mode: mode,
        }
    }    
}

impl<const BUFSIZE:usize, const NCHAN:usize> EditBox<BUFSIZE, NCHAN> {
        
    fn received_char(&mut self, mgr: &mut Manager, c: char) -> EditAction {

        let pos = self.selection.edit_pos();
        let selection = self.selection.range();
        let have_sel = selection.start < selection.end;
        if self.last_edit != LastEdit::Insert || have_sel {
            self.old_state = Some((self.text.clone_string(), pos, self.selection.sel_pos()));
            self.last_edit = LastEdit::Insert;
        }
        if have_sel {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            let _ = self.text.replace_range(selection.clone(), s);
            self.selection.set_pos(selection.start + s.len());
        } else {
            let _ = self.text.insert_char(pos, c);
            self.selection.set_pos(pos + c.len_utf8());
        }
        self.edit_x_coord = None;
        self.text.prepare();
        self.set_view_offset_from_edit_pos();
        mgr.redraw(self.id());
        EditAction::Edit
    }

    fn control_key(&mut self, mgr: &mut Manager, key: ControlKey) -> EditAction {

        let mut buf = [0u8; 4];
        let pos = self.selection.edit_pos();
        let selection = self.selection.range();
        let have_sel = selection.end > selection.start;
        let ctrl = mgr.modifiers().ctrl();
        let mut shift = mgr.modifiers().shift();
        let string;

        enum Action<'a> {
            None,
            Unhandled,
            Activate,
            Edit,
            Insert(&'a str, LastEdit),
            Delete(Range<usize>),
            Move(usize, Option<f32>),
        }
	
        let action = match key {
            ControlKey::Escape => {
                if !self.selection.is_empty() {
                    self.selection.set_empty();
                    mgr.redraw(self.id());
                    Action::None
                } else {
                    Action::Unhandled
                }
            }            
	    ControlKey::Return if ctrl => {
		// EVAALL
		let pfa_in = parser::eval_from_str(self.text.text(), &self.sample_set, &self.parts_store, self.mode);		
	
		match pfa_in {
		    Err(_) => {
			println!("can't parse");			
		    },
		    Ok(pfa) => {
			interpreter::interpret(&self.session, &mut self.sample_set, &mut self.parts_store, pfa, &self.ruffbox);
		    }
		}
				  
		Action::Activate
            }
	    ControlKey::Return => {
                Action::Insert('\n'.encode_utf8(&mut buf), LastEdit::Insert)
            }	    
            ControlKey::Tab => Action::Insert('\t'.encode_utf8(&mut buf), LastEdit::Insert),
            ControlKey::Home if ctrl => Action::Move(0, None),
            ControlKey::Home => {
                let pos = self.text.find_line(pos).map(|r| r.1.start).unwrap_or(0);
                Action::Move(pos, None)
            }
            ControlKey::End if ctrl => Action::Move(self.text.str_len(), None),
            ControlKey::End => {
                let pos = self
                    .text
                    .find_line(pos)
                    .map(|r| r.1.end)
                    .unwrap_or(self.text.str_len());
                Action::Move(pos, None)
            }
            ControlKey::Left if ctrl => {
                let mut iter = self.text.text()[0..pos].split_word_bound_indices();
                let mut p = iter.next_back().map(|(index, _)| index).unwrap_or(0);
                while self.text.text()[p..]
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace())
                    .unwrap_or(false)
                {
                    if let Some((index, _)) = iter.next_back() {
                        p = index;
                    } else {
                        break;
                    }
                }
                Action::Move(p, None)
            }
            ControlKey::Left => {
                let mut cursor = GraphemeCursor::new(pos, self.text.str_len(), true);
                cursor
                    .prev_boundary(self.text.text(), 0)
                    .unwrap()
                    .map(|pos| Action::Move(pos, None))
                    .unwrap_or(Action::None)
            }
            ControlKey::Right if ctrl => {
                let mut iter = self.text.text()[pos..].split_word_bound_indices().skip(1);
                let mut p = iter
                    .next()
                    .map(|(index, _)| pos + index)
                    .unwrap_or(self.text.str_len());
                while self.text.text()[p..]
                    .chars()
                    .next()
                    .map(|c| c.is_whitespace())
                    .unwrap_or(false)
                {
                    if let Some((index, _)) = iter.next() {
                        p = pos + index;
                    } else {
                        break;
                    }
                }
                Action::Move(p, None)
            }
            ControlKey::Right => {
                let mut cursor = GraphemeCursor::new(pos, self.text.str_len(), true);
                cursor
                    .next_boundary(self.text.text(), 0)
                    .unwrap()
                    .map(|pos| Action::Move(pos, None))
                    .unwrap_or(Action::None)
            }
            ControlKey::Up | ControlKey::Down => {
                let x = match self.edit_x_coord {
                    Some(x) => x,
                    None => self
                        .text
                        .text_glyph_pos(pos)
                        .next_back()
                        .map(|r| r.pos.0)
                        .unwrap_or(0.0),
                };
                let mut line = self.text.find_line(pos).map(|r| r.0).unwrap_or(0);
                // We can tolerate invalid line numbers here!
                line = match key {
                    ControlKey::Up => line.wrapping_sub(1),
                    ControlKey::Down => line.wrapping_add(1),
                    _ => unreachable!(),
                };
                const HALF: usize = usize::MAX / 2;
                let nearest_end = || match line {
                    0..=HALF => self.text.str_len(),
                    _ => 0,
                };
                self.text
                    .line_index_nearest(line, x)
                    .map(|pos| Action::Move(pos, Some(x)))
                    .unwrap_or(Action::Move(nearest_end(), None))
            }
            ControlKey::PageUp | ControlKey::PageDown => {
                let mut v = self
                    .text
                    .text_glyph_pos(pos)
                    .next_back()
                    .map(|r| r.pos.into())
                    .unwrap_or(Vec2::ZERO);
                if let Some(x) = self.edit_x_coord {
                    v.0 = x;
                }
                const FACTOR: f32 = 2.0 / 3.0;
                let mut h_dist = self.text.env().bounds.1 * FACTOR;
                if key == ControlKey::PageUp {
                    h_dist *= -1.0;
                }
                v.1 += h_dist;
                Action::Move(self.text.text_index_nearest(v.into()), Some(v.0))
            }
            ControlKey::Delete => {
                if have_sel {
                    Action::Delete(selection.clone())
                } else if ctrl {
                    let next = self.text.text()[pos..]
                        .split_word_bound_indices()
                        .skip(1)
                        .next()
                        .map(|(index, _)| pos + index)
                        .unwrap_or(self.text.str_len());
                    Action::Delete(pos..next)
                } else {
                    let mut cursor = GraphemeCursor::new(pos, self.text.str_len(), true);
                    cursor
                        .next_boundary(self.text.text(), 0)
                        .unwrap()
                        .map(|next| Action::Delete(pos..next))
                        .unwrap_or(Action::None)
                }
            }
            ControlKey::Backspace => {
                if have_sel {
                    Action::Delete(selection.clone())
                } else if ctrl {
                    let prev = self.text.text()[0..pos]
                        .split_word_bound_indices()
                        .next_back()
                        .map(|(index, _)| index)
                        .unwrap_or(0);
                    Action::Delete(prev..pos)
                } else {
                    // We always delete one code-point, not one grapheme cluster:
                    let prev = self.text.text()[0..pos]
                        .char_indices()
                        .rev()
                        .next()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    Action::Delete(prev..pos)
                }
            }
            ControlKey::Deselect => {
                self.selection.set_sel_pos(pos);
                mgr.redraw(self.id());
                Action::None
            }
            ControlKey::SelectAll => {
                self.selection.set_sel_pos(0);
                shift = true; // hack
                Action::Move(self.text.str_len(), None)
            }
            ControlKey::Cut if have_sel => {
                mgr.set_clipboard((self.text.text()[selection.clone()]).into());
                Action::Delete(selection.clone())
            }
            ControlKey::Copy if have_sel => {
                mgr.set_clipboard((self.text.text()[selection.clone()]).into());
                Action::None
            }
            ControlKey::Paste => {
                if let Some(content) = mgr.get_clipboard() {
                    let end = content.len();
                    
                    string = content;
                    Action::Insert(&string[0..end], LastEdit::Paste)
                } else {
                    Action::None
                }
            }
            ControlKey::Undo | ControlKey::Redo => {
                // TODO: maintain full edit history (externally?)
                // NOTE: undo *and* redo shortcuts map to this control char
                if let Some((state, pos2, sel_pos)) = self.old_state.as_mut() {
                    self.text.swap_string(state);
                    self.selection.set_edit_pos(*pos2);
                    *pos2 = pos;
                    let pos = *sel_pos;
                    *sel_pos = self.selection.sel_pos();
                    self.selection.set_sel_pos(pos);
                    self.edit_x_coord = None;
                    self.last_edit = LastEdit::None;
                }
                Action::Edit
            }
            _ => Action::Unhandled,
        };

        let result = match action {
            Action::None => EditAction::None,
            Action::Unhandled => EditAction::Unhandled,
            Action::Activate => EditAction::Activate,
            Action::Edit => EditAction::Edit,
            Action::Insert(s, edit) => {
                let mut pos = pos;
                if have_sel {
                    self.old_state =
                        Some((self.text.clone_string(), pos, self.selection.sel_pos()));
                    self.last_edit = edit;

                    self.text.replace_range(selection.clone(), s);
                    pos = selection.start;
                } else {
                    if self.last_edit != edit {
                        self.old_state =
                            Some((self.text.clone_string(), pos, self.selection.sel_pos()));
                        self.last_edit = edit;
                    }

                    self.text.replace_range(pos..pos, s);
                }
                self.selection.set_pos(pos + s.len());
                self.edit_x_coord = None;
                EditAction::Edit
            }
            Action::Delete(sel) => {
                if self.last_edit != LastEdit::Delete {
                    self.old_state =
                        Some((self.text.clone_string(), pos, self.selection.sel_pos()));
                    self.last_edit = LastEdit::Delete;
                }

                self.text.replace_range(sel.clone(), "");
                self.selection.set_pos(sel.start);
                self.edit_x_coord = None;
                EditAction::Edit
            }
            Action::Move(pos, x_coord) => {
                self.selection.set_edit_pos(pos);
                if !shift {
                    self.selection.set_empty();
                }
                self.edit_x_coord = x_coord;
                mgr.redraw(self.id());
                EditAction::None
            }
        };

        let mut set_offset = self.selection.edit_pos() != pos;
        if !self.text.required_action().is_ready() {
            self.text.prepare();
            set_offset = true;
            mgr.redraw(self.id());
        }
        if set_offset {
            self.set_view_offset_from_edit_pos();
        }

        result
    }

    fn set_edit_pos_from_coord(&mut self, mgr: &mut Manager, coord: Coord) {
        let rel_pos = (coord - self.text_pos + self.view_offset).into();
        self.selection
            .set_edit_pos(self.text.text_index_nearest(rel_pos));
        self.set_view_offset_from_edit_pos();
        self.edit_x_coord = None;
        mgr.redraw(self.id());
    }

    fn pan_delta(&mut self, mgr: &mut Manager, delta: Coord) -> bool {
        let bounds = Vec2::from(self.text.env().bounds);
        let max_offset = (self.required - bounds).ceil();
        let max_offset = Coord::from(max_offset).max(Coord::ZERO);
        let new_offset = (self.view_offset - delta).min(max_offset).max(Coord::ZERO);
        if new_offset != self.view_offset {
            self.view_offset = new_offset;
            mgr.redraw(self.id());
            true
        } else {
            false
        }
    }

    /// Update view_offset after edit_pos changes
    ///
    /// A redraw is assumed since edit_pos moved.
    fn set_view_offset_from_edit_pos(&mut self) {
        let edit_pos = self.selection.edit_pos();
        if let Some(marker) = self.text.text_glyph_pos(edit_pos).next_back() {
            let bounds = Vec2::from(self.text.env().bounds);
            let min_x = (marker.pos.0 - bounds.0).ceil();
            let min_y = (marker.pos.1 - marker.descent - bounds.1).ceil();
            let max_x = (marker.pos.0).floor();
            let max_y = (marker.pos.1 - marker.ascent).floor();
            let min = Coord(min_x as i32, min_y as i32);
            let max = Coord(max_x as i32, max_y as i32);

            let max_offset = (self.required - bounds).ceil();
            let max_offset = Coord::from(max_offset).max(Coord::ZERO);
            let max = max.min(max_offset);

            self.view_offset = self.view_offset.max(min).min(max);
        }
    }
}

impl <const BUFSIZE:usize, const NCHAN:usize>  event::Handler for EditBox<BUFSIZE, NCHAN> {
    type Msg = VoidMsg;

    fn handle(&mut self, mgr: &mut Manager, event: Event) -> Response<Self::Msg> {
        match event {
            Event::Activate => {
                mgr.request_char_focus(self.id());
                Response::None
            }
            Event::LostCharFocus => Response::None,
            Event::LostSelFocus => {
                self.selection.set_empty();
                mgr.redraw(self.id());
                Response::None
            }
            Event::Control(key) => match self.control_key(mgr, key) {
                EditAction::None => Response::None,
                EditAction::Unhandled => Response::Unhandled(Event::Control(key)),
                _ => Response::None,
            },
            Event::ReceivedCharacter(c) => match self.received_char(mgr, c) {
                EditAction::None => Response::None,
                EditAction::Unhandled => Response::Unhandled(Event::ReceivedCharacter(c)),
                _ => Response::None,
            },
            Event::PressStart { source, coord, .. } if source.is_primary() => {
                if let PressSource::Touch(touch_id) = source {
                    if self.touch_phase == TouchPhase::None {
                        self.touch_phase = TouchPhase::Start(touch_id, coord);
                        mgr.update_on_timer(TOUCH_DUR, self.id());
                    }
                } else if let PressSource::Mouse(_, repeats) = source {
                    if !mgr.modifiers().ctrl() {
                        // With Ctrl held, we scroll instead of moving the cursor
                        // (non-standard, but seems to work well)!
                        self.set_edit_pos_from_coord(mgr, coord);
                        if !mgr.modifiers().shift() {
                            self.selection.set_empty();
                        }
                        self.selection.set_anchor();
                        if repeats > 1 {
                            self.selection.expand(&self.text, repeats);
                        }
                    }
                }
                mgr.request_grab(self.id(), source, coord, GrabMode::Grab, None);
                mgr.request_char_focus(self.id());
                Response::None
            }
            Event::PressMove {
                source,
                coord,
                delta,
                ..
            } => {
                let ctrl = mgr.modifiers().ctrl();
                let mut sel_mode = 1;
                let pan = match source {
                    PressSource::Touch(touch_id) => match self.touch_phase {
                        TouchPhase::Start(id, ..) if id == touch_id => {
                            self.touch_phase = TouchPhase::Pan(id);
                            true
                        }
                        TouchPhase::Pan(id) if id == touch_id => true,
                        TouchPhase::Cursor(id) if id == touch_id => ctrl,
                        _ => false,
                    },
                    PressSource::Mouse(..) if ctrl => true,
                    PressSource::Mouse(_, repeats) => {
                        sel_mode = repeats;
                        false
                    }
                };
                if pan {
                    self.pan_delta(mgr, delta);
                } else {
                    self.set_edit_pos_from_coord(mgr, coord);
                    if sel_mode > 1 {
                        self.selection.expand(&self.text, sel_mode);
                    }
                }
                Response::None
            }
            Event::PressEnd { source, .. } => {
                match self.touch_phase {
                    TouchPhase::Start(id, coord) if source == PressSource::Touch(id) => {
                        if !mgr.modifiers().ctrl() {
                            self.set_edit_pos_from_coord(mgr, coord);
                            if !mgr.modifiers().shift() {
                                self.selection.set_empty();
                            }
                        }
                        self.touch_phase = TouchPhase::None;
                    }
                    TouchPhase::Pan(id) | TouchPhase::Cursor(id)
                        if source == PressSource::Touch(id) =>
                    {
                        self.touch_phase = TouchPhase::None;
                    }
                    _ => (),
                }
                Response::None
            }
            Event::Scroll(delta) => {
                let delta2 = match delta {
                    ScrollDelta::LineDelta(x, y) => {
                        // We arbitrarily scroll 3 lines:
                        let dist = 3.0 * self.text.env().height(Default::default());
                        let x = (x * dist).round() as i32;
                        let y = (y * dist).round() as i32;
                        Coord(x, y)
                    }
                    ScrollDelta::PixelDelta(coord) => coord,
                };
                if self.pan_delta(mgr, delta2) {
                    Response::None
                } else {
                    Response::Unhandled(Event::Scroll(delta))
                }
            }
            Event::TimerUpdate => {
                match self.touch_phase {
                    TouchPhase::Start(touch_id, coord) => {
                        if !mgr.modifiers().ctrl() {
                            self.set_edit_pos_from_coord(mgr, coord);
                            if !mgr.modifiers().shift() {
                                self.selection.set_empty();
                            }
                        }
                        self.touch_phase = TouchPhase::Cursor(touch_id);
                    }
                    _ => (),
                }
                Response::None
            }
            event => Response::Unhandled(event),
        }
    }
}
