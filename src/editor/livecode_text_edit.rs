use std::sync::Arc;

use epaint::text::{cursor::*, Galley, LayoutJob};

use egui::{output::OutputEvent, *};

use egui::widgets::text_edit::{CCursorRange, CursorRange};

use parking_lot::Mutex;

/// The text edit state stored between frames.
#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct LivecodeTextEditState {
    cursor_range: Option<CursorRange>,

    /// This is what is easiest to work with when editing text,
    /// so users are more likely to read/write this.
    ccursor_range: Option<CCursorRange>,

    #[serde(skip)]
    pub flash_cursor_range: Option<CursorRange>,
    #[serde(skip)]
    pub flash_alpha: u8,
    #[serde(skip)]
    pub selection_toggle: bool,

    /// Wrapped in Arc for cheaper clones.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub undoer: Arc<Mutex<Undoer>>,

    // If IME candidate window is shown on this text edit.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub has_ime: bool,
}

/// The output from a `TextEdit`.
pub struct LivecodeTextEditOutput {
    /// The interaction response.
    pub response: egui::Response,

    /// How the text was displayed.
    pub galley: Arc<egui::Galley>,

    /// The state we stored after the run/
    pub state: LivecodeTextEditState,

    /// Where the text cursor is.
    pub cursor_range: Option<egui::widgets::text_edit::CursorRange>,
}

type Undoer = egui::util::undoer::Undoer<(CCursorRange, String)>;

impl LivecodeTextEditState {
        
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.memory().data.get_persisted(id)
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.memory().data.insert_persisted(id, self);
    }

    /// The the currently selected range of characters.
    pub fn ccursor_range(&self) -> Option<CCursorRange> {
        self.ccursor_range.or_else(|| {
            self.cursor_range
                .map(|cursor_range| cursor_range.as_ccursor_range())
        })
    }

    /// Sets the currently selected range of characters.
    pub fn set_ccursor_range(&mut self, ccursor_range: Option<CCursorRange>) {
        self.cursor_range = None;
        self.ccursor_range = ccursor_range;
    }

    pub fn set_cursor_range(&mut self, cursor_range: Option<CursorRange>) {
        self.cursor_range = cursor_range;
        self.ccursor_range = None;
    }

    pub fn cursor_range(&mut self, galley: &Galley) -> Option<CursorRange> {
        self.cursor_range
            .map(|cursor_range| {
                // We only use the PCursor (paragraph number, and character offset within that paragraph).
                // This is so that if we resize the `TextEdit` region, and text wrapping changes,
                // we keep the same byte character offset from the beginning of the text,
                // even though the number of rows changes
                // (each paragraph can be several rows, due to word wrapping).
                // The column (character offset) should be able to extend beyond the last word so that we can
                // go down and still end up on the same column when we return.
                CursorRange {
                    primary: galley.from_pcursor(cursor_range.primary.pcursor),
                    secondary: galley.from_pcursor(cursor_range.secondary.pcursor),
                }
            })
            .or_else(|| {
                self.ccursor_range.map(|ccursor_range| CursorRange {
                    primary: galley.from_ccursor(ccursor_range.primary),
                    secondary: galley.from_ccursor(ccursor_range.secondary),
                })
            })
    }
}

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct LivecodeTextEdit<'t> {
    text: &'t mut dyn TextBuffer,
    hint_text: WidgetText,
    id: Option<Id>,
    id_source: Option<Id>,
    text_style: Option<TextStyle>,
    text_color: Option<Color32>,
    layouter: Option<&'t mut dyn FnMut(&Ui, &str, f32) -> Arc<Galley>>,
    frame: bool,
    desired_width: Option<f32>,
    desired_height_rows: usize,
    lock_focus: bool,
    cursor_at_end: bool,
    eval_callback: Option<Arc<Mutex<dyn FnMut(&String)>>>,
}

impl<'t> WidgetWithState for LivecodeTextEdit<'t> {
    type State = LivecodeTextEditState;
}

impl<'t> LivecodeTextEdit<'t> {
    pub fn load_state(ctx: &Context, id: Id) -> Option<LivecodeTextEditState> {
        LivecodeTextEditState::load(ctx, id)
    }

    pub fn store_state(ctx: &Context, id: Id, state: LivecodeTextEditState) {
        state.store(ctx, id);
    }
}

impl<'t> LivecodeTextEdit<'t> {
    /// A `LivecodeTextEdit` for multiple lines. Pressing enter key will create a new line.
    pub fn multiline(text: &'t mut dyn TextBuffer) -> Self {
        Self {
            text,
            hint_text: Default::default(),
            id: None,
            id_source: None,
            text_style: None,
            text_color: None,
            layouter: None,
            frame: true,
            desired_width: None,
            desired_height_rows: 4,
            lock_focus: false,
            cursor_at_end: true,
            eval_callback: None,
        }
    }

    /// Build a `LivecodeTextEdit` focused on code editing.
    /// By default it comes with:
    /// - monospaced font
    /// - focus lock
    pub fn code_editor(self) -> Self {
        self.text_style(TextStyle::Monospace).lock_focus(true)
    }

    /// Use if you want to set an explicit `Id` for this widget.
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    pub fn eval_callback(mut self, callback: &Arc<Mutex<dyn FnMut(&String)>>) -> Self {
        self.eval_callback = Some(Arc::clone(callback));
        self
    }

    /// A source for the unique `Id`, e.g. `.id_source("second_text_edit_field")` or `.id_source(loop_index)`.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.id_source = Some(Id::new(id_source));
        self
    }

    /// Show a faint hint text when the text field is empty.
    pub fn hint_text(mut self, hint_text: impl Into<WidgetText>) -> Self {
        self.hint_text = hint_text.into();
        self
    }

    pub fn text_style(mut self, text_style: TextStyle) -> Self {
        self.text_style = Some(text_style);
        self
    }

    pub fn text_color(mut self, text_color: Color32) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn text_color_opt(mut self, text_color: Option<Color32>) -> Self {
        self.text_color = text_color;
        self
    }

    /// Override how text is being shown inside the `LivecodeTextEdit`.
    ///
    /// This can be used to implement things like syntax highlighting.
    ///
    /// This function will be called at least once per frame,
    /// so it is strongly suggested that you cache the results of any syntax highlighter
    /// so as not to waste CPU highlighting the same string every frame.
    ///
    /// The arguments is the enclosing [`Ui`] (so you can access e.g. [`Ui::fonts`]),
    /// the text and the wrap width.
    ///
    /// ```
    /// # egui::__run_test_ui(|ui| {
    /// # let mut my_code = String::new();
    /// # fn my_memoized_highlighter(s: &str) -> egui::text::LayoutJob { Default::default() }
    /// let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
    ///     let mut layout_job: egui::text::LayoutJob = my_memoized_highlighter(string);
    ///     layout_job.wrap_width = wrap_width;
    ///     ui.fonts().layout_job(layout_job)
    /// };
    /// ui.add(egui::LivecodeTextEdit::multiline(&mut my_code).layouter(&mut layouter));
    /// # });
    /// ```
    pub fn layouter(mut self, layouter: &'t mut dyn FnMut(&Ui, &str, f32) -> Arc<Galley>) -> Self {
        self.layouter = Some(layouter);

        self
    }

    /// Default is `true`. If set to `false` there will be no frame showing that this is editable text!
    pub fn frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }

    /// Set to 0.0 to keep as small as possible.
    /// Set to [`f32::INFINITY`] to take up all available space (i.e. disable automatic word wrap).
    pub fn desired_width(mut self, desired_width: f32) -> Self {
        self.desired_width = Some(desired_width);
        self
    }

    /// Set the number of rows to show by default.
    /// The default for singleline text is `1`.
    /// The default for multiline text is `4`.
    pub fn desired_rows(mut self, desired_height_rows: usize) -> Self {
        self.desired_height_rows = desired_height_rows;
        self
    }

    /// When `false` (default), pressing TAB will move focus
    /// to the next widget.
    ///
    /// When `true`, the widget will keep the focus and pressing TAB
    /// will insert the `'\t'` character.
    pub fn lock_focus(mut self, b: bool) -> Self {
        self.lock_focus = b;
        self
    }

    /// When `true` (default), the cursor will initially be placed at the end of the text.
    ///
    /// When `false`, the cursor will initially be placed at the beginning of the text.
    pub fn cursor_at_end(mut self, b: bool) -> Self {
        self.cursor_at_end = b;
        self
    }
}

// ----------------------------------------------------------------------------

impl<'t> Widget for LivecodeTextEdit<'t> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

impl<'t> LivecodeTextEdit<'t> {
    /// Show the [`LivecodeTextEdit`], returning a rich [`TextEditOutput`].
    pub fn show(self, ui: &mut Ui) -> LivecodeTextEditOutput {
        let is_mutable = self.text.is_mutable();
        let frame = self.frame;
        let where_to_put_background = ui.painter().add(Shape::Noop);

        let margin = Vec2::new(4.0, 2.0);
        let max_rect = ui.available_rect_before_wrap().shrink2(margin);
        let mut content_ui = ui.child_ui(max_rect, *ui.layout());
        let mut output = self.show_content(&mut content_ui);
        let id = output.response.id;
        let frame_rect = output.response.rect.expand2(margin);
        ui.allocate_space(frame_rect.size());

        output.response |= ui.interact(frame_rect, id, Sense::click());

        if output.response.clicked() && !output.response.lost_focus() {
            ui.memory().request_focus(output.response.id);
        }

        if frame {
            let visuals = ui.style().interact(&output.response);
            let frame_rect = frame_rect.expand(visuals.expansion);
            let shape = if is_mutable {
                if output.response.has_focus() {
                    epaint::RectShape {
                        rect: frame_rect,
                        corner_radius: visuals.corner_radius,
                        // fill: ui.visuals().selection.bg_fill,
                        fill: ui.visuals().extreme_bg_color,
                        stroke: ui.visuals().selection.stroke,
                    }
                } else {
                    epaint::RectShape {
                        rect: frame_rect,
                        corner_radius: visuals.corner_radius,
                        fill: ui.visuals().extreme_bg_color,
                        stroke: visuals.bg_stroke, // TODO: we want to show something here, or a text-edit field doesn't "pop".
                    }
                }
            } else {
                let visuals = &ui.style().visuals.widgets.inactive;
                epaint::RectShape {
                    rect: frame_rect,
                    corner_radius: visuals.corner_radius,
                    // fill: ui.visuals().extreme_bg_color,
                    // fill: visuals.bg_fill,
                    fill: Color32::TRANSPARENT,
                    stroke: visuals.bg_stroke, // TODO: we want to show something here, or a text-edit field doesn't "pop".
                }
            };

            ui.painter().set(where_to_put_background, shape);
        }

        output
    }

    fn show_content(self, ui: &mut Ui) -> LivecodeTextEditOutput {
        let LivecodeTextEdit {
            text,
            hint_text,
            id,
            id_source,
            text_style,
            text_color,
            layouter,
            frame: _,
            desired_width,
            desired_height_rows,
            lock_focus,
            cursor_at_end,
            eval_callback,
        } = self;

        let text_color = text_color
            .or(ui.visuals().override_text_color)
            // .unwrap_or_else(|| ui.style().interact(&response).text_color()); // too bright
            .unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());

        let prev_text = text.as_ref().to_owned();
        let text_style = text_style
            .or(ui.style().override_text_style)
            .unwrap_or_else(|| ui.style().body_text_style);
        let row_height = ui.fonts().row_height(text_style);
        const MIN_WIDTH: f32 = 24.0; // Never make a `LivecodeTextEdit` more narrow than this.
        let available_width = ui.available_width().at_least(MIN_WIDTH);
        let desired_width = desired_width.unwrap_or_else(|| ui.spacing().text_edit_width);
        let wrap_width = if ui.layout().horizontal_justify() {
            available_width
        } else {
            desired_width.min(available_width)
        };

        let mut default_layouter = move |ui: &Ui, text: &str, wrap_width: f32| {
            ui.fonts().layout_job(LayoutJob::simple(
                text.to_string(),
                text_style,
                text_color,
                wrap_width,
            ))
        };

        let layouter = layouter.unwrap_or(&mut default_layouter);

        let mut galley = layouter(ui, text.as_ref(), wrap_width);

        let desired_width = galley.size().x.max(wrap_width); // always show everything in multiline

        let desired_height = (desired_height_rows.at_least(1) as f32) * row_height;
        let desired_size = vec2(desired_width, galley.size().y.max(desired_height));

        let (auto_id, rect) = ui.allocate_space(desired_size);

        let id = id.unwrap_or_else(|| {
            if let Some(id_source) = id_source {
                ui.make_persistent_id(id_source)
            } else {
                auto_id // Since we are only storing the cursor a persistent Id is not super important
            }
        });
        let mut state = LivecodeTextEditState::load(ui.ctx(), id).unwrap_or_default();

        // On touch screens (e.g. mobile in egui_web), should
        // dragging select text, or scroll the enclosing `ScrollArea` (if any)?
        // Since currently copying selected text in not supported on `egui_web`,
        // we prioritize touch-scrolling:
        let allow_drag_to_select = !ui.input().any_touches() || ui.memory().has_focus(id);

        let sense = if allow_drag_to_select {
            Sense::click_and_drag()
        } else {
            Sense::click()
        };

        let mut response = ui.interact(rect, id, sense);
        let painter = ui.painter_at(rect);

        if let Some(pointer_pos) = ui.input().pointer.interact_pos() {
            if response.hovered() && text.is_mutable() {
                ui.output().mutable_text_under_cursor = true;
            }

            // TODO: triple-click to select whole paragraph
            // TODO: drag selected text to either move or clone (ctrl on windows, alt on mac)
            
            let cursor_at_pointer =
                galley.cursor_from_pos(pointer_pos - response.rect.min );

            if ui.visuals().text_cursor_preview
                && response.hovered()
                && ui.input().pointer.is_moving()
            {
                // preview:
                paint_cursor_end(
                    ui,
                    row_height,
                    &painter,
                    response.rect.min,
                    &galley,
                    &cursor_at_pointer,
                );
            }

            if response.double_clicked() {
                // Select word:
                let center = cursor_at_pointer;
                let ccursor_range = select_word_at(text.as_ref(), center.ccursor);
                state.set_cursor_range(Some(CursorRange {
                    primary: galley.from_ccursor(ccursor_range.primary),
                    secondary: galley.from_ccursor(ccursor_range.secondary),
                }));
            } else if allow_drag_to_select {
                if response.hovered() && ui.input().pointer.any_pressed() {
                    ui.memory().request_focus(id);
                    if ui.input().modifiers.shift {
                        if let Some(mut cursor_range) = state.cursor_range(&*galley) {
                            cursor_range.primary = cursor_at_pointer;
                            state.set_cursor_range(Some(cursor_range));
                        } else {
                            state.set_cursor_range(Some(CursorRange::one(cursor_at_pointer)));
                        }
                    } else {
                        state.set_cursor_range(Some(CursorRange::one(cursor_at_pointer)));
                    }
                } else if ui.input().pointer.any_down() && response.is_pointer_button_down_on() {
                    // drag to select text:
                    if let Some(mut cursor_range) = state.cursor_range(&*galley) {
                        cursor_range.primary = cursor_at_pointer;
                        state.set_cursor_range(Some(cursor_range));
                    }
                }
            }
        }

        if response.hovered() {
            ui.output().cursor_icon = CursorIcon::Text;
        }

        let mut cursor_range = None;
        let prev_cursor_range = state.cursor_range(&*galley);
        if ui.memory().has_focus(id) {
            //ui.memory().lock_focus(id, lock_focus);

            let default_cursor_range = if cursor_at_end {
                CursorRange::one(galley.end())
            } else {
                CursorRange::default()
            };

            let (changed, new_cursor_range) = livecode_events(
                ui,
                &mut state,
                text,
                &mut galley,
                layouter,
                wrap_width,
                default_cursor_range,
                eval_callback,
            );

            if changed {
                response.mark_changed();
            }
            cursor_range = Some(new_cursor_range);
        }

        let text_draw_pos = response.rect.min;

        if ui.is_rect_visible(rect) {
            painter.galley(text_draw_pos, galley.clone());

            if text.as_ref().is_empty() && !hint_text.is_empty() {
                let hint_text_color = ui.visuals().weak_text_color();
                let galley = hint_text.into_galley(ui, Some(true), desired_size.x, text_style);
                galley.paint_with_fallback_color(&painter, response.rect.min, hint_text_color);
            }

            if ui.memory().has_focus(id) {
                if let Some(cursor_range) = state.cursor_range(&*galley) {
                    // We paint the cursor on top of the text, in case
                    // the text galley has backgrounds (as e.g. `code` snippets in markup do).
                    paint_cursor_selection(
                        ui,
                        &painter,
                        text_draw_pos,
                        &galley,
                        &cursor_range,
                        None,
                    );

                    if let Some(cursorp) = state.flash_cursor_range {
                        if state.flash_alpha > 40 {
                            paint_cursor_selection(
                                ui,
                                &painter,
                                text_draw_pos,
                                &galley,
                                &cursorp,
                                Some(Color32::from_rgba_unmultiplied(
                                    220,
                                    80,
                                    20,
                                    state.flash_alpha,
                                )),
                            );
                            state.flash_alpha -= 40;
                        }
                    }

                    paint_cursor_end(
                        ui,
                        row_height,
                        &painter,
                        text_draw_pos,
                        &galley,
                        &cursor_range.primary,
                    );

                    if text.is_mutable() {
                        // egui_web uses `text_cursor_pos` when showing IME,
                        // so only set it when text is editable and visible!
                        ui.ctx().output().text_cursor_pos = Some(
                            galley
                                .pos_from_cursor(&cursor_range.primary)
                                .translate(response.rect.min.to_vec2())
                                .left_top(),
                        );
                    }
                }
            }
        }

        state.clone().store(ui.ctx(), id);

        let selection_changed = if let (Some(cursor_range), Some(prev_cursor_range)) =
            (cursor_range, prev_cursor_range)
        {
            prev_cursor_range.as_ccursor_range() != cursor_range.as_ccursor_range()
        } else {
            false
        };

        if response.changed() {
            response.widget_info(|| WidgetInfo::text_edit(prev_text.as_str(), text.as_str()));
        } else if selection_changed {
            let cursor_range = cursor_range.unwrap();
            let char_range =
                cursor_range.primary.ccursor.index..=cursor_range.secondary.ccursor.index;
            let info = WidgetInfo::text_selection_changed(char_range, text.as_str());
            response
                .ctx
                .output()
                .events
                .push(OutputEvent::TextSelectionChanged(info));
        } else {
            response.widget_info(|| WidgetInfo::text_edit(prev_text.as_str(), text.as_str()));
        }

        LivecodeTextEditOutput {
            response,
            galley,
            state,
            cursor_range,
        }
    }
}

// ----------------------------------------------------------------------------

/// Check for (keyboard) events to edit the cursor and/or text.
#[allow(clippy::too_many_arguments)]
fn livecode_events(
    ui: &mut egui::Ui,
    state: &mut LivecodeTextEditState,
    text: &mut dyn TextBuffer,
    galley: &mut Arc<Galley>,
    layouter: &mut dyn FnMut(&Ui, &str, f32) -> Arc<Galley>,
    wrap_width: f32,
    default_cursor_range: CursorRange,
    eval_callback: Option<Arc<Mutex<dyn FnMut(&String)>>>,
) -> (bool, CursorRange) {
    let mut cursor_range = state.cursor_range(&*galley).unwrap_or(default_cursor_range);

    // We feed state to the undoer both before and after handling input
    // so that the undoer creates automatic saves even when there are no events for a while.
    state.undoer.lock().feed_state(
        ui.input().time,
        &(cursor_range.as_ccursor_range(), text.as_ref().to_owned()),
    );

    let copy_if_not_password = |ui: &Ui, text: String| {
        ui.ctx().output().copied_text = text;
    };

    let mut any_change = false;

    for event in &ui.input().events {
        let did_mutate_text = match event {
            Event::Copy => {
		// clear selection
                state.selection_toggle = false;
		
                if cursor_range.is_empty() {
                    copy_if_not_password(ui, text.as_ref().to_owned());
                } else {
                    copy_if_not_password(ui, selected_str(text, &cursor_range).to_owned());
                }
                None
            }
            Event::Cut => {
		 // clear selection
                state.selection_toggle = false;
		
                if cursor_range.is_empty() {
                    copy_if_not_password(ui, text.take());
                    Some(CCursorRange::default())
                } else {
                    copy_if_not_password(ui, selected_str(text, &cursor_range).to_owned());
                    Some(CCursorRange::one(delete_selected(text, &cursor_range)))
                }
            }
            Event::Text(text_to_insert) => {

		 // clear selection
                state.selection_toggle = false;
		
                // Newlines are handled by `Key::Enter`.
                if !text_to_insert.is_empty()
		    && text_to_insert != "\n"
		    && text_to_insert != "\r"
		{			
		    if text_to_insert == "(" {
			 // enclose selection in parenthesis and
                        // jump to opening ...
                        let selection = selected_str(text, &cursor_range).clone().to_string();
                        let selection_len = selection.len();
                        let mut ccursor = delete_selected(text, &cursor_range);
                        insert_text(&mut ccursor, text, format!("({})", selection).as_str());
                        // clear selection
                        state.selection_toggle = false;
                        // go to opening paren so the function name can be entered ...
                        ccursor.index -= selection_len + 1;
                        Some(CCursorRange::one(ccursor))
		    } else if text_to_insert == "[" {
			 // enclose selection in parenthesis and
                        // jump to opening ...
                        let selection = selected_str(text, &cursor_range).clone().to_string();
                        let selection_len = selection.len();
                        let mut ccursor = delete_selected(text, &cursor_range);
                        insert_text(&mut ccursor, text, format!("[{}]", selection).as_str());
                        // clear selection
                        state.selection_toggle = false;
                        // go to opening paren so the function name can be entered ...
                        ccursor.index -= selection_len + 1;
                        Some(CCursorRange::one(ccursor))
		    } else if text_to_insert == "{" {
			 // enclose selection in parenthesis and
                        // jump to opening ...
                        let selection = selected_str(text, &cursor_range).clone().to_string();
                        let selection_len = selection.len();
                        let mut ccursor = delete_selected(text, &cursor_range);
                        insert_text(&mut ccursor, text, format!("{{{}}}", selection).as_str());
                        // clear selection
                        state.selection_toggle = false;
                        // go to opening paren so the function name can be entered ...
                        ccursor.index -= selection_len + 1;
                        Some(CCursorRange::one(ccursor))
		    } else if text_to_insert == "\"" {
			 // enclose selection in parenthesis and
                        // jump to opening ...
                        let selection = selected_str(text, &cursor_range).clone().to_string();
                        let selection_len = selection.len();
                        let mut ccursor = delete_selected(text, &cursor_range);
                        insert_text(&mut ccursor, text, format!("\"{}\"", selection).as_str());
                        // clear selection
                        state.selection_toggle = false;
                        // go to opening paren so the function name can be entered ...
                        ccursor.index -= selection_len + 1;
                        Some(CCursorRange::one(ccursor))
		    } else {
			let mut ccursor = delete_selected(text, &cursor_range);
			insert_text(&mut ccursor, text, text_to_insert);
			Some(CCursorRange::one(ccursor))
		    }
                } else {
                    None
                }
            }
            Event::Key {
                key: Key::Tab,
                pressed: true,
                ..
            } => {
                if let Some(sexp_cursors) = find_toplevel_sexp(text.as_str(), &cursor_range) {
                    let old_cursor = cursor_range.as_ccursor_range();
                    let cup = CursorRange {
                        primary: galley.from_ccursor(sexp_cursors.primary),
                        secondary: galley.from_ccursor(sexp_cursors.secondary),
                    };

                    let formatted = { format_sexp(selected_str(text, &cup)) };

                    let mut ccursor = delete_selected(text, &cup);
                    insert_text(&mut ccursor, text, &formatted);
                    Some(CCursorRange::one(old_cursor.primary))
                } else {
                    None
                }
            }
            Event::Key {
                key: Key::Enter,
                pressed: true,
                modifiers,
            } => {
                // clear selection
                state.selection_toggle = false;
                if modifiers.command {
                    if let Some(sexp_cursors) = find_toplevel_sexp(text.as_str(), &cursor_range) {
                        let cup = CursorRange {
                            primary: galley.from_ccursor(sexp_cursors.primary),
                            secondary: galley.from_ccursor(sexp_cursors.secondary),
                        };

                        // flash selected sexp ...
                        let sel = selected_str(text, &cup);
                        state.flash_cursor_range = Some(cup);
                        state.flash_alpha = 240; // set flash alpha ()
                        if let Some(cb) = eval_callback {
                            let mut cb_loc = cb.lock();
                            cb_loc(&sel.to_string());
                        } else {
                            println!("no callback!");
                        }
                    }
                    break; // need to break here because of callback move ...
                } else {
                    let mut ccursor = delete_selected(text, &cursor_range);
                    insert_text(&mut ccursor, text, "\n");
                    // TODO: if code editor, auto-indent by same leading tabs, + one if the lines end on an opening bracket
                    Some(CCursorRange::one(ccursor))
                }
            }
            Event::Key {
                key: Key::Z,
                pressed: true,
                modifiers,
            } if modifiers.command && !modifiers.shift => {
                // TODO: redo
                if let Some((undo_ccursor_range, undo_txt)) = state
                    .undoer
                    .lock()
                    .undo(&(cursor_range.as_ccursor_range(), text.as_ref().to_owned()))
                {
                    text.replace(undo_txt);
                    Some(*undo_ccursor_range)
                } else {
                    None
                }
            }
	    Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            } => {
                // clear selection
                state.selection_toggle = false;
                cursor_range.secondary = cursor_range.primary;
                //ui.memory().surrender_focus(id);
                break;
            }
	    Event::Key {
                key: Key::Space,
                pressed: true,
                modifiers,
            } => {
                if modifiers.command {
		    //println!("toggle {}",state.selection_toggle);
                    state.selection_toggle = !state.selection_toggle;
                }
                None
            }
            Event::Key {
                key,
                pressed: true,
                modifiers,
            } => on_key_press(&mut cursor_range, text, galley, *key, modifiers, state.selection_toggle),

            Event::CompositionStart => {
                state.has_ime = true;
                None
            }

            Event::CompositionUpdate(text_mark) => {
                if !text_mark.is_empty() && text_mark != "\n" && text_mark != "\r" && state.has_ime
                {
                    let mut ccursor = delete_selected(text, &cursor_range);
                    let start_cursor = ccursor;
                    insert_text(&mut ccursor, text, text_mark);
                    Some(CCursorRange::two(start_cursor, ccursor))
                } else {
                    None
                }
            }

            Event::CompositionEnd(prediction) => {
                if !prediction.is_empty()
                    && prediction != "\n"
                    && prediction != "\r"
                    && state.has_ime
                {
                    state.has_ime = false;
                    let mut ccursor = delete_selected(text, &cursor_range);
                    insert_text(&mut ccursor, text, prediction);
                    Some(CCursorRange::one(ccursor))
                } else {
                    None
                }
            }

            _ => None,
        };

        if let Some(new_ccursor_range) = did_mutate_text {
            any_change = true;

            // Layout again to avoid frame delay, and to keep `text` and `galley` in sync.
            *galley = layouter(ui, text.as_ref(), wrap_width);

            // Set cursor_range using new galley:
            cursor_range = CursorRange {
                primary: galley.from_ccursor(new_ccursor_range.primary),
                secondary: galley.from_ccursor(new_ccursor_range.secondary),
            };
        }
    }

    state.set_cursor_range(Some(cursor_range));

    state.undoer.lock().feed_state(
        ui.input().time,
        &(cursor_range.as_ccursor_range(), text.as_ref().to_owned()),
    );

    (any_change, cursor_range)
}

// ----------------------------------------------------------------------------

fn paint_cursor_selection(
    ui: &mut Ui,
    painter: &Painter,
    pos: Pos2,
    galley: &Galley,
    cursor_range: &CursorRange,
    color: Option<Color32>,
) {
    if cursor_range.is_empty() {
        return;
    }

    // We paint the cursor selection on top of the text, so make it transparent:
    let color = if let Some(col) = color {
        col
    } else {
        ui.visuals().selection.bg_fill.linear_multiply(0.5)
    };

    let [min, max] = cursor_range.sorted();
    let min = min.rcursor;
    let max = max.rcursor;

    for ri in min.row..=max.row {
        let row = &galley.rows[ri];
        let left = if ri == min.row {
            row.x_offset(min.column)
        } else {
            row.rect.left()
        };
        let right = if ri == max.row {
            row.x_offset(max.column)
        } else {
            let newline_size = if row.ends_with_newline {
                row.height() / 2.0 // visualize that we select the newline
            } else {
                0.0
            };
            row.rect.right() + newline_size
        };
        let rect = Rect::from_min_max(
            pos + vec2(left, row.min_y()),
            pos + vec2(right, row.max_y()),
        );
        painter.rect_filled(rect, 0.0, color);
    }
}

fn paint_cursor_end(
    ui: &mut Ui,
    row_height: f32,
    painter: &Painter,
    pos: Pos2,
    galley: &Galley,
    cursor: &Cursor,
) {
    let stroke = ui.visuals().selection.stroke;

    let mut cursor_pos = galley.pos_from_cursor(cursor).translate(pos.to_vec2());
    cursor_pos.max.y = cursor_pos.max.y.at_least(cursor_pos.min.y + row_height); // Handle completely empty galleys
    cursor_pos = cursor_pos.expand(1.5); // slightly above/below row

    let top = cursor_pos.center_top();
    let bottom = cursor_pos.center_bottom();

    painter.line_segment(
        [top, bottom],
        (ui.visuals().text_cursor_width, stroke.color),
    );

    if false {
        // Roof/floor:
        let extrusion = 3.0;
        let width = 1.0;
        painter.line_segment(
            [top - vec2(extrusion, 0.0), top + vec2(extrusion, 0.0)],
            (width, stroke.color),
        );
        painter.line_segment(
            [bottom - vec2(extrusion, 0.0), bottom + vec2(extrusion, 0.0)],
            (width, stroke.color),
        );
    }
}

// ----------------------------------------------------------------------------

fn selected_str<'s>(text: &'s dyn TextBuffer, cursor_range: &CursorRange) -> &'s str {
    let [min, max] = cursor_range.sorted();
    text.char_range(min.ccursor.index..max.ccursor.index)
}

fn insert_text(ccursor: &mut CCursor, text: &mut dyn TextBuffer, text_to_insert: &str) {
    ccursor.index += text.insert_text(text_to_insert, ccursor.index);
}

// ----------------------------------------------------------------------------

fn delete_selected(text: &mut dyn TextBuffer, cursor_range: &CursorRange) -> CCursor {
    let [min, max] = cursor_range.sorted();
    delete_selected_ccursor_range(text, [min.ccursor, max.ccursor])
}

fn delete_selected_ccursor_range(text: &mut dyn TextBuffer, [min, max]: [CCursor; 2]) -> CCursor {
    text.delete_char_range(min.index..max.index);
    CCursor {
        index: min.index,
        prefer_next_row: true,
    }
}

fn delete_previous_char(text: &mut dyn TextBuffer, ccursor: CCursor) -> CCursor {
    if ccursor.index > 0 {
        let max_ccursor = ccursor;
        let min_ccursor = max_ccursor - 1;
        delete_selected_ccursor_range(text, [min_ccursor, max_ccursor])
    } else {
        ccursor
    }
}

fn delete_next_char(text: &mut dyn TextBuffer, ccursor: CCursor) -> CCursor {
    delete_selected_ccursor_range(text, [ccursor, ccursor + 1])
}

fn delete_previous_word(text: &mut dyn TextBuffer, max_ccursor: CCursor) -> CCursor {
    let min_ccursor = ccursor_previous_word(text.as_ref(), max_ccursor);
    delete_selected_ccursor_range(text, [min_ccursor, max_ccursor])
}

fn delete_next_word(text: &mut dyn TextBuffer, min_ccursor: CCursor) -> CCursor {
    let max_ccursor = ccursor_next_word(text.as_ref(), min_ccursor);
    delete_selected_ccursor_range(text, [min_ccursor, max_ccursor])
}

fn delete_paragraph_before_cursor(
    text: &mut dyn TextBuffer,
    galley: &Galley,
    cursor_range: &CursorRange,
) -> CCursor {
    let [min, max] = cursor_range.sorted();
    let min = galley.from_pcursor(PCursor {
        paragraph: min.pcursor.paragraph,
        offset: 0,
        prefer_next_row: true,
    });
    if min.ccursor == max.ccursor {
        delete_previous_char(text, min.ccursor)
    } else {
        delete_selected(text, &CursorRange::two(min, max))
    }
}

fn delete_paragraph_after_cursor(
    text: &mut dyn TextBuffer,
    galley: &Galley,
    cursor_range: &CursorRange,
) -> CCursor {
    let [min, max] = cursor_range.sorted();
    let max = galley.from_pcursor(PCursor {
        paragraph: max.pcursor.paragraph,
        offset: usize::MAX, // end of paragraph
        prefer_next_row: false,
    });
    if min.ccursor == max.ccursor {
        delete_next_char(text, min.ccursor)
    } else {
        delete_selected(text, &CursorRange::two(min, max))
    }
}

// ----------------------------------------------------------------------------

/// Returns `Some(new_cursor)` if we did mutate `text`.
fn on_key_press(
    cursor_range: &mut CursorRange,
    text: &mut dyn TextBuffer,
    galley: &Galley,
    key: Key,
    modifiers: &Modifiers,
    selection_toggle: bool,
) -> Option<CCursorRange> {
    match key {
        Key::Backspace => {
            let ccursor = if modifiers.mac_cmd {
                delete_paragraph_before_cursor(text, galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    delete_previous_word(text, cursor.ccursor)
                } else if text.as_str().len() > 0 {
                    // this seems inefficient ...
                    if let Some(cur_char) = text.as_str().chars().nth(cursor.ccursor.index - 1) {
                        //println!("cur char {}", cur_char);
                        if let Some(next_char) = text.as_str().chars().nth(cursor.ccursor.index) {
                            //println!("next char {}", next_char);
                            if (cur_char == '(' && next_char == ')')
                                || (cur_char == '[' && next_char == ']')
				|| (cur_char == '{' && next_char == '}')
                                || (cur_char == '\"' && next_char == '\"')
                            {
                                let icur = delete_previous_char(text, cursor.ccursor);
                                delete_next_char(text, icur)
                            } else {
                                delete_previous_char(text, cursor.ccursor)
                            }
                        } else {
                            delete_previous_char(text, cursor.ccursor)
                        }
                    } else {
                        delete_previous_char(text, cursor.ccursor)
                    }
                } else {
                    CCursor::new(0)
                }
            } else {
                delete_selected(text, cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }
        Key::Delete if !(cfg!(target_os = "windows") && modifiers.shift) => {
            let ccursor = if modifiers.mac_cmd {
                delete_paragraph_after_cursor(text, galley, cursor_range)
            } else if let Some(cursor) = cursor_range.single() {
                if modifiers.alt || modifiers.ctrl {
                    // alt on mac, ctrl on windows
                    delete_next_word(text, cursor.ccursor)
                } else {
                    delete_next_char(text, cursor.ccursor)
                }
            } else {
                delete_selected(text, cursor_range)
            };
            let ccursor = CCursor {
                prefer_next_row: true,
                ..ccursor
            };
            Some(CCursorRange::one(ccursor))
        }

        Key::A if modifiers.command => {
            // select all
            *cursor_range = CursorRange::two(Cursor::default(), galley.end());
            None
        }

        Key::K if modifiers.ctrl => {
            let ccursor = delete_paragraph_after_cursor(text, galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::U if modifiers.ctrl => {
            let ccursor = delete_paragraph_before_cursor(text, galley, cursor_range);
            Some(CCursorRange::one(ccursor))
        }

        Key::W if modifiers.ctrl => {
            let ccursor = if let Some(cursor) = cursor_range.single() {
                delete_previous_word(text, cursor.ccursor)
            } else {
                delete_selected(text, cursor_range)
            };
            Some(CCursorRange::one(ccursor))
        }
	
        Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
            move_single_cursor(&mut cursor_range.primary, galley, key, modifiers);
            if !modifiers.shift && !selection_toggle {
                cursor_range.secondary = cursor_range.primary;
            }
            None
        }

        _ => None,
    }
}

fn move_single_cursor(cursor: &mut Cursor, galley: &Galley, key: Key, modifiers: &Modifiers) {
    match key {
        Key::ArrowLeft => {
            if modifiers.alt || modifiers.ctrl {
                // alt on mac, ctrl on windows
                *cursor = galley.from_ccursor(ccursor_previous_word(galley.text(), cursor.ccursor));
            } else if modifiers.mac_cmd {
                *cursor = galley.cursor_begin_of_row(cursor);
            } else {
                *cursor = galley.cursor_left_one_character(cursor);
            }
        }
        Key::ArrowRight => {
            if modifiers.alt || modifiers.ctrl {
                // alt on mac, ctrl on windows
                *cursor = galley.from_ccursor(ccursor_next_word(galley.text(), cursor.ccursor));
            } else if modifiers.mac_cmd {
                *cursor = galley.cursor_end_of_row(cursor);
            } else {
                *cursor = galley.cursor_right_one_character(cursor);
            }
        }
        Key::ArrowUp => {
            if modifiers.command {
                // mac and windows behavior
                *cursor = Cursor::default();
            } else {
                *cursor = galley.cursor_up_one_row(cursor);
            }
        }
        Key::ArrowDown => {
            if modifiers.command {
                // mac and windows behavior
                *cursor = galley.end();
            } else {
                *cursor = galley.cursor_down_one_row(cursor);
            }
        }

        Key::Home => {
            if modifiers.ctrl {
                // windows behavior
                *cursor = Cursor::default();
            } else {
                *cursor = galley.cursor_begin_of_row(cursor);
            }
        }
        Key::End => {
            if modifiers.ctrl {
                // windows behavior
                *cursor = galley.end();
            } else {
                *cursor = galley.cursor_end_of_row(cursor);
            }
        }

        _ => unreachable!(),
    }
}

// ----------------------------------------------------------------------------

fn select_word_at(text: &str, ccursor: CCursor) -> CCursorRange {
    if ccursor.index == 0 {
        CCursorRange::two(ccursor, ccursor_next_word(text, ccursor))
    } else {
        let it = text.chars();
        let mut it = it.skip(ccursor.index - 1);
        if let Some(char_before_cursor) = it.next() {
            if let Some(char_after_cursor) = it.next() {
                if is_word_char(char_before_cursor) && is_word_char(char_after_cursor) {
                    let min = ccursor_previous_word(text, ccursor + 1);
                    let max = ccursor_next_word(text, min);
                    CCursorRange::two(min, max)
                } else if is_word_char(char_before_cursor) {
                    let min = ccursor_previous_word(text, ccursor);
                    let max = ccursor_next_word(text, min);
                    CCursorRange::two(min, max)
                } else if is_word_char(char_after_cursor) {
                    let max = ccursor_next_word(text, ccursor);
                    CCursorRange::two(ccursor, max)
                } else {
                    let min = ccursor_previous_word(text, ccursor);
                    let max = ccursor_next_word(text, ccursor);
                    CCursorRange::two(min, max)
                }
            } else {
                let min = ccursor_previous_word(text, ccursor);
                CCursorRange::two(min, ccursor)
            }
        } else {
            let max = ccursor_next_word(text, ccursor);
            CCursorRange::two(ccursor, max)
        }
    }
}

fn ccursor_next_word(text: &str, ccursor: CCursor) -> CCursor {
    CCursor {
        index: next_word_boundary_char_index(text.chars(), ccursor.index),
        prefer_next_row: false,
    }
}

fn ccursor_previous_word(text: &str, ccursor: CCursor) -> CCursor {
    let num_chars = text.chars().count();
    CCursor {
        index: num_chars
            - next_word_boundary_char_index(text.chars().rev(), num_chars - ccursor.index),
        prefer_next_row: true,
    }
}

fn next_word_boundary_char_index(it: impl Iterator<Item = char>, mut index: usize) -> usize {
    let mut it = it.skip(index);
    if let Some(_first) = it.next() {
        index += 1;

        if let Some(second) = it.next() {
            index += 1;
            for next in it {
                if is_word_char(next) != is_word_char(second) {
                    break;
                }
                index += 1;
            }
        }
    }
    index
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Accepts and returns character offset (NOT byte offset!).
#[allow(dead_code)]
fn find_line_start(text: &str, current_index: CCursor) -> CCursor {
    // We know that new lines, '\n', are a single byte char, but we have to
    // work with char offsets because before the new line there may be any
    // number of multi byte chars.
    // We need to know the char index to be able to correctly set the cursor
    // later.
    let chars_count = text.chars().count();

    let position = text
        .chars()
        .rev()
        .skip(chars_count - current_index.index)
        .position(|x| x == '\n');

    match position {
        Some(pos) => CCursor::new(current_index.index - pos),
        None => CCursor::new(0),
    }
}
#[allow(dead_code)]
fn decrease_identation(ccursor: &mut CCursor, text: &mut dyn TextBuffer) {
    let line_start = find_line_start(text.as_ref(), *ccursor);

    let remove_len = if text.as_ref()[line_start.index..].starts_with('\t') {
        Some(1)
    } else if text.as_ref()[line_start.index..]
        .chars()
        .take(text::TAB_SIZE)
        .all(|c| c == ' ')
    {
        Some(text::TAB_SIZE)
    } else {
        None
    };

    if let Some(len) = remove_len {
        text.delete_char_range(line_start.index..(line_start.index + len));
        if *ccursor != line_start {
            *ccursor -= len;
        }
    }
}

// LIVECODE TEXT EDIT HELPERS
/// find toplevel s-expression from current cursor position ...
fn find_toplevel_sexp(text: &str, cursorp: &CursorRange) -> Option<CCursorRange> {
    let [min, _] = cursorp.sorted();

    let mut pos = min.ccursor.index;
    let mut rev_pos = text.len() - pos;

    let mut it_l = text.chars().rev();
    let mut it_r = text.chars();

    let mut l_pos = pos;
    let mut r_pos = pos;
    let mut last_closing = pos;
    let mut last_opening = pos;

    let mut sexp_found = false;

    // special case: if the cursor is right on an opening paren,
    // move one right ...
    if let Some(cur_chars) = text.get(pos..(pos + 1)) {
        if let Some(cur_char) = cur_chars.chars().next() {
            if cur_char == '(' {
                rev_pos = text.len() - (pos + 1);
                l_pos = pos + 1;
                r_pos = pos + 1;
                last_closing = pos + 1;
                last_opening = pos + 1;
                pos = pos + 1;
            }
        }
    }

    for _ in 0..rev_pos {
        it_l.next();
    }

    for _ in 0..pos {
        it_r.next();
    }

    let mut balance: i32 = 0;
    // beginning: lparen right after newline
    let mut lparen_found = false;
    while let Some(l_char) = it_l.next() {
        if l_char == '\n' && lparen_found {
            // two newlines - assume end
            break;
        } else if l_char == '(' {
            sexp_found = true;
            l_pos -= 1;
            last_opening = l_pos;
            balance += 1;
            lparen_found = true;
        } else if l_char == ')' {
            l_pos -= 1;
            balance -= 1;
            lparen_found = false;
        } else {
            l_pos -= 1;
            lparen_found = false;
        }
    }

    while let Some(r_char) = it_r.next() {
        if r_char == '(' {
            r_pos += 1;
            balance += 1;
        } else if r_char == ')' {
            r_pos += 1;
            last_closing = r_pos;
            balance -= 1;
        } else {
            r_pos += 1;
        }

        if balance == 0 {
            break;
        }
    }

    if balance == 0 && sexp_found {
        let left = CCursor {
            index: last_opening,
            prefer_next_row: true,
        };
        let right = CCursor {
            index: last_closing,
            prefer_next_row: false,
        };
        Some(CCursorRange::two(right, left))
    } else {
        None
    }
}

/// format an s-expression (content-agnostic)
fn format_sexp(input: &str) -> String {
    let mut lvl = 0;
    let mut out = "".to_string();
    let mut no_whitespace = false;

    for c in input.chars() {
        match c {
            '(' => {
                lvl += 1;
                out.push(c);
                no_whitespace = false;
            }
            ')' => {
                lvl -= 1;
                out.push(c);
                no_whitespace = false;
            }
            '\n' => {
                out.push(c);
                no_whitespace = true;
                for _ in 0..lvl {
                    out.push(' ');
                    out.push(' ');
                }
            }
            ' ' => {
                if !no_whitespace {
                    out.push(c);
                    no_whitespace = true;
                }
            }
            '\t' => {
                // ignore tabs
            }
            _ => {
                out.push(c);
                no_whitespace = false;
            }
        }
    }
    out
}

/// get the level of indentation needed up to the end of the
/// slice.
#[allow(dead_code)]
fn sexp_indent_level(input: &str) -> usize {
    let mut lvl = 0;
    for c in input.chars() {
        match c {
            '(' => {
                lvl += 1;
            }
            ')' => {
                lvl -= 1;
            }
            _ => {}
        }
    }
    lvl
}