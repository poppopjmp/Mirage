use tokio::sync::mpsc;
use tokio::task;
use serde::{Deserialize, Serialize};
use warp::Filter;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Scan {
    id: Uuid,
    name: String,
    description: String,
    status: String,
    progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateScanRequest {
    name: String,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateScanRequest {
    name: Option<String>,
    description: Option<String>,
}

type ScansDb = Arc<Mutex<Vec<Scan>>>;

async fn create_scan(scan: CreateScanRequest, db: ScansDb) -> Result<impl warp::Reply, warp::Rejection> {
    let mut scans = db.lock().unwrap();
    let new_scan = Scan {
        id: Uuid::new_v4(),
        name: scan.name,
        description: scan.description,
        status: "CREATED".to_string(),
        progress: 0.0,
    };
    scans.push(new_scan.clone());
    Ok(warp::reply::json(&new_scan))
}

async fn get_scan(id: String, db: ScansDb) -> Result<impl warp::Reply, warp::Rejection> {
    let scans = db.lock().unwrap();
    if let Some(scan) = scans.iter().find(|&scan| scan.id.to_string() == id) {
        Ok(warp::reply::json(scan))
    } else {
        Err(warp::reject::not_found())
    }
}

async fn update_scan(id: String, update: UpdateScanRequest, db: ScansDb) -> Result<impl warp::Reply, warp::Rejection> {
    let mut scans = db.lock().unwrap();
    if let Some(scan) = scans.iter_mut().find(|scan| scan.id.to_string() == id) {
        if let Some(name) = update.name {
            scan.name = name;
        }
        if let Some(description) = update.description {
            scan.description = description;
        }
        Ok(warp::reply::json(scan))
    } else {
        Err(warp::reject::not_found())
    }
}

async fn delete_scan(id: String, db: ScansDb) -> Result<impl warp::Reply, warp::Rejection> {
    let mut scans = db.lock().unwrap();
    if let Some(pos) = scans.iter().position(|scan| scan.id.to_string() == id) {
        scans.remove(pos);
        Ok(warp::reply::with_status("Scan deleted", warp::http::StatusCode::NO_CONTENT))
    } else {
        Err(warp::reject::not_found())
    }
}

async fn start_scan(id: String, db: ScansDb, tx: mpsc::Sender<Uuid>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut scans = db.lock().unwrap();
    if let Some(scan) = scans.iter_mut().find(|scan| scan.id.to_string() == id) {
        scan.status = "RUNNING".to_string();
        let scan_id = scan.id;
        tx.send(scan_id).await.unwrap();
        Ok(warp::reply::json(scan))
    } else {
        Err(warp::reject::not_found())
    }
}

async fn scan_worker(mut rx: mpsc::Receiver<Uuid>, db: ScansDb) {
    while let Some(scan_id) = rx.recv().await {
        let mut scans = db.lock().unwrap();
        if let Some(scan) = scans.iter_mut().find(|scan| scan.id == scan_id) {
            scan.progress = 0.0;
            for i in 1..=10 {
                scan.progress += 10.0;
                task::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            scan.status = "COMPLETED".to_string();
        }
    }
}

#[tokio::main]
async fn main() {
    let db: ScansDb = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel(32);

    let db_filter = warp::any().map(move || db.clone());
    let tx_filter = warp::any().map(move || tx.clone());

    let create_scan_route = warp::post()
        .and(warp::path("scans"))
        .and(warp::body::json())
        .and(db_filter.clone())
        .and_then(create_scan);

    let get_scan_route = warp::get()
        .and(warp::path("scans"))
        .and(warp::path::param())
        .and(db_filter.clone())
        .and_then(get_scan);

    let update_scan_route = warp::put()
        .and(warp::path("scans"))
        .and(warp::path::param())
        .and(warp::body::json())
        .and(db_filter.clone())
        .and_then(update_scan);

    let delete_scan_route = warp::delete()
        .and(warp::path("scans"))
        .and(warp::path::param())
        .and(db_filter.clone())
        .and_then(delete_scan);

    let start_scan_route = warp::post()
        .and(warp::path("scans"))
        .and(warp::path::param())
        .and(warp::path("start"))
        .and(db_filter.clone())
        .and(tx_filter.clone())
        .and_then(start_scan);

    let routes = create_scan_route
        .or(get_scan_route)
        .or(update_scan_route)
        .or(delete_scan_route)
        .or(start_scan_route);

    tokio::spawn(scan_worker(rx, db.clone()));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
