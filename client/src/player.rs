use ratatui::{
    prelude::{Buffer, Rect},
    style::Color,
    widgets::Widget,
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Player {
    pub x: u16,
    pub y: u16,
}

impl Widget for &Player {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x = self.x.min(area.width.saturating_sub(2));
        let y = self.y.min(area.height.saturating_sub(1));
        for dx in 0..2 {
            for dy in 0..1 {
                buf[(x.saturating_add(dx as u16), y.saturating_add(dy as u16))].set_bg(Color::Red);
            }
        }
    }
}
