# rs_vec_db_rest

 Version: 0.9.1

 date    : 2026/10/19
 
 update :

***

Rust Axum API , Vector DB SQLite

* SQLite use
* rustc 1.99.1 

***
* build
```
cargo build --release
```

***
* table add
```
sqlite3 ./vec.db < table.sql
```

***
* start
```
./target/release/vec_db_api_rest
```

***
* Test-code
* INSERT
* table: table name
* content: text
* vector: vector data

```
curl -X POST http://localhost:8888/api/insert \
  -H "Content-Type: application/json" \
  -d '{"table": "document" , "content": "hello", "vector": [0.01 , 0.03, 0.04]}'

```

* select
* table: table name
* limit: get max data
* vector: vector data

```
curl -X POST http://localhost:8888/api/select \
  -H "Content-Type: application/json" \
  -d '{"table": "document" , "limit": 3 ,"vector": [0.02 , 0.03, 0.14]}'
  
```

***
### blog

