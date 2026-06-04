/// Tag Repository
/// Manages tags and their associations with windows

use rusqlite::{Connection, Result};
use crate::core::TagModel;
use super::super::schema::{tag as tag_schema, tag_to_window as t2w_schema};

pub struct TagRepository;

impl TagRepository {
    /// Get all tags
    pub fn get_all(conn: &Connection) -> Result<Vec<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {}, {}, {}, {} FROM {}",
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::TABLE
            )
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(TagModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
            })
        })?;

        Ok(tags.filter_map(Result::ok).collect())
    }

    /// Get tag by name
    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {}, {}, {}, {} FROM {} WHERE {} = ?1",
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::TABLE,
                tag_schema::COL_NAME
            )
        )?;

        match stmt.query_row([name], |row| {
            Ok(TagModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
            })
        }) {
            Ok(tag) => Ok(Some(tag)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Create or get existing tag (idempotent)
    pub fn ensure(conn: &Connection, name: &str, color: &str) -> Result<TagModel> {
        if let Some(tag) = Self::get_by_name(conn, name)? {
            return Ok(tag);
        }

        conn.execute(
            &format!(
                "INSERT INTO {} ({}, {}, {}) VALUES (?1, NULL, ?2)",
                tag_schema::TABLE,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR
            ),
            (name, color),
        )?;

        Ok(TagModel {
            id: Some(conn.last_insert_rowid()),
            name: name.to_string(),
            description: None,
            color: color.to_string(),
        })
    }

    /// Get tags for a window (by process name)
    pub fn get_for_process(conn: &Connection, process_name: &str) -> Result<Vec<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                r#"
                SELECT t.{}, t.{}, t.{}, t.{}
                FROM {} w
                JOIN {} t ON t.{} = w.{}
                WHERE LOWER(TRIM(w.{})) = LOWER(TRIM(?1))
                "#,
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                t2w_schema::TABLE,
                tag_schema::TABLE,
                tag_schema::COL_ID,
                t2w_schema::COL_TAG_ID,
                t2w_schema::COL_PROCESS_NAME
            )
        )?;

        let tags = stmt.query_map([process_name], |row| {
            Ok(TagModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
            })
        })?;

        Ok(tags.filter_map(Result::ok).collect())
    }

    /// Add tag to window
    pub fn add_to_window(conn: &Connection, tag_id: i64, process_name: &str) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO {} ({}, {}) VALUES (?1, ?2)",
                t2w_schema::TABLE,
                t2w_schema::COL_TAG_ID,
                t2w_schema::COL_PROCESS_NAME
            ),
            (tag_id, process_name),
        )?;
        Ok(())
    }

    /// Check if tag is applied to window
    pub fn has_for_window(conn: &Connection, process_name: &str, tag_id: i64) -> Result<bool> {
        let count: i32 = conn.query_row(
            &format!(
                "SELECT COUNT(1) FROM {} WHERE LOWER(TRIM({})) = LOWER(TRIM(?1)) AND {} = ?2",
                t2w_schema::TABLE,
                t2w_schema::COL_PROCESS_NAME,
                t2w_schema::COL_TAG_ID
            ),
            (process_name, tag_id),
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Add tag to window if not already present
    pub fn add_to_window_if_missing(conn: &Connection, tag_name: &str, process_name: &str) -> Result<()> {
        let tag = Self::ensure(conn, tag_name, "#94a3b8")?;
        let tag_id = tag.id.ok_or(rusqlite::Error::QueryReturnedNoRows)?;

        if !Self::has_for_window(conn, process_name, tag_id)? {
            Self::add_to_window(conn, tag_id, process_name)?;
        }

        Ok(())
    }

    /// Merge tags into database
    pub fn merge_many(conn: &Connection, tags: &[TagModel]) -> Result<usize> {
        let mut inserted = 0;
        for t in tags {
            if !Self::get_by_name(conn, &t.name)?.is_some() {
                conn.execute(
                    &format!(
                        "INSERT INTO {} ({}, {}, {}) VALUES (?1, ?2, ?3)",
                        tag_schema::TABLE,
                        tag_schema::COL_NAME,
                        tag_schema::COL_DESCRIPTION,
                        tag_schema::COL_COLOR
                    ),
                    (&t.name, &t.description, &t.color),
                )?;
                inserted += 1;
            }
        }
        Ok(inserted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_schema() {
        assert_eq!(tag_schema::TABLE, "tag");
        assert_eq!(t2w_schema::TABLE, "tag_to_window");
    }
}
