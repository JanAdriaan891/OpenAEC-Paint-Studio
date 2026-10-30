use crate::{App, Msg};
use iced::{
    mouse,
    widget::canvas::{self, Frame, Geometry, Path, Stroke},
    Color, Point, Rectangle, Size,
};
#[derive(Debug, Clone, Copy)]
pub enum Input {
    Down(Point, bool),
    Move(Point),
    Up,
    Wheel(f32, Point, bool),
    Cancel,
}
#[derive(Default)]
pub struct State {
    pub middle: bool,
    pub left: bool,
    pub control: bool,
}
pub struct Board<'a> {
    pub app: &'a App,
}
impl canvas::Program<Msg> for Board<'_> {
    type State = State;
    fn update(
        &self,
        state: &mut State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Msg>> {
        let at = cursor
            .position()
            .map(|p| Point::new(p.x - bounds.x, p.y - bounds.y));
        let message = match event {
            iced::Event::Window(iced::window::Event::Unfocused) => {
                state.middle = false;
                state.left = false;
                return None;
            }
            iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m)) => {
                state.control = m.control() || m.logo();
                return None;
            }
            iced::Event::Mouse(mouse::Event::ButtonPressed(b))
                if cursor.is_over(bounds)
                    && matches!(b, mouse::Button::Left | mouse::Button::Middle) =>
            {
                state.middle = *b == mouse::Button::Middle;
                state.left = !state.middle;
                Input::Down(at?, state.middle)
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.left || state.middle => {
                Input::Move(at?)
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(b))
                if (*b == mouse::Button::Left && state.left)
                    || (*b == mouse::Button::Middle && state.middle) =>
            {
                state.left = false;
                state.middle = false;
                Input::Up
            }
            iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) if cursor.is_over(bounds) => {
                let amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 40.,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                Input::Wheel(amount, at?, state.control)
            }
            _ => return None,
        };
        Some(iced::widget::Action::publish(Msg::Canvas(message, bounds.size())).and_capture())
    }
    fn draw(
        &self,
        _: &State,
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let app = self.app;
        let look = app.look();
        let mut f = Frame::new(renderer, bounds.size());
        f.fill_rectangle(Point::ORIGIN, bounds.size(), look.color.workspace);
        let origin = app.origin(bounds.size());
        let scale = app.zoom / 100.;
        let size = Size::new(
            app.editor.document.width as f32 * scale,
            app.editor.document.height as f32 * scale,
        );
        // Checkerboard is drawn in screen space and clipped to the actual paper.
        let x0 = origin.x.max(0.);
        let y0 = origin.y.max(0.);
        let x1 = (origin.x + size.width).min(bounds.width);
        let y1 = (origin.y + size.height).min(bounds.height);
        let tile = 12.;
        let mut y = y0;
        while y < y1 {
            let mut x = x0;
            while x < x1 {
                let v = if (((x - origin.x) / tile).floor() as i32
                    + ((y - origin.y) / tile).floor() as i32)
                    % 2
                    == 0
                {
                    0.96
                } else {
                    0.86
                };
                f.fill_rectangle(
                    Point::new(x, y),
                    Size::new(tile.min(x1 - x), tile.min(y1 - y)),
                    Color::from_rgb(v, v, v),
                );
                x += tile;
            }
            y += tile;
        }
        f.draw_image(Rectangle::new(origin, size), &app.image);
        f.stroke_rectangle(
            origin,
            size,
            Stroke::default()
                .with_color(look.color.border)
                .with_width(1.),
        );
        if app.grid && scale >= 0.5 {
            let mut path = canvas::path::Builder::new();
            for x in (0..=app.editor.document.width).step_by(20) {
                let sx = origin.x + x as f32 * scale;
                if sx >= 0. && sx <= bounds.width {
                    path.move_to(Point::new(sx, y0));
                    path.line_to(Point::new(sx, y1));
                }
            }
            for y in (0..=app.editor.document.height).step_by(20) {
                let sy = origin.y + y as f32 * scale;
                if sy >= 0. && sy <= bounds.height {
                    path.move_to(Point::new(x0, sy));
                    path.line_to(Point::new(x1, sy));
                }
            }
            f.stroke(
                &path.build(),
                Stroke::default()
                    .with_color(Color {
                        a: 0.22,
                        ..look.color.text_secondary
                    })
                    .with_width(0.5),
            );
        }
        if let Some(b) = app.selection_bounds() {
            let p = Point::new(origin.x + b.x * scale, origin.y + b.y * scale);
            let s = Size::new(b.width * scale, b.height * scale);
            f.stroke(
                &Path::rectangle(p, s),
                Stroke::default()
                    .with_color(Color::from_rgb8(37, 99, 235))
                    .with_width(1.),
            );
            for dx in [0., s.width] {
                for dy in [0., s.height] {
                    f.fill_rectangle(
                        Point::new(p.x + dx - 3., p.y + dy - 3.),
                        Size::new(6., 6.),
                        Color::from_rgb8(37, 99, 235),
                    );
                }
            }
        }
        if let Some(b) = app.crop_bounds() {
            f.stroke_rectangle(
                Point::new(origin.x + b.x * scale, origin.y + b.y * scale),
                Size::new(b.width * scale, b.height * scale),
                Stroke::default()
                    .with_color(Color::from_rgb8(37, 99, 235))
                    .with_width(2.),
            );
        }
        vec![f.into_geometry()]
    }
    fn mouse_interaction(
        &self,
        state: &State,
        _: Rectangle,
        _: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.middle {
            mouse::Interaction::Grabbing
        } else if self.app.tool == paint_core::edit::Tool::Select {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::Crosshair
        }
    }
}
