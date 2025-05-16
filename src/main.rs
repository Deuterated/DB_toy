use dotenv::dotenv;
use mysql::prelude::Queryable;
use mysql::*;
use std::env;
use std::fmt;
use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug)]
struct Config {
    pub database_url: String,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.database_url)
    }
}

fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        dotenv().ok(); // Load .env file
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        Config { database_url }
    })
}

fn mode_select(db_slice: &str) {
    let mut mode = String::new().to_lowercase();
    println!("Input, output, show, or exit?");
    std::io::stdin().read_line(&mut mode).expect("invalid");
    match mode.as_str().trim() {
        "input" => db_input(db_slice),
        "output" => db_output(db_slice),
        "show" => show_all(db_slice),
        "exit" => exeunt(),
        _ => {
            invalid_mode_selection(db_slice);
            Ok(())
        },
    }
    .expect("idk")
}

fn invalid_mode_selection(db_slice: &str) {
    println!("That isn't an option. Defaulting to showing all");
    show_all(db_slice).expect("line 48") //was on main for command-line testing; trying on show_all for SSI integration proof-of-concept
}

fn exeunt() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Goodbye");
    Ok(())
}

fn db_input(db_slice: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new(db_slice)?;
    let mut conn = pool.get_conn()?;
    println!("You have selected input ");
    println!("provide a post title:");
    let mut input_title_raw = String::new();
    std::io::stdin()
        .read_line(&mut input_title_raw)
        .expect("invalid");
    let input_title = input_title_raw.trim();
    println!("Post title: {}. Please provide some content:", input_title);
    let mut input_content_raw = String::new();
    std::io::stdin()
        .read_line(&mut input_content_raw)
        .expect("invalid");
    let input_content = input_content_raw.trim();
    let input_set = format!("'{}', '{}', current_date()", input_title, input_content);
    println!("input set: ({})", input_set);
    conn.exec_drop(
        "INSERT INTO posts (post_title, post_content, last_update) VALUES (?, ?, NOW())",
        (input_title, input_content),
    )?; //this was a bitch to get working, probably because I was tinkering for 3 hours befor I RTFM
    //let _ = main();
    Ok(())
}

fn db_output(db_slice: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new(db_slice)?;
    let mut conn = pool.get_conn()?;
    println!("you have selected output ");
    println!("provide a post ID number: ");
    let mut output_post = String::new();
    std::io::stdin()
        .read_line(&mut output_post)
        .expect("invalid");
    let result: Vec<(String, String)> = conn.exec(
        "SELECT post_title, post_content FROM posts WHERE post_id = ?",
        (output_post.trim(),),
    )?;
    if !(result.is_empty()) {
        for r in result {
            println!("Title: {}. Content: {}", r.0, r.1);
        }
    } else {
        println!("That post doesn't seem to exist");
    }
    //let _ = main();
    Ok(())
}

fn show_all(db_slice: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let pool = Pool::new(db_slice)?;
    let mut conn = pool.get_conn()?;

    println!("Executing query...");

    let result: Vec<Row> = conn.query("SELECT * FROM posts;")?;

    if result.is_empty() {
        println!("No posts found.");
    } else {
        for row in result {
            let post_id: i32 = row.get(0).ok_or("Missing post_id")?;
            let post_title: String = row.get(1).ok_or("Missing post_title")?;
            let post_content: String = row.get(2).ok_or("Missing post_content")?;
            let last_update_str: String = row.get(3).ok_or("Missing last_update")?;
            println!("ID: {}, Title: {} <br>", post_id, post_title);
            println!("Content: <br>");
            println!("{}", post_content);
            println!("Last Update: {} <br>", last_update_str);
        }
    }
    Ok(())
}

fn main() {
    let db_conn = get_config();
    let db_slice: &str = &db_conn.database_url.clone().to_string() as &str; //the one we actually pass around for the connections
    /*println!("direct from the get_config function: {:?}", db_conn);
    println!(
        "from the local 'db_slice' variable I'm passing around: {:?}",
        db_slice
    );
    println!(
        "the actual full text of the OnceLock (why ins't there a to_string method for this?): {:?}",
        CONFIG
    );*/ //this comment block is for local debugging
    mode_select(db_slice)
}
