#[allow(dead_code)]
pub struct OutputFormatter;

impl OutputFormatter {
    #[allow(dead_code)]
    pub fn table(headers: &[&str], rows: &[Vec<String>]) {
        // Calculate column widths
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print header
        for (i, header) in headers.iter().enumerate() {
            if i > 0 {
                print!("  ");
            }
            print!("{:<width$}", header, width = widths[i]);
        }
        println!();

        // Print separator
        for (i, width) in widths.iter().enumerate() {
            if i > 0 {
                print!("  ");
            }
            print!("{}", "-".repeat(*width));
        }
        println!();

        // Print rows
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    print!("  ");
                }
                print!("{:<width$}", cell, width = widths[i]);
            }
            println!();
        }
    }

    #[allow(dead_code)]
    pub fn json<T: serde::Serialize>(data: &T) -> anyhow::Result<()> {
        println!("{}", serde_json::to_string_pretty(data)?);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn yaml<T: serde::Serialize>(data: &T) -> anyhow::Result<()> {
        println!("{}", serde_yaml::to_string(data)?);
        Ok(())
    }
}
