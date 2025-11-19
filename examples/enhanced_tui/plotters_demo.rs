// Example: Using plotters to generate charts
// Can render to PNG (then show via terminal graphics) or ASCII

#[cfg(feature = "charts")]
use plotters::prelude::*;

#[cfg(feature = "charts")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Colony Metrics Dashboard\n");

    // Example data
    let agent_cpu = vec![
        ("backend-1", 45),
        ("frontend-1", 32),
        ("test-1", 78),
        ("devops-1", 23),
    ];

    let task_counts = vec![
        ("Pending", 5),
        ("In Progress", 3),
        ("Blocked", 1),
        ("Completed", 12),
    ];

    // 1. Bar chart to PNG
    println!("Generating bar chart...");
    generate_bar_chart(&task_counts)?;

    // 2. Line chart to PNG
    println!("Generating CPU usage chart...");
    generate_line_chart()?;

    // 3. ASCII chart (works everywhere)
    println!("\nASCII Chart (works in any terminal):");
    print_ascii_chart(&agent_cpu);

    println!("\n✓ Charts generated!");
    println!("  - task_barchart.png");
    println!("  - cpu_linechart.png");
    println!("\nTo view in terminal (if supported):");
    println!("  viuer task_barchart.png");
    println!("  # or");
    println!("  kitty +kitten icat task_barchart.png");

    Ok(())
}

#[cfg(feature = "charts")]
fn generate_bar_chart(data: &[(&str, i32)]) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("task_barchart.png", (640, 480))
        .into_drawing_area();

    root.fill(&WHITE)?;

    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(100);

    let mut chart = ChartBuilder::on(&root)
        .caption("Task Status Distribution", ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(
            0..data.len(),
            0..max_value + 5,
        )?;

    chart.configure_mesh()
        .y_desc("Count")
        .draw()?;

    chart.draw_series(
        data.iter().enumerate().map(|(idx, (label, value))| {
            let color = match *label {
                "Pending" => &YELLOW,
                "In Progress" => &GREEN,
                "Blocked" => &RED,
                "Completed" => &BLUE,
                _ => &BLACK,
            };

            Rectangle::new(
                [(idx, 0), (idx, *value)],
                color.mix(0.8).filled(),
            )
        }),
    )?;

    // Add labels
    for (idx, (label, _)) in data.iter().enumerate() {
        chart.draw_series(std::iter::once(Text::new(
            *label,
            (idx, -2),
            ("sans-serif", 15).into_font(),
        )))?;
    }

    root.present()?;
    Ok(())
}

#[cfg(feature = "charts")]
fn generate_line_chart() -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new("cpu_linechart.png", (800, 400))
        .into_drawing_area();

    root.fill(&WHITE)?;

    // Simulated CPU usage over time
    let agents = vec![
        ("backend-1", vec![10, 20, 35, 45, 50, 48, 45, 42, 40, 38]),
        ("frontend-1", vec![5, 8, 15, 25, 32, 35, 34, 33, 32, 30]),
        ("test-1", vec![60, 65, 70, 75, 78, 77, 76, 75, 74, 73]),
    ];

    let mut chart = ChartBuilder::on(&root)
        .caption("Agent CPU Usage Over Time", ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..9, 0..100)?;

    chart.configure_mesh()
        .x_desc("Time (seconds)")
        .y_desc("CPU %")
        .draw()?;

    let colors = [&RED, &BLUE, &GREEN];

    for (idx, (name, data)) in agents.iter().enumerate() {
        chart.draw_series(LineSeries::new(
            data.iter().enumerate().map(|(x, y)| (x, *y)),
            colors[idx],
        ))?
        .label(*name)
        .legend(move |(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], colors[idx])
        });
    }

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    Ok(())
}

#[cfg(feature = "charts")]
fn print_ascii_chart(data: &[(&str, i32)]) {
    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(100) as f32;
    let bar_width = 40;

    println!("┌─────────────────────────────────────────────────────────┐");
    println!("│              Agent CPU Usage (%)                        │");
    println!("├─────────────────────────────────────────────────────────┤");

    for (label, value) in data {
        let bar_length = ((*value as f32 / max_value) * bar_width as f32) as usize;
        let bar = "█".repeat(bar_length);
        let empty = "░".repeat(bar_width - bar_length);

        let color = if *value > 70 {
            "\x1b[31m" // Red
        } else if *value > 50 {
            "\x1b[33m" // Yellow
        } else {
            "\x1b[32m" // Green
        };

        println!(
            "│ {:<12} │ {}{}{}\x1b[0m │ {:3}% │",
            label, color, bar, empty, value
        );
    }

    println!("└─────────────────────────────────────────────────────────┘");
}

#[cfg(not(feature = "charts"))]
fn main() {
    eprintln!("This example requires the 'charts' feature.");
    eprintln!("Run with: cargo run --bin plotters_demo --features charts");
}
