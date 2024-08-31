use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tera::{Tera, Context};

#[derive(Serialize, Deserialize)]
struct Story {
    title: String,
    text: String,
    images: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Puzzle {
    title: String,
    text: String,
    images: Vec<String>,
    hints: Vec<String>,
    answer: String,
}

#[derive(Serialize, Deserialize)]
struct GameData {
    story: Story,
    puzzles: Vec<Puzzle>,
}

async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let data = fs::read_to_string("data.json").expect("Unable to read file");
    let game_data: GameData = serde_json::from_str(&data).expect("Unable to parse JSON");

    let mut ctx = Context::new();
    ctx.insert("story", &game_data.story);

    let rendered = tmpl.render("index.html.tera", &ctx).unwrap_or_else(|err| {
        format!("Error rendering template: {}", err)
    });

    HttpResponse::Ok().content_type("text/html").body(rendered)
}

async fn question(tmpl: web::Data<Tera>, path: web::Path<usize>) -> impl Responder {
    let puzzle_index = path.into_inner();

    let data = fs::read_to_string("data.json").expect("Unable to read file");
    let game_data: GameData = serde_json::from_str(&data).expect("Unable to parse JSON");

    if puzzle_index < game_data.puzzles.len() {
        let mut ctx = Context::new();
        ctx.insert("puzzle", &game_data.puzzles[puzzle_index]);
        ctx.insert("puzzle_index", &puzzle_index);
        ctx.insert("error", &"");

        let rendered = tmpl.render("question.html.tera", &ctx).unwrap_or_else(|err| {
            format!("Error rendering template: {}", err)
        });

        HttpResponse::Ok().content_type("text/html").body(rendered)
    } else {
        HttpResponse::BadRequest().body("Ungültige Puzzle-ID")
    }
}

async fn check_answer(
    tmpl: web::Data<Tera>,
    path: web::Path<usize>,
    form: web::Form<HashMap<String, String>>,
) -> impl Responder {
    let puzzle_index = path.into_inner();
    let answer = form.get("answer").unwrap_or(&String::new()).trim().to_string();

    let data = fs::read_to_string("data.json").expect("Unable to read file");
    let game_data: GameData = serde_json::from_str(&data).expect("Unable to parse JSON");

    if puzzle_index < game_data.puzzles.len() {
        if answer.eq_ignore_ascii_case(&game_data.puzzles[puzzle_index].answer) {
            // Weiter zur nächsten Frage
            if puzzle_index + 1 < game_data.puzzles.len() {
                HttpResponse::Found()
                    .append_header(("Location", format!("/question/{}", puzzle_index + 1)))
                    .finish()
            } else {
                // Alle Fragen richtig beantwortet, Gewinner-Seite anzeigen
                let rendered = tmpl.render("winner.html.tera", &Context::new()).unwrap_or_else(|err| {
                    format!("Error rendering template: {}", err)
                });
                HttpResponse::Ok().content_type("text/html").body(rendered)
            }
        } else {
            // Fehlernachricht anzeigen
            let mut ctx = Context::new();
            ctx.insert("puzzle", &game_data.puzzles[puzzle_index]);
            ctx.insert("puzzle_index", &puzzle_index);
            ctx.insert("error", &"Falsche Antwort. Bitte versuche es erneut!");

            let rendered = tmpl.render("question.html.tera", &ctx).unwrap_or_else(|err| {
                format!("Error rendering template: {}", err)
            });

            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
    } else {
        HttpResponse::BadRequest().body("Ungültige Puzzle-ID")
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
            .route("/question/{id}", web::get().to(question))
            .route("/check_answer/{id}", web::post().to(check_answer))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}