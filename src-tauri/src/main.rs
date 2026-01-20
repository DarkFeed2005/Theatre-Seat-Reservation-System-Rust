// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    theatre_app_lib::run()
}
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::collections::HashMap;
use chrono::Local;

// --- Data Models ---
#[derive(Clone, Serialize, Deserialize)]
struct Movie {
    id: u32,
    title: String,
    price: f64,
    hall: String,
    emoji: String,
    time: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Booking {
    id: u32,
    customer_name: String,
    email: String,
    movie_title: String,
    seats: Vec<String>,
    total_amount: f64,
    date: String,
}

// --- Application State ---
struct AppState {
    movies: Vec<Movie>,
    // Mapping: MovieID -> List of Booked Seat IDs
    booked_seats: Mutex<HashMap<u32, Vec<String>>>,
    bookings: Mutex<Vec<Booking>>,
}

// --- Commands (API for Frontend) ---

#[tauri::command]
fn get_movies(state: tauri::State<AppState>) -> Vec<Movie> {
    state.movies.clone()
}

#[tauri::command]
fn get_booked_seats(state: tauri::State<AppState>, movie_id: u32) -> Vec<String> {
    let booked = state.booked_seats.lock().unwrap();
    booked.get(&movie_id).cloned().unwrap_or_default()
}

#[tauri::command]
fn make_booking(
    state: tauri::State<AppState>,
    movie_id: u32,
    name: String,
    email: String,
    seats: Vec<String>,
    total: f64
) -> Result<Booking, String> {
    let mut booked_map = state.booked_seats.lock().unwrap();
    let mut all_bookings = state.bookings.lock().unwrap();
    
    // Check if any seat was taken while user was deciding
    let movie_booked = booked_map.entry(movie_id).or_insert(Vec::new());
    for seat in &seats {
        if movie_booked.contains(seat) {
            return Err(format!("Seat {} is no longer available.", seat));
        }
    }

    // Process Booking
    movie_booked.extend(seats.clone());
    
    let movie_title = state.movies.iter()
        .find(|m| m.id == movie_id)
        .map(|m| m.title.clone())
        .unwrap_or_default();

    let new_booking = Booking {
        id: (all_bookings.len() + 1001) as u32,
        customer_name: name,
        email,
        movie_title,
        seats,
        total_amount: total,
        date: Local::now().format("%Y-%m-%d %H:%M").to_string(),
    };
    
    all_bookings.push(new_booking.clone());
    Ok(new_booking)
}

fn main() {
    let movies = vec![
        Movie { id: 1, title: "Dune: Part Two".into(), price: 12.5, hall: "1".into(), emoji: "🏜️".into(), time: "18:00".into() },
        Movie { id: 2, title: "Oppenheimer".into(), price: 15.0, hall: "2".into(), emoji: "💣".into(), time: "20:30".into() },
        Movie { id: 3, title: "Barbie".into(), price: 12.0, hall: "3".into(), emoji: "💖".into(), time: "19:00".into() },
    ];

    tauri::Builder::default()
        .manage(AppState {
            movies,
            booked_seats: Mutex::new(HashMap::new()),
            bookings: Mutex::new(Vec::new()),
        })
        .invoke_handler(tauri::generate_handler![get_movies, get_booked_seats, make_booking])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}