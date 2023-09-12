use std::io::{self, Lines, StdinLock, Write};

use sqlite::Result;
use tabled::{
    builder::{self, Builder},
    settings::Style,
};

struct Prompt<'a>(Lines<StdinLock<'a>>);

impl<'a> Prompt<'a> {
    fn get(&mut self) -> String {
        self.next().unwrap()
    }
}

impl<'a> Default for Prompt<'a> {
    fn default() -> Self {
        Self(io::stdin().lines())
    }
}

impl<'a> Iterator for Prompt<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        print!("> ");
        io::stdout().flush().unwrap();
        self.0.next().unwrap().unwrap().into()
    }
}

fn main() -> Result<()> {
    let mut prompt = Prompt::default();

    println!("DB file (leave blank to keep db alive only in memory):");
    let db_file = match prompt.get().as_str() {
        "" => ":memory:".to_string(),
        o => o.to_string(),
    };

    let connection = sqlite::open(&db_file)?;
    println!("Opened DB \"{db_file}\"\n");

    for query in prompt {
        let mut prep = match connection.prepare(query) {
            Ok(x) => x,
            Err(e) => {
                println!("ERROR: {e}");
                continue;
            }
        };

        let titles = prep.column_names().to_owned();

        let rows = prep
            .iter()
            .flatten()
            .map(|row| {
                Vec::from(row)
                    .iter()
                    .map(|cell| {
                        use sqlite::Value as E;
                        match cell {
                            E::Binary(_) => "BINARY DATA".to_string(),
                            E::Float(x) => x.to_string(),
                            E::Integer(x) => x.to_string(),
                            E::Null => "NULL".to_string(),
                            E::String(x) => x.to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        if rows.len() > 0 {
            let mut builder = Builder::default();

            builder.set_header(titles.iter());

            for row in rows {
                builder.push_record(row.iter());
            }

            let table = builder.build().with(Style::sharp()).to_string();

            println!("{table}");
        } else {
            println!("No Data");
        }
    }

    Ok(())
}
