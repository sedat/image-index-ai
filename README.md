Given a folder containing images, indexes them using llava to get relevant tags such as sunny, beach and blue sky for a photo by the beach in the summer. Saves each image name to the db with its tags.
Then you can use natural language like 'Give me photos by the beach in summer'. Llama converts your sentence to tags and tags are used to search the db for images with such tags. 

What do you need?

Rust, PostgreSQL and Ollama with llama and llava models.

Then just run it pointing to a folder with images `cargo run ./images/`
