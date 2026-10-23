use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;
use std::io;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

mod mod_model;
mod mod_handle;

const DB_PATH: &str = "./vec.db";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdataReq {
    pub table: String,
    pub sql: String,
}

// アプリケーションの状態
type AppState = Arc<Mutex<Connection>>;

#[tokio::main]
async fn main() {
    // SQLiteデータベースの初期化
    let conn = Connection::open(DB_PATH).expect("Failed to open database");
    
    let state = Arc::new(Mutex::new(conn));

    // ルーティングの設定
    let app = Router::new()
        .route("/api/insert", post(update))
        .route("/api/select", post(mod_handle::select_handle))
        .with_state(state);

    // サーバーの起動
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888")
        .await
        .expect("Failed to bind");
    
    println!("Server running on http://localhost:8888");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

async fn update(
    State(state): State<AppState>, 
    Json(payload): Json<mod_model::InsertItem>
) -> (StatusCode, String) {
    let conn = state.lock().unwrap();
    println!("content={}", &payload.content);    
    println!("table={}", &payload.table); 
    println!("vector.len={}", &payload.vector.len());    
    let res1: &String = &payload.vector.iter() .map(|v| v.to_string()) .collect::<Vec<String>>() .join(", ");
    //println!("{}", res1);    
    let str2 = format!("[{}]", res1 );       
    //println!("str2={}", &str2);
    let uuid_value = Uuid::new_v4();
    let idStr = format!("{}", uuid_value);
    println!("idStr={}", idStr);

    let mut stmt_v1 = conn
        .prepare("SELECT id , content , embeddings FROM document LIMIT 1;")
        .unwrap();
    let item_v1: Vec<_>  = stmt_v1
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let content: Option<String> = row.get(1)?;            
            let vector_str: Option<String> = row.get(2)?;
            let vector: Vec<f64> = vector_str.expect("REASON").trim_matches(|c| c == '[' || c == ']') 
            .split(',')
            .map(|s| s.trim().parse::<f64>().unwrap()) 
            .collect(); 
            //println!("{:?}", vector);
            let v1 :Vec<f64> = Vec::new();
            Ok((id, content, vector))
        })
        .unwrap()
        .filter_map(|t| t.ok())
        .collect();
    let mut vec2: Vec<mod_model::SelectDbItem> = Vec::new();
    for row in item_v1 {
        let (id, content, vector) = row;
        println!("id={}" , &id); 
        println!("vector={:?}" , vector); 
        let row2 = mod_model::SelectDbItem{
            id: id,
            content: content.expect("REASON"),
            vector: vector,
        };
        vec2.push(row2);        
    }
    if vec2.len() > 0 {
        let target_row = vec2[0].vector.clone();
        println!("target_row.len={}", target_row.len());
        let input_len = &payload.vector.len();
        println!("input_len={}", input_len);
        if *input_len != target_row.len() {
            println!("error, valid_vector_len = NG");
            return (StatusCode::BAD_REQUEST, "NG".to_string());
        }
    } 
        
    conn.execute(
        "INSERT INTO document (id , content, embeddings) VALUES (?1, ?2, ?3)",
        params![&idStr ,&payload.content , &str2 ],
    )
    .unwrap();

    (StatusCode::CREATED, "OK".to_string())
}

