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
        let mut line = words::fill(words::job_phase(job.phase), &values);
        if let Some(error) = &job.error {
            let detail = match error {
                JobError::Other(text) => text.as_str(),
                _ => "",
            };
            line.push(' ');
            line.push_str(&words::fill(
                words::job_problem(error),
                &[("name", job.name.as_str()), ("detail", detail)],
            ));
        }
        line
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
            })
            .collect()
    }
}
