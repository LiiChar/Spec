/// Goal Repository
/// Manages goal records

use rusqlite::{Connection, Result};
use crate::core::{GoalModel, GoalOrder};
use super::super::schema::goals as schema;

pub struct GoalRepository;

impl GoalRepository {
    /// Get all goals
    pub fn get_all(conn: &Connection) -> Result<Vec<GoalModel>> {
        let mut stmt = conn.prepare(
            &format!(
                r#"
                SELECT {}, {}, {}, {}, {}, {}, {}, {}, {}, {}
                FROM {}
                ORDER BY {} DESC
                "#,
                schema::COL_ID,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_ORDERING,
                schema::COL_TIMESTAMP,
                schema::COL_START_PERIOD_TS,
                schema::COL_END_PERIOD_TS,
                schema::COL_PROCESS,
                schema::COL_TAGS,
                schema::COL_COMPLETED,
                schema::TABLE,
                schema::COL_TIMESTAMP
            )
        )?;

        let goals = stmt.query_map([], |row| {
            let tags_raw: Option<String> = row.get(8)?;
            Ok(GoalModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                ordering: GoalOrder::from_i32(row.get(3)?),
                timestamp: row.get(4)?,
                start_period_ts: row.get(5)?,
                end_period_ts: row.get(6)?,
                process: row.get(7)?,
                tags: parse_tags_json(tags_raw),
                completed: row.get::<_, i32>(9)? != 0,
            })
        })?;

        Ok(goals.filter_map(Result::ok).collect())
    }

    /// Insert a new goal
    pub fn insert(conn: &Connection, goal: &GoalModel) -> Result<i64> {
        let tags = tags_to_json(&goal.tags);
        conn.execute(
            &format!(
                r#"
                INSERT INTO {} ({}, {}, {}, {}, {}, {}, {}, {}, {})
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                schema::TABLE,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_ORDERING,
                schema::COL_TIMESTAMP,
                schema::COL_START_PERIOD_TS,
                schema::COL_END_PERIOD_TS,
                schema::COL_PROCESS,
                schema::COL_TAGS,
                schema::COL_COMPLETED
            ),
            (
                &goal.name,
                &goal.description,
                goal.ordering.as_i32(),
                goal.timestamp,
                goal.start_period_ts,
                goal.end_period_ts,
                &goal.process,
                tags,
                goal.completed as i32,
            ),
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Update an existing goal
    pub fn update(conn: &Connection, goal: &GoalModel) -> Result<()> {
        let id = goal.id.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let tags = tags_to_json(&goal.tags);
        conn.execute(
            &format!(
                r#"
                UPDATE {} SET
                    {} = ?1,
                    {} = ?2,
                    {} = ?3,
                    {} = ?4,
                    {} = ?5,
                    {} = ?6,
                    {} = ?7,
                    {} = ?8,
                    {} = ?9
                WHERE {} = ?10
                "#,
                schema::TABLE,
                schema::COL_NAME,
                schema::COL_DESCRIPTION,
                schema::COL_ORDERING,
                schema::COL_TIMESTAMP,
                schema::COL_START_PERIOD_TS,
                schema::COL_END_PERIOD_TS,
                schema::COL_PROCESS,
                schema::COL_TAGS,
                schema::COL_COMPLETED,
                schema::COL_ID
            ),
            (
                &goal.name,
                &goal.description,
                goal.ordering.as_i32(),
                goal.timestamp,
                goal.start_period_ts,
                goal.end_period_ts,
                &goal.process,
                tags,
                goal.completed as i32,
                id,
            ),
        )?;
        Ok(())
    }

    /// Delete a goal
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
    fn test_goal_schema() {
        assert_eq!(schema::TABLE, "goals");
    }
}
