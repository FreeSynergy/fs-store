//! Pattern: none (pure functions from the state to sentences; the sentences themselves are in `words`).

use crate::backend::JobError;
use crate::words;

use super::store::StoreModel;
use super::types::Problem;

impl StoreModel {
    /// One sentence about the job, or about there being none.
    #[must_use]
    pub fn job_line(&self) -> String {
        let Some(job) = &self.job else {
            return words::NO_JOB.to_owned();
        };
        let version = job.version.map(|v| v.to_string()).unwrap_or_default();
        let values = [("name", job.name.as_str()), ("version", version.as_str())];
        words::fill(words::job_phase(job.phase), &values)
    }

    /// Lines about where the installed package `name` is: how, which version, commands, folder, source.
    #[must_use]
    pub fn detail_lines(&self, name: &str) -> Vec<String> {
        let d = self.details(name);
        let mut lines = Vec::new();
        if !d.kinds.is_empty() {
            let kinds: Vec<&str> = d.kinds.iter().map(|k| words::install_kind(*k)).collect();
            lines.push(words::fill(
                words::DETAIL_KIND,
                &[("kind", &kinds.join(", "))],
            ));
        }
        for (command, path) in &d.commands {
            lines.push(words::fill(
                words::DETAIL_COMMAND,
                &[("command", command), ("path", path)],
            ));
        }
        if let Some(folder) = &d.folder {
            lines.push(words::fill(words::DETAIL_FOLDER, &[("folder", folder)]));
        }
        if let Some(source) = &d.source {
            lines.push(words::fill(words::DETAIL_SOURCE, &[("source", source)]));
        }
        lines
    }

    /// The lines of the About page.
    #[must_use]
    pub fn about_lines() -> Vec<String> {
        let version = env!("CARGO_PKG_VERSION");
        vec![
            words::fill(words::ABOUT_NAME, &[("version", version)]),
            words::ABOUT_LICENCE.to_owned(),
            words::fill(words::ABOUT_URL, &[("url", words::PROJECT_URL)]),
        ]
    }

    /// The sentences for [`StoreModel::problems`].
    #[must_use]
    pub fn problem_lines(&self) -> Vec<String> {
        self.problems
            .iter()
            .map(|p| match p {
                Problem::Source(e) => words::source_problem(e).to_owned(),
                Problem::Backend(detail) => {
                    words::fill(words::STORE_PROBLEM, &[("detail", detail)])
                }
                Problem::Unreadable { source, detail } => words::fill(
                    words::SOURCE_UNREADABLE,
                    &[("source", source), ("detail", detail)],
                ),
                Problem::Skipped { id, reason } => {
                    words::fill(words::ENTRY_SKIPPED, &[("id", id), ("reason", reason)])
                }
                Problem::Busy(name) => words::fill(words::JOB_BUSY, &[("name", name)]),
                Problem::JobFailed { name, error } => job_failed_line(name, error),
                Problem::NothingToDo(name) => words::fill(words::NOTHING_TO_DO, &[("name", name)]),
                Problem::LaunchFailed { command, detail } if command.is_empty() => {
                    words::fill(words::OPEN_NO_COMMAND, &[("detail", detail)])
                }
                Problem::LaunchFailed { command, detail } => words::fill(
                    words::OPEN_FAILED,
                    &[("command", command), ("detail", detail)],
                ),
                Problem::UninstallFailed { name, detail } => words::fill(
                    words::UNINSTALL_FAILED,
                    &[("name", name), ("detail", detail)],
                ),
                Problem::NoHomepage(name) => words::fill(words::NO_HOMEPAGE, &[("name", name)]),
                Problem::RepositoryFailed { url, detail } => words::fill(
                    words::REPOSITORY_FAILED,
                    &[("url", url), ("detail", detail)],
                ),
                Problem::EmptyAddress => words::EMPTY_ADDRESS.to_owned(),
            })
            .collect()
    }
}

fn job_failed_line(name: &str, error: &JobError) -> String {
    let detail = match error {
        JobError::Other(text) | JobError::Unsupported(text) => text.as_str(),
        _ => "",
    };
    words::fill(
        words::job_problem(error),
        &[("name", name), ("detail", detail)],
    )
}
