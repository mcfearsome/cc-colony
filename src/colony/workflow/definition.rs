use super::types::{WorkflowDefinition, WorkflowStep};
use crate::error::{ColonyError, ColonyResult};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Load a workflow definition from a YAML file
pub fn load_workflow_definition(path: &Path) -> ColonyResult<WorkflowDefinition> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ColonyError::Colony(format!("Failed to read workflow file: {}", e)))?;

    let definition: WorkflowDefinition = serde_yaml::from_str(&content)
        .map_err(|e| ColonyError::Colony(format!("Failed to parse workflow YAML: {}", e)))?;

    validate_workflow_definition(&definition)?;

    Ok(definition)
}

/// Save a workflow definition to a YAML file
pub fn save_workflow_definition(path: &Path, definition: &WorkflowDefinition) -> ColonyResult<()> {
    let yaml = serde_yaml::to_string(definition)
        .map_err(|e| ColonyError::Colony(format!("Failed to serialize workflow: {}", e)))?;

    std::fs::write(path, yaml)
        .map_err(|e| ColonyError::Colony(format!("Failed to write workflow file: {}", e)))?;

    Ok(())
}

/// Validate a workflow definition
pub fn validate_workflow_definition(definition: &WorkflowDefinition) -> ColonyResult<()> {
    // Check workflow name is valid
    if definition.name.is_empty() {
        return Err(ColonyError::Colony(
            "Workflow name cannot be empty".to_string(),
        ));
    }

    if definition.name.contains('/') || definition.name.contains('\\') {
        return Err(ColonyError::Colony(
            "Workflow name cannot contain path separators".to_string(),
        ));
    }

    // Check we have at least one step
    if definition.steps.is_empty() {
        return Err(ColonyError::Colony(
            "Workflow must have at least one step".to_string(),
        ));
    }

    // Check for duplicate step names
    let mut step_names = HashSet::new();
    for step in &definition.steps {
        if !step_names.insert(&step.name) {
            return Err(ColonyError::Colony(format!(
                "Duplicate step name: {}",
                step.name
            )));
        }
    }

    // Validate dependencies form a valid DAG (no cycles)
    validate_dependencies(definition)?;

    // Validate each step
    for step in &definition.steps {
        validate_step(step, definition)?;
    }

    Ok(())
}

/// Validate a single workflow step
fn validate_step(step: &WorkflowStep, definition: &WorkflowDefinition) -> ColonyResult<()> {
    // Check step name is not empty
    if step.name.is_empty() {
        return Err(ColonyError::Colony("Step name cannot be empty".to_string()));
    }

    // Check agent is specified
    if step.agent.is_empty() {
        return Err(ColonyError::Colony(format!(
            "Step '{}': agent cannot be empty",
            step.name
        )));
    }

    // Check instructions are provided
    if step.instructions.is_empty() {
        return Err(ColonyError::Colony(format!(
            "Step '{}': instructions cannot be empty",
            step.name
        )));
    }

    // Validate dependencies exist
    if let Some(deps) = &step.depends_on {
        for dep in deps {
            if !definition.steps.iter().any(|s| &s.name == dep) {
                return Err(ColonyError::Colony(format!(
                    "Step '{}': dependency '{}' not found",
                    step.name, dep
                )));
            }
        }
    }

    // Validate parallel count
    if let Some(parallel) = step.parallel {
        if parallel == 0 {
            return Err(ColonyError::Colony(format!(
                "Step '{}': parallel count must be greater than 0",
                step.name
            )));
        }
    }

    // Validate retry config
    if let Some(retry) = &step.retry {
        if retry.max_attempts == 0 {
            return Err(ColonyError::Colony(format!(
                "Step '{}': retry max_attempts must be greater than 0",
                step.name
            )));
        }
    }

    // Validate timeout format (simple validation for now)
    if let Some(timeout) = &step.timeout {
        if !is_valid_duration(timeout) {
            return Err(ColonyError::Colony(format!(
                "Step '{}': invalid timeout format '{}' (use format like '5m', '1h', '30s')",
                step.name, timeout
            )));
        }
    }

    Ok(())
}

/// Validate that dependencies form a DAG (no cycles)
fn validate_dependencies(definition: &WorkflowDefinition) -> ColonyResult<()> {
    // Build adjacency list
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in &definition.steps {
        graph.insert(&step.name, vec![]);
    }

    for step in &definition.steps {
        if let Some(deps) = &step.depends_on {
            for dep in deps {
                if let Some(edges) = graph.get_mut(dep.as_str()) {
                    edges.push(&step.name);
                }
            }
        }
    }

    // Perform topological sort using DFS
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for step in &definition.steps {
        if !visited.contains(step.name.as_str())
            && has_cycle(&graph, &step.name, &mut visited, &mut rec_stack) {
                return Err(ColonyError::Colony(
                    "Workflow contains a dependency cycle".to_string(),
                ));
            }
    }

    Ok(())
}

/// Check for cycles in the dependency graph using DFS
fn has_cycle(
    graph: &HashMap<&str, Vec<&str>>,
    node: &str,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                if has_cycle(graph, neighbor, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(neighbor) {
                return true;
            }
        }
    }

    rec_stack.remove(node);
    false
}

/// Check if a duration string is valid (simple check)
fn is_valid_duration(duration: &str) -> bool {
    if duration.is_empty() {
        return false;
    }

    let chars: Vec<char> = duration.chars().collect();
    let last = chars[chars.len() - 1];

    // Must end with a unit (s, m, h, d)
    if !matches!(last, 's' | 'm' | 'h' | 'd') {
        return false;
    }

    // Everything before the unit must be a number
    let num_part = &duration[..duration.len() - 1];
    num_part.parse::<u64>().is_ok()
}

/// Get the topological order of steps (for execution planning)
pub fn topological_sort(definition: &WorkflowDefinition) -> ColonyResult<Vec<Vec<String>>> {
    // Build dependency map
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();

    for step in &definition.steps {
        in_degree.insert(step.name.clone(), 0);
        graph.insert(step.name.clone(), vec![]);
    }

    for step in &definition.steps {
        if let Some(deps) = &step.depends_on {
            *in_degree.get_mut(&step.name).unwrap() += deps.len();
            for dep in deps {
                graph.get_mut(dep).unwrap().push(step.name.clone());
            }
        }
    }

    // Topological sort with level grouping
    let mut levels = vec![];
    let mut current_level = vec![];

    // Find steps with no dependencies (level 0)
    for (step_name, &degree) in &in_degree {
        if degree == 0 {
            current_level.push(step_name.clone());
        }
    }

    while !current_level.is_empty() {
        levels.push(current_level.clone());
        let mut next_level = vec![];

        for step_name in &current_level {
            if let Some(dependents) = graph.get(step_name) {
                for dependent in dependents {
                    let degree = in_degree.get_mut(dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        next_level.push(dependent.clone());
                    }
                }
            }
        }

        current_level = next_level;
    }

    // Check if all steps were processed
    let total_steps: usize = levels.iter().map(|level| level.len()).sum();
    if total_steps != definition.steps.len() {
        return Err(ColonyError::Colony(
            "Workflow contains a dependency cycle".to_string(),
        ));
    }

    Ok(levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_duration() {
        assert!(is_valid_duration("5m"));
        assert!(is_valid_duration("30s"));
        assert!(is_valid_duration("1h"));
        assert!(is_valid_duration("2d"));
        assert!(!is_valid_duration(""));
        assert!(!is_valid_duration("5"));
        assert!(!is_valid_duration("m5"));
        assert!(!is_valid_duration("abc"));
    }
}
