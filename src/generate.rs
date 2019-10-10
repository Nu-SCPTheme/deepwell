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

use diesel::pg::PgConnection;
use diesel::result::Error;
use diesel::sql_types::*;
use diesel::{self, sql_query, RunQueryDsl};
use heck::CamelCase;
use std::mem;

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
    Date,
    Json,
}

impl DataType {
    fn from_pg_name(mut value: &str) -> Self {
        if value.starts_with("_") {
            value = &value[1..];
        }

        match value {
            "int2" | "int8" => DataType::Number,
            "text" => DataType::Text,
            "date" | "timestamp" => DataType::Date,
            "json" | "jsonb" => DataType::Json,
            _ => panic!("Unknown postgres type: {}", value),
        }
    }

    fn ts_type_name(self) -> &'static str {
        match self {
            DataType::Number => "number",
            DataType::Text => "string",
            DataType::Date => "Date",
            DataType::Json => "object",
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

        #[sql_type = "Bool"]
        is_nullable: bool,
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
        use diesel::pg::Pg;

        let columns: Vec<Column> = sql_query(COLUMN_LIST_QUERY)
            .bind::<Text, _>(&table.name)
            .load::<ColumnRow>(conn)?
            .into_iter()
            .map(|row| Column {
                name: row.column_name,
                data_type: DataType::from_pg_name(&row.udt_name),
                is_array: row.data_type == "ARRAY",
                is_nullable: row.is_nullable,
                has_default: row.column_default.is_some(),
            })
            .collect();

        mem::replace(&mut table.columns, columns);
    }

    Ok(tables)
}

pub fn typescript_interfaces(conn: &PgConnection) -> Result<String, Error> {
    let mut output = String::new();
    let tables = load_schema(conn)?;

    for table in &tables {
        output.push_str("export interface ");
        output.push_str(&table.name.to_camel_case());
        output.push_str(" {\n");

        for column in &table.columns {
            output.push_str("  ");
            output.push_str(&column.name);

            if column.has_default {
                output.push('?');
            }

            output.push_str(": ");
            output.push_str(column.data_type.ts_type_name());

            if column.is_array {
                output.push_str("[]");
            }

            output.push_str(";\n");
        }

        output.push_str("}\n\n");
    }

    Ok(output)
}
