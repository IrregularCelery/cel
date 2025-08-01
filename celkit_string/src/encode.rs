use celkit_core::internal::sys::*;

fn escape_text(input: &str) -> String {
    let mut output = String::new();

    for c in input.chars() {
        match c {
            '\x08' => output.push_str("\\b"), // Backspace                \b
            '\x0C' => output.push_str("\\f"), // Formfeed Page Break      \f
            '\n' => output.push_str("\\n"),   // Newline (Line Feed)      \n
            '\r' => output.push_str("\\r"),   // Carriage Return          \r
            '\t' => output.push_str("\\t"),   // Horizontal Tab           \t
            '\\' => output.push_str("\\\\"),  // Backslash                \\
            '"' => output.push_str("\\\""),   // Double quotation mark    \"
            c if c.is_control() => {
                output.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => output.push(c),
        }
    }

    output
}

/// Minified encoding (single-line)
mod mini {
    use crate::encode::escape_text;
    use celkit_core::internal::sys::*;
    use celkit_core::internal::{Number, Result, Value};

    pub struct Encoder {
        input: Value,
    }

    impl Encoder {
        pub fn new(input: Value) -> Self {
            Self { input }
        }

        pub fn encode(self) -> Result<String> {
            self.encode_value(&self.input)
        }

        fn encode_null(&self) -> Result<String> {
            Ok("null".to_string())
        }

        fn encode_boolean(&self, value: &bool) -> Result<String> {
            Ok(value.to_string())
        }

        fn encode_number(&self, value: &Number) -> Result<String> {
            Ok(value.to_string())
        }

        fn encode_text(&self, value: &String) -> Result<String> {
            Ok(format!("\"{}\"", escape_text(value)))
        }

        fn encode_array(&self, value: &Vec<Value>) -> Result<String> {
            let items: Result<Vec<String>> = value.iter().map(|v| self.encode_value(v)).collect();
            let items = items?;

            Ok(format!("[{}]", items.join(",")))
        }

        fn encode_tuple(&self, value: &Vec<Value>) -> Result<String> {
            let members: Result<Vec<String>> = value.iter().map(|v| self.encode_value(v)).collect();
            let members = members?;

            Ok(format!("({})", members.join(",")))
        }

        fn encode_object(&self, value: &BTreeMap<String, Value>) -> Result<String> {
            let entries: Result<Vec<String>> = value
                .iter()
                .map(|entry| {
                    Ok(format!(
                        "\"{}\":{}",
                        escape_text(entry.0),        // Entry key
                        self.encode_value(entry.1)?  // Entry value
                    ))
                })
                .collect();
            let entries = entries?;

            Ok(format!("{{{}}}", entries.join(",")))
        }

        fn encode_struct(&self, value: &BTreeMap<String, Value>) -> Result<String> {
            let fields: Result<Vec<String>> = value
                .iter()
                .map(|field| {
                    Ok(format!(
                        "{}={}",
                        field.0,                     // Field name
                        self.encode_value(field.1)?  // Field value
                    ))
                })
                .collect();
            let fields = fields?;

            Ok(format!("@({})", fields.join(",")))
        }

        fn encode_value(&self, value: &Value) -> Result<String> {
            match value {
                Value::Null => self.encode_null(),
                Value::Boolean(b) => self.encode_boolean(b),
                Value::Number(n) => self.encode_number(n),
                Value::Text(t) => self.encode_text(t),
                Value::Array(a) => self.encode_array(a),
                Value::Tuple(t) => self.encode_tuple(t),
                Value::Object(o) => self.encode_object(o),
                Value::Struct(_, s) => self.encode_struct(s),
            }
        }
    }
}

/// Prettified encoding (multi-line)
mod pretty {
    use crate::encode::escape_text;
    use celkit_core::internal::sys::*;
    use celkit_core::internal::{Number, Result, Value};

    pub struct Encoder {
        input: Value,
        indent_size: usize,
        max_line_length: usize,
        trailing_comma: bool,
    }

    impl Encoder {
        pub fn new(input: Value) -> Self {
            Self {
                input,
                indent_size: 2,
                max_line_length: 100,
                trailing_comma: true,
            }
        }

        pub fn indent_size(mut self, size: usize) -> Self {
            self.indent_size = size;

            self
        }

        pub fn max_line_length(mut self, length: usize) -> Self {
            self.max_line_length = length;

            self
        }

        pub fn trailing_comma(mut self, enabled: bool) -> Self {
            self.trailing_comma = enabled;

            self
        }

        pub fn encode(self) -> Result<String> {
            let depth = 0;

            self.encode_value(&self.input, depth)
        }

        fn indent(&self, level: usize) -> String {
            " ".repeat(level * self.indent_size)
        }

        fn encode_null(&self) -> Result<String> {
            Ok("null".to_string())
        }

        fn encode_boolean(&self, value: &bool) -> Result<String> {
            Ok(value.to_string())
        }

        fn encode_number(&self, value: &Number) -> Result<String> {
            Ok(value.to_string())
        }

        fn encode_text(&self, value: &String) -> Result<String> {
            Ok(format!("\"{}\"", escape_text(value)))
        }

        fn encode_array(&self, value: &Vec<Value>, depth: usize) -> Result<String> {
            if value.is_empty() {
                return Ok("[]".to_string());
            }

            let current_indent = self.indent(depth);
            let next_indent = self.indent(depth + 1);

            let mut items = Vec::new();
            let mut single_line_length = 0;
            let mut can_fit_single_line = true;

            single_line_length += 1; // Opening array character "["

            for (i, item) in value.iter().enumerate() {
                let encoded_item = self.encode_value(item, depth + 1)?;

                items.push(encoded_item);

                if can_fit_single_line {
                    single_line_length += items[i].len();

                    if i < value.len() - 1 {
                        single_line_length += 2; // Separator comma and space ", "
                    }
                }
            }

            if can_fit_single_line {
                single_line_length += 1; // Closing array character "]"

                // It's safe to assume this value is a child (nested) element if
                // the `depth` is non-zero. So, we add `1` to the length of the line
                // to account for a possible comma from the parent.
                let comma_allowance = if depth > 0 { 1 } else { 0 };

                // Check if it exceeded the line limit
                if current_indent.len() + single_line_length + comma_allowance
                    > self.max_line_length
                {
                    can_fit_single_line = false;
                }
            }

            if can_fit_single_line {
                return Ok(format!("[{}]", items.join(", ")));
            }

            let mut output = String::new();
            let mut current_line = next_indent.clone();
            let empty_line_len = next_indent.len();

            output.push_str("[\n");

            for (i, encoded_item) in items.into_iter().enumerate() {
                let mut formatted_item = encoded_item;

                if i < value.len() - 1 || self.trailing_comma {
                    formatted_item.push_str(", ");
                }

                // Check if this item would fit in the current line
                if current_line.len() + formatted_item.len() <= self.max_line_length
                    || current_line.len() <= empty_line_len
                {
                    current_line.push_str(&formatted_item);

                    continue;
                }

                // Current line has content and would exceed the limit, wrap to next line
                output.push_str(&current_line.trim_end());
                output.push('\n');

                current_line = format!("{}{}", next_indent, formatted_item);
            }

            // Add the last line if it has content
            if current_line.len() > empty_line_len {
                output.push_str(&current_line.trim_end());
                output.push('\n');
            }

            output.push_str(&current_indent);
            output.push(']');

            Ok(output)
        }

        fn encode_tuple(&self, value: &Vec<Value>, depth: usize) -> Result<String> {
            if value.is_empty() {
                return Ok("()".to_string());
            }

            let current_indent = self.indent(depth);
            let next_indent = self.indent(depth + 1);

            let mut members = Vec::new();
            let mut single_line_length = 0;
            let mut can_fit_single_line = true;

            single_line_length += 1; // Opening tuple character "("

            for (i, member) in value.iter().enumerate() {
                let encoded_member = self.encode_value(member, depth + 1)?;

                members.push(encoded_member);

                if can_fit_single_line {
                    single_line_length += members[i].len();

                    if i < value.len() - 1 {
                        single_line_length += 2; // Separator comma and space ", "
                    }
                }
            }

            if can_fit_single_line {
                single_line_length += 1; // Closing tuple character ")"

                // It's safe to assume this value is a child (nested) element if
                // the `depth` is non-zero. So, we add `1` to the length of the line
                // to account for a possible comma from the parent.
                let comma_allowance = if depth > 0 { 1 } else { 0 };

                // Check if it exceeded the line limit
                if current_indent.len() + single_line_length + comma_allowance
                    > self.max_line_length
                {
                    can_fit_single_line = false;
                }
            }

            if can_fit_single_line {
                return Ok(format!("({})", members.join(", ")));
            }

            let mut output = String::new();
            let mut current_line = next_indent.clone();
            let empty_line_len = next_indent.len();

            output.push_str("(\n");

            for (i, encoded_member) in members.into_iter().enumerate() {
                let mut formatted_member = encoded_member;

                if i < value.len() - 1 || self.trailing_comma {
                    formatted_member.push_str(", ");
                }

                // Check if this member would fit in the current line
                if current_line.len() + formatted_member.len() <= self.max_line_length
                    || current_line.len() <= empty_line_len
                {
                    current_line.push_str(&formatted_member);

                    continue;
                }

                // Current line has content and would exceed the limit, wrap to next line
                output.push_str(&current_line.trim_end());
                output.push('\n');

                current_line = format!("{}{}", next_indent, formatted_member);
            }

            // Add the last line if it has content
            if current_line.len() > empty_line_len {
                output.push_str(&current_line.trim_end());
                output.push('\n');
            }

            output.push_str(&current_indent);
            output.push(')');

            Ok(output)
        }

        fn encode_object(&self, value: &BTreeMap<String, Value>, depth: usize) -> Result<String> {
            if value.is_empty() {
                return Ok("{}".to_string());
            }

            let current_indent = self.indent(depth);
            let next_indent = self.indent(depth + 1);

            let mut entries = Vec::new();
            let mut single_line_length = 0;
            let mut can_fit_single_line = true;

            single_line_length += 1; // Opening object character "{"

            for (i, entry) in value.iter().enumerate() {
                let encoded_entry = format!(
                    "\"{}\": {}",
                    escape_text(entry.0),                   // Entry key
                    self.encode_value(entry.1, depth + 1)?  // Entry value
                );

                entries.push(encoded_entry);

                if can_fit_single_line {
                    single_line_length += entries[i].len();

                    if i < value.len() - 1 {
                        single_line_length += 2; // Separator comma and space ", "
                    }
                }
            }

            if can_fit_single_line {
                single_line_length += 1; // Closing object character "}"

                // It's safe to assume this value is a child (nested) element if
                // the `depth` is non-zero. So, we add `1` to the length of the line
                // to account for a possible comma from the parent.
                let comma_allowance = if depth > 0 { 1 } else { 0 };

                // Check if it exceeded the line limit
                if current_indent.len() + single_line_length + comma_allowance
                    > self.max_line_length
                {
                    can_fit_single_line = false;
                }
            }

            if can_fit_single_line {
                return Ok(format!("{{{}}}", entries.join(", ")));
            }

            let mut output = String::new();
            let mut current_line = next_indent.clone();
            let empty_line_len = next_indent.len();

            output.push_str("{\n");

            for (i, encoded_entry) in entries.into_iter().enumerate() {
                let mut formatted_entry = encoded_entry;

                if i < value.len() - 1 || self.trailing_comma {
                    formatted_entry.push_str(", ");
                }

                // Check if this entry would fit in the current line
                if current_line.len() + formatted_entry.len() <= self.max_line_length
                    || current_line.len() <= empty_line_len
                {
                    current_line.push_str(&formatted_entry);

                    continue;
                }

                // Current line has content and would exceed the limit, wrap to next line
                output.push_str(&current_line.trim_end());
                output.push('\n');

                current_line = format!("{}{}", next_indent, formatted_entry);
            }

            // Add the last line if it has content
            if current_line.len() > empty_line_len {
                output.push_str(&current_line.trim_end());
                output.push('\n');
            }

            output.push_str(&current_indent);
            output.push('}');

            Ok(output)
        }

        fn encode_struct(&self, value: &BTreeMap<String, Value>, depth: usize) -> Result<String> {
            if value.is_empty() {
                return Ok("@()".to_string());
            }

            let current_indent = self.indent(depth);
            let next_indent = self.indent(depth + 1);

            let fields: Result<Vec<String>> = value
                .iter()
                .map(|field| {
                    Ok(format!(
                        "{} = {}",
                        field.0,                                // Field name
                        self.encode_value(field.1, depth + 1)?  // Field value
                    ))
                })
                .collect();
            let fields = fields?;

            let mut output = String::new();
            let mut current_line = next_indent.clone();
            let empty_line_len = next_indent.len();

            output.push_str("@(");

            for (i, encoded_field) in fields.into_iter().enumerate() {
                let mut formatted_field = encoded_field;

                if i < value.len() - 1 || self.trailing_comma {
                    formatted_field.push_str(", ");
                }

                // Each field has its own line
                output.push_str(&current_line.trim_end());
                output.push('\n');

                current_line = format!("{}{}", next_indent, formatted_field);
            }

            // Add the last line if it has content
            if current_line.len() > empty_line_len {
                output.push_str(&current_line.trim_end());
                output.push('\n');
            }

            output.push_str(&current_indent);
            output.push(')');

            Ok(output)
        }

        fn encode_value(&self, value: &Value, depth: usize) -> Result<String> {
            match value {
                Value::Null => self.encode_null(),
                Value::Boolean(b) => self.encode_boolean(b),
                Value::Number(n) => self.encode_number(n),
                Value::Text(t) => self.encode_text(t),
                Value::Array(a) => self.encode_array(a, depth),
                Value::Tuple(t) => self.encode_tuple(t, depth),
                Value::Object(o) => self.encode_object(o, depth),
                Value::Struct(_, s) => self.encode_struct(s, depth),
            }
        }
    }
}

pub fn to_string<T: ?Sized + celkit_core::Serialize>(
    value: &T,
) -> celkit_core::internal::Result<String> {
    let serialized = value.serialize()?;

    pretty::Encoder::new(serialized).encode()
}

pub fn to_mini<T: ?Sized + celkit_core::Serialize>(
    value: &T,
) -> celkit_core::internal::Result<mini::Encoder> {
    let serialized = value.serialize()?;

    Ok(mini::Encoder::new(serialized))
}

pub fn to_pretty<T: ?Sized + celkit_core::Serialize>(
    value: &T,
) -> celkit_core::internal::Result<pretty::Encoder> {
    let serialized = value.serialize()?;

    Ok(pretty::Encoder::new(serialized))
}
