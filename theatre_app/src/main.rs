use iced::{
    widget::{button, column, container, row, text, scrollable, Space, text_input, Button},
    Alignment, Element, Length, Sandbox, Settings, Color, Theme,
};
use serde::{Deserialize, Serialize};
use chrono::Local;
use std::fs;
use uuid::Uuid;

// ============================================================================
// Data Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Show {
    id: usize,
    name: String,
    date: String,
    time: String,
    hall: String,
    price: f64,
    available_seats: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Booking {
    id: String,
    show_id: usize,
    customer_name: String,
    seat: String,
    booking_time: String,
    price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Seat {
    row: char,
    col: usize,
    is_booked: bool,
    booking_id: Option<String>,
}

// ============================================================================
// Application State
// ============================================================================

struct TheatreApp {
    current_view: View,
    shows: Vec<Show>,
    bookings: Vec<Booking>,
    seats: Vec<Vec<Vec<Seat>>>, 
    selected_show: Option<usize>,
    selected_seat: Option<(usize, usize)>,
    customer_name: String,
    booking_id_input: String,
    error_message: Option<String>,
    success_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum View {
    Home,
    ShowSelection,
    Booking,
    CancelBooking,
    ViewSeats,
    Records,
    Statistics,
}

#[derive(Debug, Clone)]
enum Message {
    ChangeView(View),
    SelectShow(usize),
    SelectSeat(usize, usize),
    CustomerNameChanged(String),
    ConfirmBooking,
    BookingIdChanged(String),
    CancelBookingConfirm,
    ExportRecords,
}

impl Sandbox for TheatreApp {
    type Message = Message;

    fn new() -> Self {
        let shows = vec![
            Show { id: 0, name: "Dune: Part Two".to_string(), date: "15-03-2024".to_string(), time: "18:00".to_string(), hall: "Hall 1".to_string(), price: 1500.0, available_seats: 20 },
            Show { id: 1, name: "Oppenheimer".to_string(), date: "20-03-2024".to_string(), time: "20:30".to_string(), hall: "Hall 2".to_string(), price: 2250.0, available_seats: 20 },
            Show { id: 2, name: "Barbie".to_string(), date: "22-03-2024".to_string(), time: "19:00".to_string(), hall: "Hall 3".to_string(), price: 2000.0, available_seats: 20 },
            Show { id: 3, name: "Deadpool & Wolverine".to_string(), date: "25-03-2024".to_string(), time: "21:00".to_string(), hall: "Hall 4".to_string(), price: 1500.0, available_seats: 20 },
            Show { id: 4, name: "Inside Out 2".to_string(), date: "28-03-2024".to_string(), time: "17:30".to_string(), hall: "Hall 5".to_string(), price: 1500.0, available_seats: 20 },
        ];

        let seats = (0..5).map(|_| {
            (0..4).map(|row| {
                (0..5).map(|col| Seat {
                    row: char::from_u32('A' as u32 + row as u32).unwrap(),
                    col: col + 1,
                    is_booked: false,
                    booking_id: None,
                }).collect()
            }).collect()
        }).collect();

        Self {
            current_view: View::Home,
            shows,
            bookings: Vec::new(),
            seats,
            selected_show: None,
            selected_seat: None,
            customer_name: String::new(),
            booking_id_input: String::new(),
            error_message: None,
            success_message: None,
        }
    }

    fn title(&self) -> String { "Premium Theatre Reservation System".to_string() }

    fn update(&mut self, message: Message) {
        self.error_message = None;
        self.success_message = None;

        match message {
            Message::ChangeView(view) => {
                self.current_view = view;
                self.customer_name.clear();
                self.booking_id_input.clear();
                self.selected_seat = None;
            }
            Message::SelectShow(id) => {
                self.selected_show = Some(id);
                self.current_view = View::Booking;
            }
            Message::SelectSeat(row, col) => {
                if let Some(show_id) = self.selected_show {
                    if !self.seats[show_id][row][col].is_booked {
                        self.selected_seat = Some((row, col));
                    }
                }
            }
            Message::CustomerNameChanged(name) => self.customer_name = name,
            Message::ConfirmBooking => {
                if let (Some(show_id), Some((row, col))) = (self.selected_show, self.selected_seat) {
                    if self.customer_name.trim().is_empty() {
                        self.error_message = Some("Please enter customer name".to_string());
                        return;
                    }

                    let booking_id = Uuid::new_v4().to_string();
                    let seat = &mut self.seats[show_id][row][col];
                    
                    seat.is_booked = true;
                    seat.booking_id = Some(booking_id.clone());

                    let booking = Booking {
                        id: booking_id.clone(),
                        show_id,
                        customer_name: self.customer_name.clone(),
                        seat: format!("{}{}", seat.row, seat.col),
                        booking_time: Local::now().format("%d-%m-%Y %H:%M:%S").to_string(),
                        price: self.shows[show_id].price,
                    };

                    self.bookings.push(booking.clone());
                    self.shows[show_id].available_seats -= 1;
                    self.save_ticket(&booking);

                    self.success_message = Some(format!("Booking confirmed! ID: {}", booking_id));
                    self.customer_name.clear();
                    self.selected_seat = None;
                }
            }
            Message::BookingIdChanged(id) => self.booking_id_input = id,
            Message::CancelBookingConfirm => {
                let booking_id = self.booking_id_input.trim();
                if let Some(idx) = self.bookings.iter().position(|b| b.id == booking_id) {
                    let show_id = self.bookings[idx].show_id;
                    for row in &mut self.seats[show_id] {
                        for seat in row {
                            if seat.booking_id.as_deref() == Some(booking_id) {
                                seat.is_booked = false;
                                seat.booking_id = None;
                            }
                        }
                    }
                    self.bookings.remove(idx);
                    self.shows[show_id].available_seats += 1;
                    self.success_message = Some("Booking cancelled successfully".to_string());
                    self.booking_id_input.clear();
                } else {
                    self.error_message = Some("Booking ID not found".to_string());
                }
            }
            Message::ExportRecords => {
                self.export_records();
                self.success_message = Some("Records exported to bookings_export.json".to_string());
            }
        }
    }

    // FIXED: Added '_ for lifetime elision
    fn view(&self) -> Element<'_, Message> {
        let content = match self.current_view {
            View::Home => self.home_view(),
            View::ShowSelection => self.show_selection_view(),
            View::Booking => self.booking_view(),
            View::CancelBooking => self.cancel_booking_view(),
            View::ViewSeats => self.view_seats(),
            View::Records => self.records_view(),
            View::Statistics => self.statistics_view(),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .style(container_dark_style)
            .into()
    }

    fn theme(&self) -> Theme { Theme::Dark }
}

impl TheatreApp {
    // FIXED: Added '_ to all return types
    fn home_view(&self) -> Element<'_, Message> {
        column![
            text("🎬 Premium Theatre Reservation").size(48),
            text("Your ultimate movie booking experience").size(20),
            Space::with_height(40),
            column![
                menu_button("🎥 Browse Movies", Message::ChangeView(View::ShowSelection)),
                menu_button("🎫 Book Seats", Message::ChangeView(View::ShowSelection)),
                menu_button("❌ Cancel Booking", Message::ChangeView(View::CancelBooking)),
                menu_button("💺 View Seats", Message::ChangeView(View::ViewSeats)),
                menu_button("📋 All Records", Message::ChangeView(View::Records)),
                menu_button("📊 Statistics", Message::ChangeView(View::Statistics)),
            ].spacing(15).align_items(Alignment::Center)
        ]
        .spacing(20).align_items(Alignment::Center).width(Length::Fill).into()
    }

    fn show_selection_view(&self) -> Element<'_, Message> {
        let shows: Element<_> = self.shows.iter()
            .fold(column![].spacing(15), |col, show| col.push(show_card(show)))
            .into();

        column![
            text("Now Showing").size(36),
            Space::with_height(20),
            shows,
            Space::with_height(20),
            button("← Back to Home").on_press(Message::ChangeView(View::Home)).padding(10)
        ].width(Length::Fill).into()
    }

    fn booking_view(&self) -> Element<'_, Message> {
        if let Some(show_id) = self.selected_show {
            let show = &self.shows[show_id];
            let mut seat_grid = column![].spacing(10);
            
            for (r_idx, row) in self.seats[show_id].iter().enumerate() {
                let mut seat_row = row![text(format!("{}", r_idx + 1)).size(16)].spacing(8);
                for (c_idx, seat) in row.iter().enumerate() {
                    let is_sel = self.selected_seat == Some((r_idx, c_idx));
                    seat_row = seat_row.push(create_seat_button(seat, is_sel, r_idx, c_idx));
                }
                seat_grid = seat_grid.push(seat_row);
            }

            let mut content = column![
                text(format!("Booking: {}", show.name)).size(32),
                text(format!("📅 {} | ⏰ {} | 🏛️ {} | 💰 LKR {:.2}", show.date, show.time, show.hall, show.price)).size(16),
                Space::with_height(20),
                text("🎬 SCREEN").size(20),
                Space::with_height(10),
                seat_grid,
                Space::with_height(20),
                text_input("Enter your name", &self.customer_name).on_input(Message::CustomerNameChanged).padding(10),
                button("✅ Confirm Booking").on_press(Message::ConfirmBooking).padding(15),
                button("← Back").on_press(Message::ChangeView(View::ShowSelection)).padding(10)
            ].spacing(10).align_items(Alignment::Center);

            if let Some(msg) = &self.error_message { content = content.push(text(msg).style(Color::from_rgb(0.9, 0.3, 0.3))); }
            if let Some(msg) = &self.success_message { content = content.push(text(msg).style(Color::from_rgb(0.3, 0.9, 0.3))); }

            content.into()
        } else {
            column![text("No show selected"), button("← Back").on_press(Message::ChangeView(View::ShowSelection))].into()
        }
    }

    fn cancel_booking_view(&self) -> Element<'_, Message> {
        let mut content = column![
            text("Cancel Booking").size(36),
            text_input("Enter Booking ID", &self.booking_id_input).on_input(Message::BookingIdChanged).padding(10),
            button("❌ Cancel Booking").on_press(Message::CancelBookingConfirm).padding(15),
            button("← Back to Home").on_press(Message::ChangeView(View::Home)).padding(10)
        ].spacing(15).align_items(Alignment::Center);

        if let Some(msg) = &self.error_message { content = content.push(text(msg).style(Color::from_rgb(0.9, 0.3, 0.3))); }
        if let Some(msg) = &self.success_message { content = content.push(text(msg).style(Color::from_rgb(0.3, 0.9, 0.3))); }
        content.into()
    }

    fn view_seats(&self) -> Element<'_, Message> {
        column![
            text("Seat Availability").size(36),
            button("← Back to Home").on_press(Message::ChangeView(View::Home)).padding(10)
        ].spacing(10).align_items(Alignment::Center).into()
    }

    fn records_view(&self) -> Element<'_, Message> {
        let records: Element<_> = if self.bookings.is_empty() {
            text("No bookings yet").into()
        } else {
            self.bookings.iter().rev().fold(column![].spacing(10), |col, b| {
                col.push(container(column![
                    text(format!("🎫 ID: {}", b.id)).size(14),
                    text(format!("👤 {}", b.customer_name)).size(16),
                    text(format!("🎬 {} | 💺 {}", self.shows[b.show_id].name, b.seat)).size(14),
                ].padding(15)).style(container_card_style).width(Length::Fill))
            }).into()
        };

        column![
            text("All Booking Records").size(36),
            button("💾 Export Records").on_press(Message::ExportRecords).padding(10),
            scrollable(records),
            button("← Back to Home").on_press(Message::ChangeView(View::Home)).padding(10)
        ].spacing(10).into()
    }

    fn statistics_view(&self) -> Element<'_, Message> {
        let total_bookings = self.bookings.len().to_string();
        let total_revenue = format!("LKR {:.2}", self.bookings.iter().map(|b| b.price).sum::<f64>());
        let available_seats = self.shows.iter().map(|s| s.available_seats).sum::<usize>().to_string();

        column![
            text("Booking Statistics").size(36),
            Space::with_height(20),
            stat_card("📊 Total Bookings", total_bookings),
            stat_card("💰 Total Revenue", total_revenue),
            stat_card("💺 Available Seats", available_seats),
            Space::with_height(20),
            button("← Back to Home").on_press(Message::ChangeView(View::Home)).padding(10)
        ].spacing(10).align_items(Alignment::Center).into()
    }

    fn save_ticket(&self, booking: &Booking) {
        let show = &self.shows[booking.show_id];
        let content = format!("Movie: {}\nSeat: {}\nPrice: LKR {:.2}\nID: {}", show.name, booking.seat, booking.price, booking.id);
        let _ = fs::write(format!("ticket_{}.txt", booking.id), content);
    }

    fn export_records(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.bookings) {
            let _ = fs::write("bookings_export.json", json);
        }
    }
}

// ============================================================================
// Styles and Helpers
// ============================================================================

fn container_dark_style(_theme: &Theme) -> container::Appearance {
    container::Appearance { background: Some(Color::from_rgb(0.05, 0.05, 0.1).into()), ..Default::default() }
}

fn container_card_style(_theme: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(Color::from_rgb(0.1, 0.1, 0.15).into()),
        border: iced::Border { color: Color::from_rgb(0.3, 0.3, 0.4), width: 1.0, radius: 8.0.into() },
        ..Default::default()
    }
}

fn menu_button<'a>(label: &str, message: Message) -> Button<'a, Message> {
    button(text(label).size(20)).on_press(message).padding(20).width(Length::Fixed(400.0))
}

// FIXED: Added '_ to return type
fn show_card(show: &Show) -> Element<'_, Message> {
    container(column![
        text(&show.name).size(24),
        text(format!("💺 {} seats available", show.available_seats)).size(14),
        button("Book Now →").on_press(Message::SelectShow(show.id)).padding(10),
    ].spacing(10).padding(20)).style(container_card_style).width(Length::Fill).into()
}

// FIXED: Added '_ to return type
fn create_seat_button(seat: &Seat, is_selected: bool, row: usize, col: usize) -> Element<'_, Message> {
    let emoji = if seat.is_booked { "🔴" } else if is_selected { "🟡" } else { "🟢" };
    let btn = button(text(emoji).size(24)).padding(8);
    if !seat.is_booked { btn.on_press(Message::SelectSeat(row, col)).into() } else { btn.into() }
}

fn stat_card<'a>(label: impl Into<String>, value: impl Into<String>) -> Element<'a, Message> {
    container(column![
        text(label.into()).size(18),
        text(value.into()).size(32),
    ].spacing(10).padding(20).align_items(Alignment::Center))
    .width(Length::Fixed(300.0)).style(container_card_style).into()
}

fn main() -> iced::Result {
    TheatreApp::run(Settings {
        window: iced::window::Settings { size: iced::Size::new(900.0, 700.0), ..Default::default() },
        ..Default::default()
    })
}