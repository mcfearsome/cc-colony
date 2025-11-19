// Example: ASCII task dependency graph renderer
// Pure terminal, no graphics required

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Completed,
}

impl TaskStatus {
    fn symbol(&self) -> &str {
        match self {
            TaskStatus::Pending => "○",
            TaskStatus::InProgress => "◐",
            TaskStatus::Blocked => "✗",
            TaskStatus::Completed => "●",
        }
    }

    fn color(&self) -> &str {
        match self {
            TaskStatus::Pending => "\x1b[33m",      // Yellow
            TaskStatus::InProgress => "\x1b[32m",   // Green
            TaskStatus::Blocked => "\x1b[31m",      // Red
            TaskStatus::Completed => "\x1b[90m",    // Gray
        }
    }
}

pub struct AsciiGraphRenderer {
    width: usize,
    height: usize,
}

impl AsciiGraphRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Render task graph in ASCII using topological sort
    pub fn render(&self, tasks: &[Task]) -> String {
        let mut output = String::new();
        let reset = "\x1b[0m";

        // Build adjacency list
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();

        for task in tasks {
            graph.entry(&task.id).or_insert_with(Vec::new);
            in_degree.entry(&task.id).or_insert(0);

            for dep in &task.dependencies {
                graph.entry(dep.as_str())
                    .or_insert_with(Vec::new)
                    .push(&task.id);
                *in_degree.entry(&task.id).or_insert(0) += 1;
            }
        }

        // Topological sort by levels
        let mut levels: Vec<Vec<&str>> = Vec::new();
        let mut current_level: Vec<&str> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut visited = HashSet::new();

        while !current_level.is_empty() {
            levels.push(current_level.clone());

            let mut next_level = Vec::new();
            for &node in &current_level {
                visited.insert(node);

                if let Some(neighbors) = graph.get(node) {
                    for &neighbor in neighbors {
                        let deps_satisfied = tasks.iter()
                            .find(|t| t.id == neighbor)
                            .map(|t| t.dependencies.iter().all(|d| visited.contains(d.as_str())))
                            .unwrap_or(false);

                        if deps_satisfied && !next_level.contains(&neighbor) {
                            next_level.push(neighbor);
                        }
                    }
                }
            }

            current_level = next_level;
        }

        // Render header
        output.push_str("┌");
        output.push_str(&"─".repeat(self.width - 2));
        output.push_str("┐\n");
        output.push_str(&format!("│{:^width$}│\n", "Task Dependency Graph", width = self.width - 2));
        output.push_str("├");
        output.push_str(&"─".repeat(self.width - 2));
        output.push_str("┤\n");

        // Render levels
        for (level_idx, level) in levels.iter().enumerate() {
            // Level header
            output.push_str(&format!("│ Level {} ", level_idx));
            output.push_str(&"─".repeat(self.width - 12));
            output.push_str(" │\n");

            // Tasks in this level
            for task_id in level {
                let task = tasks.iter().find(|t| t.id == *task_id).unwrap();
                let status_color = task.status.color();
                let status_symbol = task.status.symbol();

                output.push_str("│  ");
                output.push_str(status_color);
                output.push_str(status_symbol);
                output.push_str(reset);
                output.push_str(&format!(" {:<30} ", task.title));

                // Show dependencies
                if !task.dependencies.is_empty() {
                    output.push_str("← ");
                    let dep_str = task.dependencies.join(", ");
                    output.push_str(&dep_str[..dep_str.len().min(20)]);
                }

                // Pad to width
                let current_len = 3 + 1 + 31 +
                    if task.dependencies.is_empty() { 0 }
                    else { 2 + task.dependencies.join(", ").len().min(20) };
                output.push_str(&" ".repeat(self.width.saturating_sub(current_len + 1)));
                output.push_str("│\n");
            }

            // Connection lines
            if level_idx < levels.len() - 1 {
                output.push_str("│  ");
                for (i, _) in level.iter().enumerate() {
                    if i > 0 {
                        output.push_str("    ");
                    }
                    output.push_str("  │  ");
                }
                output.push('\n');
                output.push_str("│  ");
                for (i, _) in level.iter().enumerate() {
                    if i > 0 {
                        output.push_str("    ");
                    }
                    output.push_str("  ↓  ");
                }
                output.push('\n');
            }
        }

        // Footer with legend
        output.push_str("├");
        output.push_str(&"─".repeat(self.width - 2));
        output.push_str("┤\n");
        output.push_str("│ Legend: ");
        output.push_str(&format!("{}○{} Pending  ", TaskStatus::Pending.color(), reset));
        output.push_str(&format!("{}◐{} In Progress  ", TaskStatus::InProgress.color(), reset));
        output.push_str(&format!("{}✗{} Blocked  ", TaskStatus::Blocked.color(), reset));
        output.push_str(&format!("{}●{} Done", TaskStatus::Completed.color(), reset));
        let legend_len = "│ Legend: ○ Pending  ◐ In Progress  ✗ Blocked  ● Done".len();
        output.push_str(&" ".repeat(self.width.saturating_sub(legend_len + 1)));
        output.push_str("│\n");
        output.push_str("└");
        output.push_str(&"─".repeat(self.width - 2));
        output.push_str("┘\n");

        output
    }

    /// Render simplified tree view
    pub fn render_tree(&self, tasks: &[Task]) -> String {
        let mut output = String::new();
        let reset = "\x1b[0m";

        // Find root tasks (no dependencies)
        let roots: Vec<_> = tasks.iter()
            .filter(|t| t.dependencies.is_empty())
            .collect();

        output.push_str("Task Dependency Tree\n");
        output.push_str(&"━".repeat(40));
        output.push('\n');

        for (idx, root) in roots.iter().enumerate() {
            let is_last_root = idx == roots.len() - 1;
            self.render_task_recursive(
                root,
                tasks,
                &mut output,
                "",
                is_last_root,
                reset,
            );
        }

        output
    }

    fn render_task_recursive(
        &self,
        task: &Task,
        all_tasks: &[Task],
        output: &mut String,
        prefix: &str,
        is_last: bool,
        reset: &str,
    ) {
        // Current task
        let branch = if is_last { "└─ " } else { "├─ " };
        let status_color = task.status.color();
        let status_symbol = task.status.symbol();

        output.push_str(prefix);
        output.push_str(branch);
        output.push_str(status_color);
        output.push_str(status_symbol);
        output.push_str(reset);
        output.push_str(" ");
        output.push_str(&task.title);
        output.push_str(&format!(" ({})", task.id));
        output.push('\n');

        // Find children (tasks that depend on this one)
        let children: Vec<_> = all_tasks.iter()
            .filter(|t| t.dependencies.contains(&task.id))
            .collect();

        // Render children
        let new_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });
        for (idx, child) in children.iter().enumerate() {
            let is_last_child = idx == children.len() - 1;
            self.render_task_recursive(
                child,
                all_tasks,
                output,
                &new_prefix,
                is_last_child,
                reset,
            );
        }
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_graph() {
        let tasks = vec![
            Task {
                id: "task-1".to_string(),
                title: "Setup Project".to_string(),
                status: TaskStatus::Completed,
                dependencies: vec![],
            },
            Task {
                id: "task-2".to_string(),
                title: "Implement Auth".to_string(),
                status: TaskStatus::InProgress,
                dependencies: vec!["task-1".to_string()],
            },
            Task {
                id: "task-3".to_string(),
                title: "Build UI".to_string(),
                status: TaskStatus::Pending,
                dependencies: vec!["task-2".to_string()],
            },
            Task {
                id: "task-4".to_string(),
                title: "Write Tests".to_string(),
                status: TaskStatus::Pending,
                dependencies: vec!["task-2".to_string()],
            },
            Task {
                id: "task-5".to_string(),
                title: "Deploy".to_string(),
                status: TaskStatus::Blocked,
                dependencies: vec!["task-3".to_string(), "task-4".to_string()],
            },
        ];

        let renderer = AsciiGraphRenderer::new(80, 30);
        println!("{}", renderer.render(&tasks));
        println!("\n");
        println!("{}", renderer.render_tree(&tasks));
    }
}

fn main() {
    // Demo
    let tasks = vec![
        Task {
            id: "task-1".to_string(),
            title: "Setup Project".to_string(),
            status: TaskStatus::Completed,
            dependencies: vec![],
        },
        Task {
            id: "task-2".to_string(),
            title: "Implement Authentication".to_string(),
            status: TaskStatus::InProgress,
            dependencies: vec!["task-1".to_string()],
        },
        Task {
            id: "task-3".to_string(),
            title: "Build Frontend UI".to_string(),
            status: TaskStatus::Pending,
            dependencies: vec!["task-2".to_string()],
        },
        Task {
            id: "task-4".to_string(),
            title: "Write Integration Tests".to_string(),
            status: TaskStatus::Pending,
            dependencies: vec!["task-2".to_string()],
        },
        Task {
            id: "task-5".to_string(),
            title: "Setup CI/CD Pipeline".to_string(),
            status: TaskStatus::Pending,
            dependencies: vec!["task-1".to_string()],
        },
        Task {
            id: "task-6".to_string(),
            title: "Deploy to Production".to_string(),
            status: TaskStatus::Blocked,
            dependencies: vec!["task-3".to_string(), "task-4".to_string(), "task-5".to_string()],
        },
    ];

    let renderer = AsciiGraphRenderer::new(80, 30);

    println!("\n=== Layered Graph View ===\n");
    println!("{}", renderer.render(&tasks));

    println!("\n=== Tree View ===\n");
    println!("{}", renderer.render_tree(&tasks));
}
