/// Default names to search for in the CWD when the binary is called.
pub enum FilefileNames {
    YAML,
    JSON,
    TOML,
}
impl FilefileNames {
    pub fn as_str(&self) -> &'static str {
        match self {
            FilefileNames::YAML => "Filefile.yaml",
            FilefileNames::JSON => "Filefile.json",
            FilefileNames::TOML => "Filefile.toml",
        }
    }

    pub fn default() -> FilefileNames {
        FilefileNames::YAML
    }

    #[allow(dead_code)]
    pub fn default_string() -> String {
        String::from("Filefile.yaml")
    }
}

pub struct FilefileNamesIterator {
    current: Option<FilefileNames>,
}

impl FilefileNamesIterator {
    pub fn new() -> Self {
        Self {
            current: Some(FilefileNames::YAML),
        }
    }
}

impl Iterator for FilefileNamesIterator {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?; // Take the current variant
        self.current = match current {
            FilefileNames::YAML => Some(FilefileNames::JSON),
            FilefileNames::JSON => Some(FilefileNames::TOML),
            FilefileNames::TOML => None, // End of iteration
        };
        Some(current.as_str())
    }
}
