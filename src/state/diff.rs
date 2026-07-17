use std::path::PathBuf;

pub struct FileDiff {
    pub added: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
}

pub struct TextDiffEntry {
    pub path: PathBuf,
    pub change: String,
}

pub struct TextDiff {
    pub added: Vec<TextDiffEntry>,
    pub deleted: Vec<TextDiffEntry>,
    pub modified: Vec<TextDiffEntry>,
}

impl FileDiff {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.deleted.is_empty() && self.modified.is_empty()
    }
}

impl std::fmt::Display for FileDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for change in &self.added {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Added:")?;
            }
        }
        for change in &self.deleted {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Deleted:")?;
            }
        }
        for change in &self.modified {
            if let Some(change) = change.to_str() {
                write!(f, "\n{:<11}{change}", "Modified:")?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for TextDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in &self.added {
            writeln!(f, "{:<11}{}", "Added:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.deleted {
            writeln!(f, "{:<11}{}", "Deleted:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }
        for entry in &self.modified {
            writeln!(f, "{:<11}{}", "Modified:", entry.path.display())?;
            writeln!(f, "{}", entry.change)?;
        }

        Ok(())
    }
}
