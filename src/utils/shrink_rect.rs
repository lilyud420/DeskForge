use ratatui::layout::Rect;

pub fn shrink(area: Rect, x_offset: u16, y_offset: u16, width_sub: u16, height_sub: u16) -> Rect {
    Rect {
        x: area.x + x_offset,
        y: area.y + y_offset,
        width: area.width + width_sub,
        height: area.height.saturating_sub(height_sub),
    }
}
