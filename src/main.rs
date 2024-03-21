
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::{Path};
use chrono::NaiveDateTime;
use walkdir::WalkDir;
use reqwest::Client;
use serde_json::json;
use data_encoding::BASE64;
use sqlx::PgPool;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the database
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

    // Create photos table
    create_photos_table(&pool).await?;

    // UPLOAD FLOW
    // get folder path from command line arguments
    let folder_path = std::env::args().nth(1).unwrap_or_else(|| "./images".to_string());
    // Upload photos to the database
    upload_photos(&pool, &folder_path).await?;

    /*
    // SEARCH FLOW
    // Search photos by tags
    let query = "Give me pictures where I'm by the beach with my friends.";
    let photos = search_photos_by_tags(&pool, query).await?;
    for photo in photos {
        println!("Photo: {:?}", photo.file_path);
    }
    */


    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default()
        .to_lowercase();

    matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp")
}

async fn image_to_base64(path: &Path) -> Result<String, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer).expect("Failed to read file");

    let base64_encoded = BASE64.encode(&buffer);
    Ok(base64_encoded)
}

async fn create_photos_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    let query = r#"
        CREATE TABLE IF NOT EXISTS photos (
            photo_id SERIAL PRIMARY KEY,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            tags TEXT[],
            created_at TIMESTAMP DEFAULT NOW()
        )
    "#;

    sqlx::query(query)
        .execute(pool)
        .await?;

    Ok(())
}

async fn upload_photos(pool: &PgPool, directory: &str) -> Result<(), Box<dyn Error>> {
    let folder_path = std::env::args().nth(1).unwrap_or_else(|| "./images".to_string());
    let client = Client::new();

    for entry in WalkDir::new(&folder_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && is_image_file(path) {
            let base64_image = image_to_base64(path).await?;
            let prompt = "
You are an image tagging assistant. Your task is to analyze the given image and generate a comma-separated list of relevant tags or keywords that can be used to categorize and search for similar images in a database.

When generating tags, please follow these guidelines:

1. Use concise, descriptive words or short phrases that accurately describe the content of the image.
2. Avoid using full sentences or unnecessary words in the tags.
3. Include tags that describe the main subject(s), objects, scenes, activities, emotions, colors, and any other relevant aspects of the image.
4. Use plural forms for nouns when appropriate (e.g., \"trees\" instead of \"tree\").
5. Separate each tag with a comma and a space (e.g., \"nature, landscape, trees, mountain\").
6. Do not include any additional text or explanations beyond the comma-separated list of tags.

Please analyze the provided image and generate a list of relevant tags following the guidelines above.
";
            let payload = json!({
                "stream": false,
                "model": "llava",
                "prompt": prompt,
                "images": [base64_image]
            });

            let response = client
                .post("http://localhost:11434/api/generate")
                .json(&payload)
                .send()
                .await?;

            let response_json: serde_json::Value = response.json().await?;
            let response = response_json["response"].as_str().unwrap().trim();
            println!("Tags: {}", response);
            let tags: Vec<&str> = response.split(", ").collect();

            Photo::add_photo(
                &pool,
                path.file_name().unwrap().to_str().unwrap(),
                path.canonicalize().unwrap().to_str().unwrap(),
                tags,
            )
                .await?;

            println!("Added photo: {} ", path.file_name().unwrap().to_str().unwrap());
        }
    }
    Ok(())
}

// Given a query from user, send a request to get relavant tags from user's search sentence
async fn get_tags_from_search_query(query: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let client = Client::new();

    let prompt = format!(
        "You are a photo tagging assistant. Your task is to extract relevant tags from a given search query that can be used to search for photos in a database.

The search query will be provided to you, and you should respond with a comma-separated list of tags that best represent the query.

Here are some examples:

Search query: \"Give me pictures from sunny days\"
sunny, clear sky, daylight, outdoor, nature

Search query: \"Show me photos of cars on the street\"
cars, street, urban, transportation

Search query: \"I want to see images of beaches with palm trees\"
beach, palm trees, tropical, nature, coastline

Remember to keep the tags concise, relevant, and easy to search for in a database. Avoid using full sentences or unnecessary words in the tags. Only output data as comma-separated tags. Do not output anything else.

Search query: \"{}\"",
        query
    );

    let payload = json!({
        "stream": false,
        "model": "llama2",
        "prompt": prompt,
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&payload)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let response_text = response_json["response"].as_str().unwrap().trim();
    println!("Tags to search: {}", response_text);
    let tags: Vec<String> = response_text
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    Ok(tags)
}

async fn search_photos_by_tags(pool: &PgPool, query: &str) -> Result<Vec<Photo>, Box<dyn Error>> {
    // get tags from query
    let tags = get_tags_from_search_query(query).await?;
    // search photos by tags
    let photos = Photo::search_photos_by_tags(pool, tags).await?;
    Ok(photos)
}

#[derive(Debug, sqlx::FromRow)]
struct Photo {
    photo_id: i32,
    file_name: String,
    pub(crate) file_path: String,
    tags: Vec<String>,
    created_at: NaiveDateTime,
}

impl Photo {
    // Function to add a new photo to the database
    async fn add_photo(pool: &PgPool, file_name: &str, file_path: &str, tags: Vec<&str>) -> Result<(), sqlx::Error> {
        let tags_array = tags.into_iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let query = "INSERT INTO photos (file_name, file_path, tags) VALUES ($1, $2, $3)";
        let _ = sqlx::query(query)
            .bind(file_name)
            .bind(file_path)
            .bind(tags_array)
            .execute(pool)
            .await?;

        Ok(())
    }

    // Function to search for photos by tags
    async fn search_photos_by_tags(
        pool: &PgPool,
        search_tags: Vec<String>,
    ) -> Result<Vec<Photo>, sqlx::Error> {
        if search_tags.is_empty() {
            let query = "SELECT photo_id, file_name, file_path, tags, created_at FROM photos";
            sqlx::query_as::<_, Photo>(query)
                .fetch_all(pool)
                .await
        } else {
            let tags_query = search_tags
                .iter()
                .map(|tag| format!("'{}'", tag))
                .collect::<Vec<_>>()
                .join(", ");

            let query = format!(
                "
            SELECT p.photo_id, p.file_name, p.file_path, p.tags, p.created_at
            FROM photos p
            WHERE p.tags && ARRAY[{}]
        ",
                tags_query
            );

            sqlx::query_as::<_, Photo>(&query)
                .fetch_all(pool)
                .await
        }
    }
}