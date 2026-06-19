use chrono::{Datelike, NaiveDate, Weekday};
use crate::storage::{Category, Task};

fn weekday_pt(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Seg",
        Weekday::Tue => "Ter",
        Weekday::Wed => "Qua",
        Weekday::Thu => "Qui",
        Weekday::Fri => "Sex",
        Weekday::Sat => "Sáb",
        Weekday::Sun => "Dom",
    }
}

fn month_pt(m: u32) -> &'static str {
    match m {
        1 => "Jan", 2 => "Fev", 3 => "Mar", 4 => "Abr",
        5 => "Mai", 6 => "Jun", 7 => "Jul", 8 => "Ago",
        9 => "Set", 10 => "Out", 11 => "Nov", 12 => "Dez",
        _ => "???",
    }
}

pub fn format_date(date: NaiveDate) -> String {
    format!(
        "{}, {} {} {}",
        weekday_pt(date.weekday()),
        date.day(),
        month_pt(date.month()),
        date.year()
    )
}

pub fn generate_report(date: NaiveDate, tasks: &[Task], categories: &[Category]) -> String {
    let mut lines = vec![
        format!("📊 **Relatório do dia — {}**", format_date(date)),
        String::new(),
    ];

    for category in categories {
        let cat_tasks: Vec<&Task> = tasks.iter().filter(|t| &t.category == category).collect();
        lines.push(format!("{} **{}:**", category.icon(), category.label()));
        if cat_tasks.is_empty() {
            lines.push("(nenhum)".to_string());
        } else {
            for t in cat_tasks {
                lines.push(format!("• {}", t.description));
            }
        }
        lines.push(String::new());
    }

    lines.join("\n")
}
