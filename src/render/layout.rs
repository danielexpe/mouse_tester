use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    Wide,
    Normal,
    Compact,
    Minimal,
    TooSmall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Areas {
    pub header: Rect,
    pub canvas: Rect,
    pub side: Option<Rect>,
    pub lower: Option<Rect>,
    pub footer: Rect,
}

pub fn compute(area: Rect) -> (LayoutKind, Areas) {
    if area.width < 60 || area.height < 20 {
        return (
            LayoutKind::TooSmall,
            Areas {
                header: area,
                canvas: area,
                side: None,
                lower: None,
                footer: area,
            },
        );
    }

    let kind = if area.width >= 120 && area.height >= 36 {
        LayoutKind::Wide
    } else if area.width >= 100 && area.height >= 30 {
        LayoutKind::Normal
    } else if area.width >= 80 && area.height >= 24 {
        LayoutKind::Compact
    } else {
        LayoutKind::Minimal
    };

    let header_height = if matches!(kind, LayoutKind::Wide | LayoutKind::Normal) {
        3
    } else {
        2
    };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(area);

    let header = rows[0];
    let body = rows[1];
    let footer = rows[2];

    if matches!(kind, LayoutKind::Wide | LayoutKind::Normal) {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(72), Constraint::Percentage(28)])
            .split(body);
        (
            kind,
            Areas {
                header,
                canvas: columns[0],
                side: Some(columns[1]),
                lower: None,
                footer,
            },
        )
    } else if kind == LayoutKind::Compact {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(8)])
            .split(body);
        (
            kind,
            Areas {
                header,
                canvas: rows[0],
                side: None,
                lower: Some(rows[1]),
                footer,
            },
        )
    } else {
        (
            kind,
            Areas {
                header,
                canvas: body,
                side: None,
                lower: None,
                footer,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_breakpoints() {
        assert_eq!(compute(Rect::new(0, 0, 120, 36)).0, LayoutKind::Wide);
        assert_eq!(compute(Rect::new(0, 0, 100, 30)).0, LayoutKind::Normal);
        assert_eq!(compute(Rect::new(0, 0, 80, 24)).0, LayoutKind::Compact);
        assert_eq!(compute(Rect::new(0, 0, 60, 20)).0, LayoutKind::Minimal);
        assert_eq!(compute(Rect::new(0, 0, 59, 20)).0, LayoutKind::TooSmall);
    }

    #[test]
    fn wide_has_side_panel() {
        let (_, areas) = compute(Rect::new(0, 0, 120, 36));
        assert!(areas.side.is_some());
        assert!(areas.lower.is_none());
        assert!(areas.canvas.width > 0);
    }

    #[test]
    fn compact_has_lower_panel() {
        let (_, areas) = compute(Rect::new(0, 0, 80, 24));
        assert!(areas.side.is_none());
        assert!(areas.lower.is_some());
    }

    #[test]
    fn too_small_uses_full_area() {
        let (kind, areas) = compute(Rect::new(0, 0, 40, 10));
        assert_eq!(kind, LayoutKind::TooSmall);
        assert_eq!(areas.canvas, Rect::new(0, 0, 40, 10));
    }
}
