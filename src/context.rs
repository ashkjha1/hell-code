use ignore::WalkBuilder;

pub fn get_project_context(root: &str) -> String {
    let mut context = String::new();
    context.push_str("Project Structure:\n");
    
    for result in WalkBuilder::new(root).hidden(true).build() {
        if let Ok(entry) = result {
            let depth = entry.depth();
            let indent = "  ".repeat(depth);
            let path = entry.path();
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            
            if path.is_dir() {
                context.push_str(&format!("{}{} [DIR]\n", indent, name));
            } else {
                context.push_str(&format!("{}{}\n", indent, name));
            }
        }
    }
    context
}
