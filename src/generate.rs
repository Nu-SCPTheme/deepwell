/*
 * generate.rs
 *
 * deepwell - Database management and migrations service
 * Copyright (C) 2019 Ammon Smith, not_a_seagull
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use askama::Template;
use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::sql_types::*;
use diesel::{self, sql_query, RunQueryDsl};
use heck::CamelCase;
use std::mem;

const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");

const TABLE_LIST_QUERY: &str = "
    SELECT table_name FROM information_schema.tables
        WHERE tables.table_schema = 'public'
        AND tables.table_name NOT IN ('__diesel_schema_migrations', 'schema_version')
        AND tables.table_type = 'BASE TABLE'
";

const COLUMN_LIST_QUERY: &str = "
    SELECT
        columns.column_name,
        columns.data_type,
        columns.udt_name,
        columns.column_default,
        columns.is_nullable
    FROM information_schema.columns
        WHERE columns.table_schema = 'public'
        AND columns.table_name = $1
";

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum DataType {
    Number,
    Text,
    Bytes,
    Date,
    Json,
    BitField,
}

impl DataType {
    fn from_pg_name(mut value: &str) -> Self {
        if value.starts_with("_") {
            value = &value[1..];
        }

        match value {
            "int2" | "int4" | "int8" => DataType::Number,
            "text" => DataType::Text,
            "bytea" => DataType::Bytes,
            "date" | "timestamp" => DataType::Date,
            "json" | "jsonb" => DataType::Json,
            "bit" => DataType::BitField,
            _ => panic!("Unknown postgres type: {}", value),
        }
    }

    fn ts_type_name(self) -> &'static str {
        match self {
            DataType::Number => "number",
            DataType::Text => "string",
            DataType::Bytes => "Buffer",
            DataType::Date => "Date",
            DataType::Json => "object",
            DataType::BitField => "number", // ?
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Table {
    name: String,
    columns: Vec<Column>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Column {
    name: String,
    data_type: DataType,
    is_array: bool,
    is_nullable: bool,
    has_default: bool,
}

fn load_schema(conn: &PgConnection) -> Result<Vec<Table>, Error> {
    #[derive(Debug, QueryableByName)]
    struct TableRow {
        #[sql_type = "Text"]
        table_name: String,
    }

    #[derive(Debug, QueryableByName)]
    struct ColumnRow {
        #[sql_type = "Text"]
        column_name: String,

        #[sql_type = "Text"]
        data_type: String,

        #[sql_type = "Text"]
        udt_name: String,

        #[sql_type = "Nullable<Text>"]
        column_default: Option<String>,

        #[sql_type = "Text"]
        is_nullable: String,
    }

    let mut tables: Vec<Table> = sql_query(TABLE_LIST_QUERY)
        .load::<TableRow>(conn)?
        .into_iter()
        .map(|row| Table {
            name: row.table_name,
            columns: vec![],
        })
        .collect();

    for table in tables.iter_mut() {
        let columns: Vec<Column> = sql_query(COLUMN_LIST_QUERY)
            .bind::<Text, _>(&table.name)
            .load::<ColumnRow>(conn)?
            .into_iter()
            .map(|row| Column {
                name: row.column_name,
                data_type: DataType::from_pg_name(&row.udt_name),
                is_array: row.data_type == "ARRAY",
                is_nullable: row.is_nullable == "YES",
                has_default: row.column_default.is_some(),
            })
            .collect();

        mem::replace(&mut table.columns, columns);
    }

    Ok(tables)
}

pub fn typescript_interfaces(conn: &PgConnection) -> Result<String, Error> {
    // Data definitions
    #[derive(Debug, Clone, Template)]
    #[template(path = "models.ts", escape = "none")]
    struct ModelsTemplate<'a> {
        version: &'static str,
        models: Vec<Model<'a>>,
    }

    #[derive(Debug, Clone)]
    struct Model<'a> {
        name: String,
        fields: &'a [Column],
    }

    // Used in template
    let _ = DataType::ts_type_name;

    // Setup render data
    let tables = load_schema(conn)?;
    let models = tables.iter().map(|table| {
        let table_name = {
            let mut name = table.name.to_camel_case();

            if name.ends_with('s') {
                name.pop();
            }

            name.push_str("Model");
            name
        };

        Model {
            name: table_name,
            fields: table.columns.as_slice(),
        }
    }).collect();

    let template = ModelsTemplate {
        version: &CARGO_VERSION,
        models,
    };

    let output = template.render().expect("Template rendering failed");
    Ok(output)
}
