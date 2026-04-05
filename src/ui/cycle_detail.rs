use chrono::NaiveDate;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState},
};

use super::{format_date, issue_list::issue_row};
use crate::app::App;

#[derive(Debug)]
pub struct BurndownData {
    pub actual: Vec<(f64, f64)>,
    pub ideal: Vec<(f64, f64)>,
    pub today_x: f64,
    pub total: f64,
    pub remaining: f64,
    pub using_estimate: bool,
}

pub fn compute_burndown(
    issues: &[crate::api::types::Issue],
    starts_at: &str,
    ends_at: &str,
    today: NaiveDate,
) -> BurndownData {
    let start = match NaiveDate::parse_from_str(&starts_at[..10], "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return BurndownData { actual: vec![], ideal: vec![], today_x: 0.0, total: 0.0, remaining: 0.0, using_estimate: false },
    };
    let end = match NaiveDate::parse_from_str(&ends_at[..10], "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => return BurndownData { actual: vec![], ideal: vec![], today_x: 0.0, total: 0.0, remaining: 0.0, using_estimate: false },
    };
    let total_days = (end - start).num_days();

    let estimate_sum: f64 = issues.iter().filter_map(|i| i.estimate).sum();
    let using_estimate = estimate_sum > 0.0;
    let total = if using_estimate { estimate_sum } else { issues.len() as f64 };

    if total == 0.0 {
        return BurndownData { actual: vec![], ideal: vec![], today_x: 0.0, total: 0.0, remaining: 0.0, using_estimate };
    }

    let ideal = vec![(0.0, total), (total_days as f64, 0.0)];

    let last_day = if today < start {
        return BurndownData { actual: vec![], ideal, today_x: 0.0, total, remaining: total, using_estimate };
    } else if today > end {
        end
    } else {
        today
    };
    let today_x = ((last_day - start).num_days() as f64).min(total_days as f64);

    let days_to_plot = (last_day - start).num_days();
    let mut actual = Vec::with_capacity(days_to_plot as usize + 1);
    for day_idx in 0..=days_to_plot {
        let day_end = start + chrono::Duration::try_days(day_idx).unwrap();
        let day_end_str = day_end.format("%Y-%m-%d").to_string();
        let completed: f64 = issues
            .iter()
            .filter(|i| {
                i.completed_at
                    .as_deref()
                    .and_then(|s| s.get(..10))
                    .map(|d| d <= day_end_str.as_str())
                    .unwrap_or(false)
            })
            .map(|i| if using_estimate { i.estimate.unwrap_or(0.0) } else { 1.0 })
            .sum();
        actual.push((day_idx as f64, total - completed));
    }

    let remaining = actual.last().map(|&(_, r)| r).unwrap_or(total);
    BurndownData { actual, ideal, today_x, total, remaining, using_estimate }
}

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let Some(cycle) = &app.current_cycle else {
        return;
    };
    let th = &app.theme;

    let chunks = Layout::vertical([
        Constraint::Length(3), // cycle info
        Constraint::Min(0),    // issues table
        Constraint::Length(1), // footer
    ])
    .split(area);

    // Cycle info
    let progress = cycle
        .progress
        .map(|p| format!("{:.0}%", p * 100.0))
        .unwrap_or_else(|| "-".to_string());
    let start = format_date(cycle.starts_at.as_deref());
    let end = format_date(cycle.ends_at.as_deref());
    let cycle_name = cycle
        .name
        .clone()
        .unwrap_or_else(|| format!("Cycle #{}", cycle.number.unwrap_or(0.0)));

    let meta = Paragraph::new(vec![Line::from(vec![
        Span::styled(" Progress: ", Style::default().fg(th.text_dim)),
        Span::styled(progress, Style::default().fg(th.success)),
        Span::raw("    "),
        Span::styled("Start: ", Style::default().fg(th.text_dim)),
        Span::raw(start),
        Span::raw("    "),
        Span::styled("End: ", Style::default().fg(th.text_dim)),
        Span::raw(end),
    ])])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", cycle_name))
            .title_style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD)),
    );
    f.render_widget(meta, chunks[0]);

    // Issues table
    let loading = if app.loading {
        format!(" ({} Loading...)", app.spinner_symbol())
    } else {
        String::new()
    };
    let rows: Vec<Row> = app
        .cycle_issues
        .iter()
        .map(|issue| issue_row(issue, th))
        .collect();

    let header = Row::new(vec!["ID", "Title", "Status", "Priority", "Assignee"])
        .style(Style::default().fg(th.accent).add_modifier(Modifier::BOLD));

    let widths = [
        Constraint::Length(10),
        Constraint::Min(20),
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(16),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Issues ({}){}",
            app.cycle_issues.len(),
            loading
        )))
        .row_highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(th.highlight_fg),
        )
        .highlight_symbol(" > ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.selected_cycle_issue_index));
    f.render_stateful_widget(table, chunks[1], &mut table_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Esc/q", Style::default().fg(th.accent)),
        Span::raw(":back "),
        Span::styled("j/k", Style::default().fg(th.accent)),
        Span::raw(":move "),
        Span::styled("Enter", Style::default().fg(th.accent)),
        Span::raw(":detail "),
    ]));
    f.render_widget(footer, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{Issue, Priority};
    use chrono::NaiveDate;

    fn make_issue(estimate: Option<f64>, completed_at: Option<&str>) -> Issue {
        Issue {
            id: "i".into(),
            identifier: "ENG-1".into(),
            title: "t".into(),
            priority: Priority::None,
            priority_label: None,
            state: None,
            assignee: None,
            labels: None,
            description: None,
            created_at: None,
            updated_at: None,
            completed_at: completed_at.map(|s| s.to_string()),
            estimate,
            comments: None,
            project: None,
            cycle: None,
        }
    }

    #[test]
    fn burndown_uses_estimates_when_set() {
        let issues = vec![
            make_issue(Some(5.0), Some("2026-03-03T10:00:00.000Z")),
            make_issue(Some(8.0), None),
        ];
        let today = NaiveDate::from_ymd_opt(2026, 3, 5).unwrap();
        let bd = compute_burndown(&issues, "2026-03-01T00:00:00.000Z", "2026-03-14T00:00:00.000Z", today);
        assert_eq!(bd.total, 13.0);
        // day 0 (Mar 1): nothing completed yet → 13 remaining
        assert_eq!(bd.actual[0], (0.0, 13.0));
        // day 2 (Mar 3): issue-001 (5pt) completed → 8 remaining
        assert_eq!(bd.actual[2], (2.0, 8.0));
        // today_x = 4 (Mar 5)
        assert_eq!(bd.today_x, 4.0);
    }

    #[test]
    fn burndown_falls_back_to_issue_count_when_no_estimates() {
        let issues = vec![
            make_issue(None, Some("2026-03-03T00:00:00.000Z")),
            make_issue(None, None),
            make_issue(None, None),
        ];
        let today = NaiveDate::from_ymd_opt(2026, 3, 4).unwrap();
        let bd = compute_burndown(&issues, "2026-03-01T00:00:00.000Z", "2026-03-14T00:00:00.000Z", today);
        assert_eq!(bd.total, 3.0);
        assert!(!bd.using_estimate);
        // day 2: 1 issue completed → 2 remaining
        assert_eq!(bd.actual[2], (2.0, 2.0));
    }

    #[test]
    fn burndown_empty_when_cycle_not_started() {
        let issues = vec![make_issue(Some(5.0), None)];
        let today = NaiveDate::from_ymd_opt(2026, 2, 28).unwrap();
        let bd = compute_burndown(&issues, "2026-03-01T00:00:00.000Z", "2026-03-14T00:00:00.000Z", today);
        assert!(bd.actual.is_empty());
        assert_eq!(bd.ideal, vec![(0.0, 5.0), (13.0, 0.0)]);
    }

    #[test]
    fn burndown_clamps_to_cycle_end() {
        let issues = vec![make_issue(Some(3.0), Some("2026-03-10T00:00:00.000Z"))];
        let today = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
        let bd = compute_burndown(&issues, "2026-03-01T00:00:00.000Z", "2026-03-14T00:00:00.000Z", today);
        assert_eq!(bd.actual.last().unwrap().0, 13.0);
        assert_eq!(bd.today_x, 13.0);
    }
}
