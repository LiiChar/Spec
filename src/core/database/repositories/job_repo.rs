/// Job Repository
/// Manages job records

use rusqlite::{Connection, Result};
use crate::core::JobModel;
use super::super::schema::jobs as schema;

pub struct JobRepository;

impl JobRepository {
    /// Get all jobs
    pub fn get_all(conn: &Connection) -> Result<Vec<JobModel>> {
        let mut stmt = conn.prepare(
            &format!(
                r#"
                SELECT {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}
                FROM {}
                ORDER BY {} DESC
                "#,
                schema::COL_ID,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_DEF_START_TS,
                schema::COL_DEF_END_TS,
                schema::COL_START_TS,
                schema::COL_END_TS,
                schema::COL_PROCESS_PATH,
                schema::COL_TAGS,
                schema::COL_CRON,
                schema::COL_COLOR,
                schema::TABLE,
                schema::COL_START_TS
            )
        )?;

        let jobs = stmt.query_map([], |row| {
            let proccess_path_str: String = row.get(7)?;
            let proccess_path: Vec<Option<i64>> = proccess_path_str
                .split(',')
                .filter(|s| !s.is_empty() && *s != "None")
                .map(|s| s.trim().parse::<i64>().ok())
                .collect();
            let tags_raw: Option<String> = row.get(8)?;

            Ok(JobModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                def_start_ts: row.get(3)?,
                def_end_ts: row.get(4)?,
                start_ts: row.get(5)?,
                end_ts: row.get(6)?,
                proccess_path,
                tags: parse_tags_json(tags_raw),
                cron: row.get(9)?,
                color: row.get(10)?,
            })
        })?;

        Ok(jobs.filter_map(Result::ok).collect())
    }

    /// Insert a new job
    pub fn insert(conn: &Connection, job: &JobModel) -> Result<i64> {
        let process_paths = job
            .proccess_path
            .iter()
            .fold(String::new(), |acc, e| format!("{acc},{e:?}"));
        let tags = tags_to_json(&job.tags);

        conn.execute(
            &format!(
                r#"
                INSERT INTO {} (
                    {}, {}, {}, {}, {}, {}, {}, {}, {}, {}
                )
                VALUES (
                    :name, :description, :def_start_ts,
                    :def_end_ts, :start_ts, :end_ts,
                    :proccess_path, :tags, :cron, :color
                )
                "#,
                schema::TABLE,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_DEF_START_TS,
                schema::COL_DEF_END_TS,
                schema::COL_START_TS,
                schema::COL_END_TS,
                schema::COL_PROCESS_PATH,
                schema::COL_TAGS,
                schema::COL_CRON,
                schema::COL_COLOR
            ),
            rusqlite::named_params! {
                ":name": job.name,
                ":description": job.description,
                ":def_start_ts": job.def_start_ts,
                ":def_end_ts": job.def_end_ts,
                ":start_ts": job.start_ts,
                ":end_ts": job.end_ts,
                ":proccess_path": process_paths,
                ":tags": tags,
                ":cron": job.cron,
                ":color": job.color
            },
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Update an existing job
    pub fn update(conn: &Connection, job: &JobModel) -> Result<usize> {
        let process_paths = job
            .proccess_path
            .iter()
            .fold(String::new(), |acc, e| format!("{acc},{e:?}"));
        let tags = tags_to_json(&job.tags);

        let id = job.id.ok_or(rusqlite::Error::QueryReturnedNoRows)?;

        let rows = conn.execute(
            &format!(
                r#"
                UPDATE {} SET
                    {} = :name,
                    {} = :description,
                    {} = :def_start_ts,
                    {} = :def_end_ts,
                    {} = :start_ts,
                    {} = :end_ts,
                    {} = :proccess_path,
                    {} = :tags,
                    {} = :cron,
                    {} = :color
                WHERE {} = :id
                "#,
                schema::TABLE,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_DEF_START_TS,
                schema::COL_DEF_END_TS,
                schema::COL_START_TS,
                schema::COL_END_TS,
                schema::COL_PROCESS_PATH,
                schema::COL_TAGS,
                schema::COL_CRON,
                schema::COL_COLOR,
                schema::COL_ID
            ),
            rusqlite::named_params! {
                ":name": job.name,
                ":description": job.description,
                ":def_start_ts": job.def_start_ts,
                ":def_end_ts": job.def_end_ts,
                ":start_ts": job.start_ts,
                ":end_ts": job.end_ts,
                ":proccess_path": process_paths,
                ":tags": tags,
                ":cron": job.cron,
                ":color": job.color,
                ":id": id,
            },
        )?;

        if rows == 0 {
            Err(rusqlite::Error::QueryReturnedNoRows)
        } else {
            Ok(rows)
        }
    }

    /// Delete a job
    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        conn.execute(
            &format!("DELETE FROM {} WHERE {} = ?1", schema::TABLE, schema::COL_ID),
            [id],
        )?;
        Ok(())
    }
}

fn tags_to_json(tags: &[crate::core::TagModel]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
}

fn parse_tags_json(raw: Option<String>) -> Vec<crate::core::TagModel> {
    raw.and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_schema() {
        assert_eq!(schema::TABLE, "jobs");
    }
}
