use super::*;
use crate::theme::StyleClass;
use crate::*;
use parley::{PlainEditor, StyleProperty};
use smol_str::SmolStr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct TextBoxData<'text> {
    text: &'text mut String,
}

pub type TextBox<'text, 'style, 'classes> = Widget<'style, 'classes, TextBoxData<'text>>;

impl<'text, 'style, 'classes> TextBox<'text, 'style, 'classes> {
    pub const TYPE_CLASS: StyleClass = StyleClass::new_static("###text_box");

    #[track_caller]
    #[must_use]
    #[inline]
    pub fn new(text: &'text mut String) -> Self {
        TextBoxData { text }.into()
    }
}

impl WidgetData for TextBoxData<'_> {
    #[inline]
    fn type_class(&self) -> StyleClass {
        TextBox::TYPE_CLASS
    }
}

struct Editor {
    editor: PlainEditor<Color>,
    width: Option<f32>,
    font_size: Float<Pixel>,
    font_family: FontStack<'static>,
    font_style: FontStyle,
    font_weight: FontWeight,
    font_width: FontWidth,
    text_color: Color,
}

impl Deref for Editor {
    type Target = PlainEditor<Color>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.editor
    }
}

impl DerefMut for Editor {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.editor
    }
}

impl Editor {
    fn update_or_insert<'gui, Renderer: rendering::Renderer>(
        uid: Uid,
        gui: &'gui mut ByorGuiContext<'_, Renderer>,
    ) -> &'gui mut Self {
        let width = if gui.parent_style().text_wrap {
            let padding = gui.computed_parent_style().padding();
            gui.previous_state(uid).map(|state| {
                (state.size.x - padding.left - padding.right)
                    .value()
                    .max(0.0)
            })
        } else {
            None
        };

        let font_size = gui.computed_parent_style().font_size();
        let font_family = gui.parent_style().font_family.clone();
        let font_style = gui.parent_style().font_style;
        let font_weight = gui.parent_style().font_weight;
        let font_width = gui.parent_style().font_width;
        let text_color = gui.parent_style().text_color;

        let editor = gui
            .persistent_state_mut(uid)
            .get_or_insert_with(PersistentStateKey::TextBoxEditor, || {
                let mut editor = PlainEditor::<Color>::new(1.0);
                editor.set_alignment(parley::Alignment::Start);
                editor.set_quantize(true);
                editor.set_width(width);
                editor.set_scale(font_size.value());

                let styles = editor.edit_styles();
                styles.insert(StyleProperty::FontStack(font_family.clone()));
                styles.insert(StyleProperty::FontStyle(font_style));
                styles.insert(StyleProperty::FontWeight(font_weight));
                styles.insert(StyleProperty::FontWidth(font_width));
                styles.insert(StyleProperty::Brush(text_color));

                Editor {
                    editor,
                    width,
                    font_size,
                    font_family: font_family.clone(),
                    font_style,
                    font_weight,
                    font_width,
                    text_color,
                }
            })
            .expect("invalid editor type");

        if width != editor.width {
            editor.set_width(width);
            editor.width = width;
        }

        if font_size != editor.font_size {
            editor.set_scale(font_size.value());
            editor.font_size = font_size;
        }

        if font_family != editor.font_family {
            editor
                .edit_styles()
                .insert(StyleProperty::FontStack(font_family.clone()));
            editor.font_family = font_family;
        }

        if font_style != editor.font_style {
            editor
                .edit_styles()
                .insert(StyleProperty::FontStyle(font_style));
            editor.font_style = font_style;
        }

        if font_weight != editor.font_weight {
            editor
                .edit_styles()
                .insert(StyleProperty::FontWeight(font_weight));
            editor.font_weight = font_weight;
        }

        if font_width != editor.font_width {
            editor
                .edit_styles()
                .insert(StyleProperty::FontWidth(font_width));
            editor.font_width = font_width;
        }

        if text_color != editor.text_color {
            editor
                .edit_styles()
                .insert(StyleProperty::Brush(text_color));
            editor.text_color = text_color;
        }

        editor
    }
}

struct TextBoxRenderer<Renderer: rendering::Renderer> {
    _renderer: PhantomData<fn(Renderer)>,
}

impl<Renderer: rendering::Renderer> Default for TextBoxRenderer<Renderer> {
    #[inline]
    fn default() -> Self {
        Self {
            _renderer: PhantomData,
        }
    }
}

impl<Renderer: rendering::Renderer> rendering::NodeRenderer for TextBoxRenderer<Renderer> {
    type Renderer = Renderer;

    fn render(
        &self,
        context: rendering::RenderContext<'_, Self::Renderer>,
    ) -> Result<(), <Self::Renderer as rendering::Renderer>::Error> {
        if let Some(editor) = context
            .persistent_state
            .get::<Editor>(PersistentStateKey::TextBoxEditor)
        {
            let position = context.position
                + Vec2 {
                    x: context.style.padding().left,
                    y: context.style.padding().top,
                };

            for (selection, _) in editor.selection_geometry() {
                let min = Vec2 {
                    x: selection.x0.px(),
                    y: selection.y0.px(),
                };
                let max = Vec2 {
                    x: selection.x1.px(),
                    y: selection.y1.px(),
                };

                context.renderer.fill_rect(
                    position + min,
                    max - min,
                    0.px(),
                    // TODO: let the user pick this color
                    Color::rgb(66, 135, 245).into(),
                )?;
            }

            context.renderer.draw_text_layout(
                editor.try_layout().expect("layout was not updated"),
                position,
                &mut rendering::UnimplementedBoxRenderer::default(),
            )?;

            if let Some(cursor) =
                editor.cursor_geometry(1.pt().to_pixel(context.scale_factor).value())
                && context.input_state.focused
            {
                let min = Vec2 {
                    x: cursor.x0.px(),
                    y: cursor.y0.px(),
                };
                let max = Vec2 {
                    x: cursor.x1.px(),
                    y: cursor.y1.px(),
                };

                context.renderer.fill_rect(
                    position + min,
                    max - min,
                    0.px(),
                    context.style.text_color().into(),
                )?;
            }
        }

        Ok(())
    }
}

const CTRL_A: Shortcut = Shortcut {
    modifiers: Modifiers::CONTROL,
    key: Key::Character(SmolStr::new_inline("A")),
    location: None,
};

enum EditAction {
    Insert(SmolStr),
    Delete,
    DeleteWord,
    Backdelete,
    BackdeleteWord,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordLeft,
    MoveWordRight,
    SelectAll,
}

fn build_edit_action_list(input_state: &mut InputState) -> SmallVec<[EditAction; 2]> {
    let mut edit_actions = SmallVec::new();

    input_state.retain_key_events(|event| {
        match event.matches(&CTRL_A) {
            KeyEventMatch::True => {
                edit_actions.push(EditAction::SelectAll);
                return false;
            }
            KeyEventMatch::ConsumeOnly => return false,
            KeyEventMatch::False => (),
        }

        match event {
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::Delete),
                modifiers,
                ..
            } if modifiers.contains(Modifiers::CONTROL) => {
                edit_actions.push(EditAction::DeleteWord);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::Delete),
                ..
            } => {
                edit_actions.push(EditAction::Delete);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::Backspace),
                modifiers,
                ..
            } if modifiers.contains(Modifiers::CONTROL) => {
                edit_actions.push(EditAction::BackdeleteWord);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::Backspace),
                ..
            } => {
                edit_actions.push(EditAction::Backdelete);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowLeft),
                modifiers,
                ..
            } if modifiers.contains(Modifiers::CONTROL) => {
                edit_actions.push(EditAction::MoveWordLeft);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowLeft),
                ..
            } => {
                edit_actions.push(EditAction::MoveLeft);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowRight),
                modifiers,
                ..
            } if modifiers.contains(Modifiers::CONTROL) => {
                edit_actions.push(EditAction::MoveWordRight);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowRight),
                ..
            } => {
                edit_actions.push(EditAction::MoveRight);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowUp),
                ..
            } => {
                edit_actions.push(EditAction::MoveUp);
                return false;
            }
            KeyEvent::Pressed {
                key: Key::Named(NamedKey::ArrowDown),
                ..
            } => {
                edit_actions.push(EditAction::MoveDown);
                return false;
            }
            KeyEvent::Pressed {
                text: Some(text), ..
            } => {
                edit_actions.push(EditAction::Insert(text.clone()));
                return false;
            }
            _ => (),
        }

        true
    });

    edit_actions
}

impl<Renderer: rendering::Renderer> LeafWidgetData<Renderer> for TextBoxData<'_> {
    type ShowResult = ();

    fn show(
        self,
        gui: &mut ByorGuiContext<'_, Renderer>,
        uid: MaybeUid,
        style: Style,
    ) -> WidgetResult<Self::ShowResult> {
        let uid = uid.produce();

        let contents = NodeContents::default()
            .with_renderer(TextBoxRenderer::default())
            .with_builder(|mut gui| {
                let edit_actions = if gui.parent_input_state().focused {
                    build_edit_action_list(gui.global_input_state_mut())
                } else {
                    SmallVec::new()
                };

                let editor = Editor::update_or_insert(uid, &mut gui);

                if *self.text != editor.raw_text() {
                    editor.set_text(&self.text);
                }

                let mut text_changed = false;
                with_global_font_cache(|layout_context, font_context| {
                    let mut driver = editor.driver(font_context, layout_context);

                    for edit_action in &edit_actions {
                        match edit_action {
                            EditAction::Insert(text) => {
                                driver.insert_or_replace_selection(text);
                                text_changed = true;
                            }
                            EditAction::Delete => {
                                driver.delete();
                                text_changed = true;
                            }
                            EditAction::DeleteWord => {
                                driver.delete_word();
                                text_changed = true;
                            }
                            EditAction::Backdelete => {
                                driver.backdelete();
                                text_changed = true;
                            }
                            EditAction::BackdeleteWord => {
                                driver.backdelete_word();
                                text_changed = true;
                            }
                            EditAction::MoveLeft => driver.move_left(),
                            EditAction::MoveRight => driver.move_right(),
                            EditAction::MoveUp => driver.move_up(),
                            EditAction::MoveDown => driver.move_down(),
                            EditAction::MoveWordLeft => driver.move_word_left(),
                            EditAction::MoveWordRight => driver.move_word_right(),
                            EditAction::SelectAll => driver.select_all(),
                        }
                    }

                    driver.refresh_layout()
                });

                if text_changed {
                    self.text.clear();
                    self.text.push_str(editor.raw_text());
                }
            });

        gui.insert_node(Some(uid), &style, contents)?;

        Ok(())
    }
}
