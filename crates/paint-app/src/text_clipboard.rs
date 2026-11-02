//! iced's browser clipboard is unavailable. Bridge asynchronous browser text
//! reads into the widget's synchronous Clipboard contract without image actions.
use crate::Msg;
use iced::Element;
pub fn wrap(content: Element<'_, Msg>) -> Element<'_, Msg> {
    #[cfg(target_arch = "wasm32")]
    {
        Element::new(web::TextClipboard { content })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        content
    }
}
#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use iced::advanced::clipboard::Kind;
    use iced::advanced::{
        layout::{self, Layout},
        overlay, renderer,
        widget::{Operation, Tree},
        Clipboard, Shell, Widget,
    };
    use iced::{mouse, Event, Length, Rectangle, Renderer, Size, Theme, Vector};
    thread_local! {
        static TEXT: std::cell::RefCell<Option<String>> = const {std::cell::RefCell::new(None)};
        static READING: std::cell::Cell<Option<u64>> = const {std::cell::Cell::new(None)};
        static EPOCH: std::cell::Cell<u64> = const {std::cell::Cell::new(0)};
        static READ_THIS_EVENT: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};
    }
    struct BrowserClipboard;
    impl Clipboard for BrowserClipboard {
        fn read(&self, kind: Kind) -> Option<String> {
            if kind != Kind::Standard {
                return None;
            }
            READ_THIS_EVENT.with(|c| c.set(true));
            if let Some(text) = TEXT.with(|c| c.borrow_mut().take()) {
                return Some(text);
            }
            let epoch = EPOCH.with(|c| c.get());
            if READING.with(|c| c.get()) != Some(epoch) {
                READING.with(|c| c.set(Some(epoch)));
                wasm_bindgen_futures::spawn_local(async move {
                    let result: Result<String, String> = async {
                        let clipboard = crate::clipboard::browser_clipboard()?;
                        wasm_bindgen_futures::JsFuture::from(clipboard.read_text())
                            .await
                            .map_err(|e| format!("{e:?}"))?
                            .as_string()
                            .ok_or("Geen klembordtekst".into())
                    }
                    .await;
                    if READING.with(|c| c.get()) == Some(epoch) {
                        READING.with(|c| c.set(None));
                    }
                    if EPOCH.with(|c| c.get()) != epoch {
                        return;
                    }
                    if let Ok(text) = result {
                        TEXT.with(|c| *c.borrow_mut() = Some(text));
                        // Deliver a widget paste only after its text is available.
                        if let Some(canvas) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.query_selector("canvas").ok().flatten())
                        {
                            let init = web_sys::KeyboardEventInit::new();
                            init.set_key("F24");
                            init.set_code("F24");
                            let (control, meta) = crate::web::keyboard_modifiers();
                            init.set_ctrl_key(control);
                            init.set_meta_key(meta);
                            init.set_bubbles(true);
                            init.set_cancelable(true);
                            if let Ok(event) =
                                web_sys::KeyboardEvent::new_with_keyboard_event_init_dict(
                                    "keydown", &init,
                                )
                            {
                                let _ = canvas.dispatch_event(&event);
                            }
                            let (control, meta) = crate::web::keyboard_modifiers();
                            init.set_ctrl_key(control);
                            init.set_meta_key(meta);
                            if let Ok(event) =
                                web_sys::KeyboardEvent::new_with_keyboard_event_init_dict(
                                    "keyup", &init,
                                )
                            {
                                let _ = canvas.dispatch_event(&event);
                            }
                        }
                    }
                });
            }
            None
        }
        fn write(&mut self, kind: Kind, contents: String) {
            if kind != Kind::Standard {
                return;
            }
            TEXT.with(|c| *c.borrow_mut() = None);
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(clipboard) = crate::clipboard::browser_clipboard() {
                    let _ =
                        wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&contents)).await;
                }
            });
        }
    }
    pub struct TextClipboard<'a> {
        pub content: Element<'a, Msg>,
    }
    impl Widget<Msg, Theme, Renderer> for TextClipboard<'_> {
        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.content)]
        }
        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(std::slice::from_ref(&self.content));
        }
        fn size(&self) -> Size<Length> {
            self.content.as_widget().size()
        }
        fn layout(
            &mut self,
            tree: &mut Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        }
        fn operate(
            &mut self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation,
        ) {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        }
        fn update(
            &mut self,
            tree: &mut Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            _clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Msg>,
            viewport: &Rectangle,
        ) {
            // An F24 wake-up carries no text and preserves the real modifier
            // state. Route it as Paste inside this widget, avoiding winit's
            // asynchronous modifier coalescing and accidental insertion of "v".
            let reply = matches!(
                event,
                Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::F24),
                    ..
                })
            ) && TEXT.with(|c| c.borrow().is_some());
            let mut routed = event.clone();
            // Wasm has no macOS target_os, so iced's command modifier defaults
            // to Ctrl. Normalize browser Cmd editing chords for text widgets.
            if let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) =
                &mut routed
            {
                if modifiers.logo()
                    && matches!(
                        key.as_ref(),
                        iced::keyboard::Key::Character(
                            "a" | "A" | "c" | "C" | "v" | "V" | "x" | "X" | "z" | "Z" | "y" | "Y"
                        )
                    )
                {
                    *modifiers |= iced::keyboard::Modifiers::CTRL;
                }
            }
            if reply {
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key,
                    modified_key,
                    modifiers,
                    text,
                    ..
                }) = &mut routed
                {
                    *key = iced::keyboard::Key::Character("v".into());
                    *modified_key = key.clone();
                    *modifiers = iced::keyboard::Modifiers::CTRL;
                    *text = None;
                }
            }
            let invalidate = !reply
                && match event {
                    Event::Mouse(mouse::Event::ButtonPressed(_))
                    | Event::Touch(_)
                    | Event::Window(iced::window::Event::Unfocused) => true,
                    Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key, modifiers, ..
                    }) => {
                        !((modifiers.control() || modifiers.logo())
                            && matches!(key.as_ref(), iced::keyboard::Key::Character("v" | "V")))
                    }
                    _ => false,
                };
            if invalidate {
                EPOCH.with(|c| c.set(c.get().wrapping_add(1)));
                TEXT.with(|c| *c.borrow_mut() = None);
            }
            READ_THIS_EVENT.with(|c| c.set(false));
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                &routed,
                layout,
                cursor,
                renderer,
                &mut BrowserClipboard,
                shell,
                viewport,
            );
            if READ_THIS_EVENT.with(|c| c.get()) {
                shell.capture_event();
            }
        }
        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                viewport,
                renderer,
            )
        }
        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'b>,
            renderer: &Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, Msg, Theme, Renderer>> {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }
}
