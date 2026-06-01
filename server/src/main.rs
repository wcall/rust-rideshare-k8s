use chrono::prelude::*;
use warp::Filter;

// Vehicle enum
#[derive(Debug, PartialEq)]
enum Vehicle {
    Car,
    Bike,
    Scooter,
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "trace");

    pretty_env_logger::init_timed();

    let root = warp::path::end().map(|| "Rust rideshare service");

    let bike = warp::path("bike").map(|| {
        order_bike(1);

        "Bike ordered"
    });

    let scooter = warp::path("scooter").map(|| {
        order_scooter(2);

        "Scooter ordered"
    });

    let car = warp::path("car").map(|| {
        order_car(3);

        "Car ordered"
    });

    let routes = warp::get().and(root).or(bike).or(scooter).or(car);

    warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
}

fn order_bike(n: u64) {
    find_nearest_vehicle(n, Vehicle::Bike);
}

fn order_scooter(n: u64) {
    find_nearest_vehicle(n, Vehicle::Scooter);
}

fn order_car(n: u64) {
    find_nearest_vehicle(n, Vehicle::Car);
}

fn find_nearest_vehicle(search_radius: u64, vehicle: Vehicle) {
    let mut _i: u64 = 0;

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_millis() < u128::from(search_radius * 200) {
        _i += 1;
    }

    if vehicle == Vehicle::Car {
        check_driver_availability(search_radius);
    }
}

fn check_driver_availability(search_radius: u64) {
    let mut _i: u64 = 0;

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_millis() < u128::from(search_radius * 200) {
        _i += 1;
    }
    // Every 4 minutes this will artificially create make requests in eu-north region slow
    // this is just for demonstration purposes to show how performance impacts show up in the
    // flamegraph
    let time_minutes = Local::now().minute();
    if std::env::var("REGION").unwrap_or_else(|_| "eu-north".to_owned()) == "eu-north"
        && (time_minutes % 4 == 0)
    {
        mutex_lock(search_radius);
    }
}

fn mutex_lock(search_radius: u64) {
    let mut _i: u64 = 0;

    let start_time = std::time::Instant::now();
    while start_time.elapsed().as_millis() < u128::from(search_radius * 4000) {
        _i += 1;
    }
}
