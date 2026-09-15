pub const CHROME_HEIGHT_DEFAULT: f64 = 128.0;
pub const CHROME_HEIGHT_MAX: f64 = 168.0;
pub const PANE_GAP: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct LayoutPlan {
    pub chrome: Rect,
    pub panes: Vec<(String, Rect)>,
}

pub fn compute_layout(
    window_width: f64,
    window_height: f64,
    chrome_height: f64,
    enabled_ids_in_order: &[String],
) -> LayoutPlan {
    let chrome_h = chrome_height.clamp(0.0, CHROME_HEIGHT_MAX);
    let chrome = Rect {
        x: 0.0,
        y: (window_height - chrome_h).max(0.0),
        width: window_width.max(0.0),
        height: chrome_h,
    };
    let pane_region_h = (window_height - chrome_h).max(0.0);
    let n = enabled_ids_in_order.len();
    if n == 0 {
        return LayoutPlan {
            chrome,
            panes: vec![],
        };
    }

    let row_counts: Vec<usize> = row_counts(n);

    let row_count = row_counts.len();
    let total_gap_y = PANE_GAP * (row_count.saturating_sub(1) as f64);
    let row_h = if row_count == 0 {
        0.0
    } else {
        (pane_region_h - total_gap_y) / row_count as f64
    };

    let mut panes = Vec::with_capacity(n);
    let mut idx = 0usize;
    let mut y = 0.0;
    for cols in row_counts {
        let total_gap_x = PANE_GAP * (cols.saturating_sub(1) as f64);
        let col_w = if cols == 0 {
            0.0
        } else {
            (window_width - total_gap_x) / cols as f64
        };
        for c in 0..cols {
            let id = enabled_ids_in_order[idx].clone();
            panes.push((
                id,
                Rect {
                    x: c as f64 * (col_w + PANE_GAP),
                    y,
                    width: col_w,
                    height: row_h,
                },
            ));
            idx += 1;
        }
        y += row_h + PANE_GAP;
    }

    LayoutPlan { chrome, panes }
}

/// Layout A for n≤7, then even rows of at most 5 columns (15 → 5+5+5).
fn row_counts(n: usize) -> Vec<usize> {
    if n == 0 {
        vec![]
    } else if n <= 4 {
        vec![n]
    } else if n == 5 {
        vec![3, 2]
    } else {
        let rows = n.div_ceil(5);
        let base = n / rows;
        let extra = n % rows;
        (0..rows)
            .map(|i| if i < extra { base + 1 } else { base })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_sits_at_bottom_with_default_height() {
        let plan = compute_layout(1000.0, 800.0, CHROME_HEIGHT_DEFAULT, &[]);
        assert_eq!(plan.chrome.y, 800.0 - CHROME_HEIGHT_DEFAULT);
        assert_eq!(plan.chrome.height, CHROME_HEIGHT_DEFAULT);
        assert_eq!(plan.chrome.width, 1000.0);
        assert!(plan.panes.is_empty());
    }

    #[test]
    fn seven_panes_use_four_plus_three() {
        let ids: Vec<String> = ["chatgpt", "claude", "gemini", "copilot", "zai", "deepseek", "grok"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let plan = compute_layout(1002.0, 800.0, 96.0, &ids);
        assert_eq!(plan.panes.len(), 7);
        // top row y=0, bottom row y = pane_h + gap
        let pane_region_h = 800.0 - 96.0;
        let row_h = (pane_region_h - PANE_GAP) / 2.0;
        assert!((plan.panes[0].1.y - 0.0).abs() < 0.01);
        assert!((plan.panes[4].1.y - (row_h + PANE_GAP)).abs() < 0.01);
        assert_eq!(plan.panes[0].1.width, plan.panes[1].1.width);
        // bottom row three equal columns
        assert_eq!(plan.panes[4].1.width, plan.panes[5].1.width);
        assert_eq!(plan.panes[5].1.width, plan.panes[6].1.width);
    }

    #[test]
    fn five_panes_split_three_plus_two() {
        let ids: Vec<String> = ["chatgpt", "claude", "gemini", "copilot", "zai"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let plan = compute_layout(1000.0, 800.0, 96.0, &ids);
        assert_eq!(plan.panes.len(), 5);
        let top: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y < 1.0).collect();
        let bottom: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y >= 1.0).collect();
        assert_eq!(top.len(), 3);
        assert_eq!(bottom.len(), 2);
    }

    #[test]
    fn four_or_fewer_single_row() {
        let ids: Vec<String> = ["chatgpt", "claude"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let plan = compute_layout(1000.0, 800.0, 96.0, &ids);
        assert!(plan.panes.iter().all(|(_, r)| r.y < 0.01));
        assert!((plan.panes[0].1.width - plan.panes[1].1.width).abs() < 0.01);
    }

    #[test]
    fn chrome_height_clamped_to_max() {
        let plan = compute_layout(800.0, 600.0, 999.0, &[]);
        assert_eq!(plan.chrome.height, CHROME_HEIGHT_MAX);
    }

    fn ids(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn eight_panes_use_four_plus_four() {
        let plan = compute_layout(1000.0, 800.0, 96.0, &ids(8));
        let top: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y < 1.0).collect();
        let bottom: Vec<_> = plan.panes.iter().filter(|(_, r)| r.y >= 1.0).collect();
        assert_eq!(top.len(), 4);
        assert_eq!(bottom.len(), 4);
    }

    #[test]
    fn fifteen_panes_use_three_rows_of_five() {
        let plan = compute_layout(1000.0, 800.0, 96.0, &ids(15));
        assert_eq!(plan.panes.len(), 15);
        let mut rows: Vec<Vec<f64>> = Vec::new();
        for (_, r) in &plan.panes {
            if let Some(row) = rows.iter_mut().find(|ys| (ys[0] - r.y).abs() < 0.01) {
                row.push(r.y);
            } else {
                rows.push(vec![r.y]);
            }
        }
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].len(), 5);
        assert_eq!(rows[1].len(), 5);
        assert_eq!(rows[2].len(), 5);
        assert!((plan.panes[0].1.width - plan.panes[1].1.width).abs() < 0.01);
        assert!((plan.panes[5].1.width - plan.panes[9].1.width).abs() < 0.01);
    }
}
