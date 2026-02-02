use colored::*;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use operator::crd::{IndustrialPLC, PLCPhase};

pub enum StatusStyle {
    Success,
    Warning,
    Error,
    Neutral,
}

/// Print a beautiful ASCII table of PLC status
pub fn print_plc_table(plcs: &[IndustrialPLC]) {
    if plcs.is_empty() {
        println!("{}", "⚠️  No IndustrialPLC resources found".yellow());
        return;
    }
    
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("PLC Name").fg(Color::Cyan),
            Cell::new("Device").fg(Color::Cyan),
            Cell::new("Register").fg(Color::Cyan),
            Cell::new("Desired").fg(Color::Cyan),
            Cell::new("Actual").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Phase").fg(Color::Cyan),
            Cell::new("Drifts").fg(Color::Cyan),
        ]);
    
    for plc in plcs {
        let name = plc.metadata.name.as_deref().unwrap_or("unknown");
        let device = format!("{}:{}", plc.spec.device_address, plc.spec.port);
        let register = plc.spec.target_register.to_string();
        let desired = plc.spec.target_value.to_string();
        
        let (actual, status, phase, drifts) = if let Some(ref s) = plc.status {
            let actual_str = s.current_value.map(|v: u16| v.to_string()).unwrap_or_else(|| "-".to_string());
            
            let status_str = if s.in_sync {
                "✓ SYNCED".to_string()
            } else if s.phase == PLCPhase::DriftDetected {
                "⚠ DRIFT".to_string()
            } else {
                "✗ UNKNOWN".to_string()
            };
            
            (actual_str, status_str, format!("{:?}", s.phase), s.drift_events.to_string())
        } else {
            ("-".to_string(), "PENDING".to_string(), "Pending".to_string(), "0".to_string())
        };
        
        // Colorize status
        let status_cell = match status.as_str() {
            "✓ SYNCED" => Cell::new(status).fg(Color::Green),
            "⚠ DRIFT" => Cell::new(status).fg(Color::Yellow),
            _ => Cell::new(status).fg(Color::Red),
        };
        
        // Colorize phase
        let phase_cell = match phase.as_str() {
            "Connected" => Cell::new(phase).fg(Color::Green),
            "DriftDetected" => Cell::new(phase).fg(Color::Yellow),
            "Correcting" => Cell::new(phase).fg(Color::Blue),
            "Failed" => Cell::new(phase).fg(Color::Red),
            _ => Cell::new(phase).fg(Color::Grey),
        };
        
        table.add_row(vec![
            Cell::new(name),
            Cell::new(device),
            Cell::new(register),
            Cell::new(desired).fg(Color::Green),
            Cell::new(actual),
            status_cell,
            phase_cell,
            Cell::new(drifts),
        ]);
    }
    
    println!("{}", table);
}

/// Print a status summary box
pub fn print_status_summary(status: &operator::crd::IndustrialPLCStatus, style: StatusStyle) {
    let border_color = match style {
        StatusStyle::Success => Color::Green,
        StatusStyle::Warning => Color::Yellow,
        StatusStyle::Error => Color::Red,
        StatusStyle::Neutral => Color::Grey,
    };
    
    let status_icon = match style {
        StatusStyle::Success => "✓",
        StatusStyle::Warning => "⚠",
        StatusStyle::Error => "✗",
        StatusStyle::Neutral => "○",
    };
    
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);
    
    table.set_header(vec![Cell::new(format!("{} Status Summary", status_icon)).fg(border_color)]);
    
    table.add_row(vec![
        Cell::new("Phase:"),
        Cell::new(format!("{:?}", status.phase)).fg(border_color),
    ]);
    
    table.add_row(vec![
        Cell::new("In Sync:"),
        Cell::new(if status.in_sync { "Yes ✓" } else { "No ✗" })
            .fg(if status.in_sync { Color::Green } else { Color::Red }),
    ]);
    
    if let Some(value) = status.current_value {
        table.add_row(vec![
            Cell::new("Current Value:"),
            Cell::new(value.to_string()),
        ]);
    }
    
    table.add_row(vec![
        Cell::new("Drift Events:"),
        Cell::new(status.drift_events.to_string()),
    ]);
    
    table.add_row(vec![
        Cell::new("Corrections:"),
        Cell::new(status.corrections_applied.to_string()).fg(Color::Green),
    ]);
    
    if let Some(ref error) = status.last_error {
        table.add_row(vec![
            Cell::new("Last Error:"),
            Cell::new(error).fg(Color::Red),
        ]);
    }
    
    table.add_row(vec![
        Cell::new("Message:"),
        Cell::new(&status.message),
    ]);
    
    if let Some(ref updated) = status.last_update {
        table.add_row(vec![
            Cell::new("Last Update:"),
            Cell::new(updated).fg(Color::Grey),
        ]);
    }
    
    println!("{}", table);
}

/// Print a simple status line
pub fn print_status_line(plc: &IndustrialPLC) {
    let name = plc.metadata.name.as_deref().unwrap_or("unknown");
    
    if let Some(ref status) = plc.status {
        let emoji = if status.in_sync { "✓" } else { "✗" };
        let color = if status.in_sync { "green" } else { "red" };
        
        println!("{} {}: {} (phase: {:?})", 
            emoji,
            name,
            if status.in_sync { "SYNCED".color(color) } else { "DRIFT".color(color) },
            status.phase
        );
    } else {
        println!("○ {}: {}", name, "PENDING".dimmed());
    }
}
