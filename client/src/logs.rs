#[derive(Debug)]
pub struct DebugLogs {
    pub lines: Vec<String>
}

impl DebugLogs {
    fn format_text(&mut self, level: &str, message: String) {
        println!("{}", message);
        self.lines.push(format!("[{0}]: {1}", level.to_uppercase(), message));
    }
    pub fn debug(&mut self, text: String) {
        self.format_text("debug", text);
    }

    pub fn info(&mut self, text: String) {
        self.format_text("info", text);
    }

    pub fn warn(&mut self, text: String) {
        self.format_text("warn", text);
    }
    
    pub fn error(&mut self, text: String) {
        self.format_text("error", text);
    }

    pub fn clear_logs(&mut self) {
        self.lines.clear();
        self.info(lc!("Cleared logs"));
    } 

    pub fn get_logs(&mut self) -> String {
        return self.lines.join("\n");
    }
}