use regex::Regex;
/// Tag Repository
/// Manages tags and their associations with windows

use rusqlite::{Connection, Result};
use crate::core::{EventModel, TagModel};
use super::super::schema::{tag as tag_schema, tag_to_window as t2w_schema};

pub struct TagRepository;

impl TagRepository {
    /// Get all tags
    pub fn get_all(conn: &Connection) -> Result<Vec<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {}, {}, {}, {}, {} FROM {}",
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::COL_FILTER,
                tag_schema::TABLE
            )
        )?;

        let tags = stmt.query_map([], |row| {
            Ok(TagModel {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                color: row.get(3)?,
                filter: row.get(4)?,
            })
        })?;

        Ok(tags.filter_map(Result::ok).collect())
    }

    /// Get tag by name
    pub fn get_by_name(conn: &Connection, name: &str) -> Result<Option<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                "SELECT {}, {}, {}, {}, {} FROM {} WHERE {} = ?1",
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::COL_FILTER,
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
                filter: row.get(4)?,
            })
        }) {
            Ok(tag) => Ok(Some(tag)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn ensure(conn: &Connection, name: &str, color: &str, filter: Option<String>) -> Result<TagModel> {
        if let Some(tag) = Self::get_by_name(conn, name)? {
            return Ok(tag);
        }

        conn.execute(
            &format!(
                "INSERT INTO {} ({}, {}, {}) VALUES (?1, ?2, ?3)",
                tag_schema::TABLE,
                tag_schema::COL_NAME,
                tag_schema::COL_COLOR,
                tag_schema::COL_FILTER
            ),
            rusqlite::params![name, color, filter.clone()],
        )?;

        Ok(TagModel {
            id: Some(conn.last_insert_rowid()),
            name: name.to_string(),
            description: None,
            color: color.to_string(),
            filter,
        })
    }

    /// Get tags for a window (by process name)
    pub fn get_for_process(conn: &Connection, process_name: &str) -> Result<Vec<TagModel>> {
        let mut stmt = conn.prepare(
            &format!(
                r#"
                SELECT t.{}, t.{}, t.{}, t.{}, t.{}
                FROM {} w
                JOIN {} t ON t.{} = w.{}
                WHERE LOWER(TRIM(w.{})) = LOWER(TRIM(?1))
                "#,
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::COL_FILTER,
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
                filter: row.get(4)?,
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
        let tag = Self::ensure(conn, tag_name, "#94a3b8", None)?;
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
                        "INSERT INTO {} ({}, {}, {}, {}) VALUES (?1, ?2, ?3)",
                        tag_schema::TABLE,
                        tag_schema::COL_NAME,
                        tag_schema::COL_DESCRIPTION,
                        tag_schema::COL_COLOR,
                        tag_schema::COL_FILTER
                    ),
                    (&t.name, &t.description, &t.color, &t.filter),
                )?;
                inserted += 1;
            }
        }
        Ok(inserted)
    }

    pub fn update(conn: &Connection, tag: &TagModel) -> Result<usize> {
        let mut stmt = conn.prepare(
            &format!(
                "UPDATE {} SET {} = ?, {}, {} = ?, {} = ? WHERE {} = ?1 and {} = ?2",
                tag_schema::TABLE,
                tag_schema::COL_NAME,
                tag_schema::COL_DESCRIPTION,
                tag_schema::COL_COLOR,
                tag_schema::COL_FILTER,
                tag_schema::COL_ID,
                tag_schema::COL_NAME,
            ),
        )?;

        let id = stmt.execute(
            &[
                &tag.name,
                &tag.description.clone().unwrap_or("".to_string()),
                &tag.color,
                &tag.filter.clone().unwrap_or("".to_string()),
                &tag.id.unwrap_or(-1).to_string(),
                &tag.name,

            ],
        )?;

        Ok(id)
    }

    pub fn delete(conn: &Connection, id: i64) -> Result<()> {
        let mut stmt = conn.prepare(
            &format!(
                "DELETE FROM {} WHERE {} = ?1",
                tag_schema::TABLE,
                tag_schema::COL_ID
            ),
        )?;

        stmt.execute(&[&id])?;

        Ok(())
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

pub fn attach_tags(
    conn: &Connection,
    events: Vec<EventModel>,
) -> Vec<EventModel> {
    events
        .into_iter()
        .map(|mut event| {
            if let Some(window) = &mut event.window {
                window.tags = TagRepository::get_for_process(
                    conn,
                    &window.process_name,
                )
                .unwrap_or_default();
            }

            event
        })
        .collect()
}

pub fn auto_tag(
    conn: &Connection,
    tags: &[TagModel],
    mut event: EventModel,
) -> Result<EventModel, String> {
    let Some(window) = &mut event.window else {
        return Ok(event);
    };

    let process_name = window.process_name.as_str();

    for tag in tags {
        let Some(filter) = &tag.filter else {
            continue;
        };

        let regex = Regex::new(filter)
            .map_err(|e| format!("Invalid regex '{}': {}", filter, e))?;

        if regex.is_match(process_name) {
            // Не добавляем тег повторно
            if !window.tags.iter().any(|t| t.id == tag.id) {
                let exec = conn.execute(
                    &format!(
                        "INSERT INTO {} ({}, {}) VALUES (?1, ?2)",
                        t2w_schema::TABLE,
                        t2w_schema::COL_TAG_ID,
                        t2w_schema::COL_PROCESS_NAME
                    ),
                    (&tag.id, process_name),
                );
                match exec{
                    Ok(_) => window.tags.push(tag.clone()),
                    Err(_) => continue,
                };
            }
        }
    }

    Ok(event)
}