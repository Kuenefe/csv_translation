use reqwest::Client;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::time::Duration;
use tokio::task;
use std::thread;

#[derive(Debug, serde::Deserialize, Clone)]
struct CsvRow {
    document_name: String,
    text_german: Option<String>,
    text_original: String,
    page_number: usize,
    comment: Option<String>,
}

fn read_csv(csv_path: &str) -> Result<Vec<CsvRow>, Box<dyn Error>> {
    let file = File::open(csv_path)?;
    let mut csv_rows = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(file);

    for result in rdr.deserialize() {
        let record: CsvRow = result?;
        csv_rows.push(record);
    }

    Ok(csv_rows)
}

async fn translate_csv_data(mut csv_data: Vec<CsvRow>) -> Result<Vec<CsvRow>, Box<dyn Error>> {
    let url = "http://localhost:5000/translate";
    let client = Client::new();
    let mut tasks = Vec::with_capacity(csv_data.len());

    for row in csv_data.iter() {
        let text_to_translate = row.text_original.clone();
        let client_clone = client.clone();
        let url_clone = url.to_string();

        let task = task::spawn(async move {
            let res = client_clone
                .post(&url_clone)
                .form(&[
                    ("q", &text_to_translate),
                    ("source", &"en".to_string()),
                    ("target", &"de".to_string()),
                ])
                .send()
                .await;

            if let Ok(response) = res {
                let text = response
                    .text()
                    .await
                    .ok()
                    .unwrap_or_else(|| "funzt net".to_string());
                Some(text)
            } else {
                None
            }
        });

        tasks.push(task);
    }

    for (i, task) in tasks.into_iter().enumerate() {
        if let Ok(Some(translated_text)) = task.await {
            csv_data[i].text_german = Some(translated_text);
        }
    }

    Ok(csv_data)
}

#[tokio::main]
async fn main() {
    loop {
        println!("Bitte geben Sie den Pfad der CSV-Datei zur Übersetzung an ('q' zum Beenden):");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut csv_file_path = String::new();
        io::stdin()
            .read_line(&mut csv_file_path)
            .expect("Konnte den Pfad nicht einlesen!");

        let csv_file_path = csv_file_path.trim();

        if csv_file_path.eq_ignore_ascii_case("q") {
            println!("Programm wird beendet in");
            println!("3..");
            thread::sleep(Duration::from_millis(300));
            println!("2..");
            thread::sleep(Duration::from_millis(300));
            println!("1..");
            thread::sleep(Duration::from_millis(300));
            println!("0..");
            thread::sleep(Duration::from_millis(300));
            println!("Good bye");
            thread::sleep(Duration::from_millis(200));
            break;
        }

        match read_csv(csv_file_path) {
            Ok(csv_data) => {
                match translate_csv_data(csv_data).await {
                    Ok(translated_data) => {
                        println!("Übersetzte Daten:\n {:?}", translated_data);
                        //break;
                    }
                    Err(e) => println!("Es gab ein Problem bei der Übersetzung: {}", e),
                }
            }
            Err(e) => {
                println!("Fehler beim Einlesen der CSV-Datei: {}. Bitte versuchen Sie es erneut.", e);
            }
        }
    }
}
