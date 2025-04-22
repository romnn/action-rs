pub const ENV_VAR: &str = "GITHUB_STEP_SUMMARY";
pub const DOCS_URL: &str = "https://docs.github.com/actions/using-workflows/workflow-commands-for-github-actions#adding-a-job-summary";

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct TableCell {
    /// Cell content
    pub data: String,
    /// Render cell as header
    pub header: bool,
    /// Number of columns the cell extends
    pub colspan: usize,
    /// Number of rows the cell extends
    pub rowspan: usize,
}

impl TableCell {
    #[must_use]
    pub fn new(data: String) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn header(data: String) -> Self {
        Self {
            data,
            header: true,
            ..Self::default()
        }
    }
}

impl Default for TableCell {
    fn default() -> Self {
        Self {
            data: String::new(),
            header: false,
            colspan: 1,
            rowspan: 1,
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct ImageOptions {
    /// The width of the image in pixels.
    width: Option<usize>,

    /// The height of the image in pixels.
    height: Option<usize>,
}

// todo: finish porting the summary stuff
// finish the proc macro, and test it!
// continue with the cache stuff?
