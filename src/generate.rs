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
use diesel::{self, sql_query, RunQueryDsl};
use heck::CamelCase;

const TABLE_LIST_QUERY: &str = "
    SELECT table_name FROM information_schema.tables
        WHERE tables.table_schema = 'public'
        AND tables.table_name NOT IN ('__diesel_schema_migrations', 'schema_version')
        AND tables.table_type = 'BASE TABLE'
";

const COLUMN_LIST_QUERY: &str = "
    SELECT
        columns.column_name,
        columns.udt_name,
        columns.column_default,
        columns.is_nullable
    FROM information_schema.columns
        WHERE columns.table_schema = 'public'
        AND columns.table_name = ?
";

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
enum DataType {
    Number,
    Text,
    Date,
    Json,
}

impl DataType {
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
    has_default: bool,
}

fn load_schema(conn: &PgConnection) -> Result<Vec<Table>, Error> {
    /*
    let tables: Vec<_> = sql_query(TABLE_LIST_QUERY)
        .load(conn)?;
    */

    unimplemented!()
}

pub fn typescript_interfaces(conn: &PgConnection) -> Result<String, Error> {
    let mut output = String::new();
    let tables = load_schema(conn)?;

    for table in &tables {
        output.push_str("export interface ");
        output.push_str(&table.name.to_camel_case());
        output.push_str(" {");

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
