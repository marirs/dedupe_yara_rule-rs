use regex::Regex;
use std::path::Path;

/// Collects YAR/YARA files recursively from the provided directory path.
///
/// This function recursively searches for files with ".yar" or ".yara" extensions
/// in the specified directory and its subdirectories. It returns a vector of strings
/// containing the file paths.
///
/// # Arguments
///
/// * `path` - A reference to a `dyn AsRef<Path>` that represents the directory path.
pub fn collect_yar_files<P: AsRef<Path> + ToString>(path: P) -> Vec<String> {
    if path.as_ref().is_dir() {
        path.as_ref()
            .read_dir()
            .unwrap()
            .flatten()
            .flat_map(|c| collect_yar_files(c.path().to_string_lossy().to_string()))
            .collect::<Vec<_>>()
    } else {
        match path.as_ref().extension() {
            Some(k) if k.eq_ignore_ascii_case("yar") || k.eq_ignore_ascii_case("yara") => {
                vec![path.as_ref().to_str().unwrap().into()]
            }
            _ => vec![],
        }
    }
}

/// Removes all comments from the given string.
/// The function uses a regular expression to find and remove both single-line and multi-line comments.
///
/// # Arguments
///
/// * `st` - The input string from which comments will be removed.
///
/// # Returns
///
/// * The input string with all comments removed.
pub fn remove_comments(st: String) -> String {
    let comments = Regex::new(r#"(/\*([^*]|[\r\n]|(\*+([^*/]|[\r\n])))*\*+/|^//.*?$)"#).unwrap();

    let st = comments.replace_all(&st, "");
    st.into()
}

/// Collects import statements from a given string.
///
/// # Arguments
///
/// * `st` - A string containing import statements.
///
/// # Returns
///
/// A vector of import statements.
pub fn collect_imports(st: String) -> Vec<String> {
    st.lines()
        .map(|x| {
            if x.starts_with("import") { x } else { "" }
                .trim()
                .replace("import\"", "import \"")
                .replace(['“', '”'], "\"")
        })
        .collect()
}
