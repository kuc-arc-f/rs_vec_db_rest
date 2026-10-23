use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use rusqlite::{params, Connection};
use rusqlite::{types::ValueRef};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::fs;
use std::io;
use serde_json::{json, Map, Value};
use uuid::Uuid;


pub fn cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64, Box<dyn std::error::Error>> {
    if a.len() != b.len() {
        return Err("error , vectors must have the same length".into());
    }

    let mut dot_product = 0.0_f64;
    let mut a_magnitude = 0.0_f64;
    let mut b_magnitude = 0.0_f64;

    for i in 0..a.len() {
        dot_product += (a[i] * b[i]) as f64;
        a_magnitude += (a[i] * a[i]) as f64;
        b_magnitude += (b[i] * b[i]) as f64;
    }

    if a_magnitude == 0.0 || b_magnitude == 0.0 {
        return Ok(0.0);
    }

    Ok(dot_product / (a_magnitude.sqrt() * b_magnitude.sqrt()))
}

pub async fn select_handle(
    State(state): State<super::AppState>,
    Json(payload): Json<super::mod_model::SelectReq>,
) -> (StatusCode, String) {
    let conn = state.lock().unwrap();
    
    let table_name = &payload.table;
    let limit = &payload.limit;
    let vector = &payload.vector;
    //validate

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
    let mut vec2: Vec<super::mod_model::SelectDbItem> = Vec::new();
    for row in item_v1 {
        let (id, content, vector) = row;
        //println!("id={}" , &id); 
        //println!("vector={:?}" , vector); 
        let row2 = super::mod_model::SelectDbItem{
            id: id,
            content: content.expect("REASON"),
            vector: vector,
        };
        vec2.push(row2);        
    }
    if vec2.len() > 0 {
        let target_row = vec2[0].vector.clone();
        println!("target_row.len={}", target_row.len());
        let input_len = &vector.len();
        println!("input_len={}", input_len);
        if *input_len != target_row.len() {
            println!("error, valid_vector_len = NG");
            return (StatusCode::BAD_REQUEST, "NG".to_string());
        }
    }    

    let mut stmt = conn
        .prepare("SELECT id , content , embeddings FROM document;")
        .unwrap();
    
    let items: Vec<_>  = stmt
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
    let mut vec1: Vec<super::mod_model::SelectDbItem> = Vec::new();
    for row in items {
        let (id, content, vector) = row;
        //println!("id={}" , &id); 
        //println!("vector={:?}" , vector); 
        let row2 = super::mod_model::SelectDbItem{
            id: id,
            content: content.expect("REASON"),
            vector: vector,
        };
        vec1.push(row2);        
    }
    let mut scoreItems = Vec::<super::mod_model::DbValueItem>::new();
    for item in &vec1 {
        //println!("id={}", item.id);
        let distance = cosine_similarity(&vector, &item.vector).unwrap();
        if distance > 0.6 {
            scoreItems.push(super::mod_model::DbValueItem {
                id: item.id.clone(),
                content: item.content.clone(),
                distance: distance,
            });
        }
    }
    // score の降順ソート
    scoreItems.sort_by(|a, b| {
        b.distance
            .partial_cmp(&a.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });  
    let mut outItems = Vec::<super::mod_model::DbValueItem>::new(); 
    let top_k = limit;
    let mut outCount: i32 = 0;
    for row_item in &scoreItems {
        if outCount < *top_k {
            println!("id={}, distance={} \n", row_item.id, row_item.distance);
            outItems.push(super::mod_model::DbValueItem {
                id: row_item.id.clone(),
                content: row_item.content.clone(),
                distance: row_item.distance,
            });            
        }
        outCount += 1;
    }        
    //println!("outItems={:?}", outItems);      
    let output = json!({
        "rows": outItems,
    });
    let mut ouStr: String = "".to_string();
    match serde_json::to_string(&output) {
        Ok(json_str) => {
            //println!("{}", json_str);
            ouStr = json_str.clone();
        },
        Err(e) => eprintln!("エラー: {}", e),
    }    
    println!("ouStr={}", &ouStr);     
    (StatusCode::CREATED, ouStr.to_string())
}
