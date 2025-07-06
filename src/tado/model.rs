pub struct Weather {
    pub solar_intensity: SolarIntensity,
    pub outside_temperature: Temperature,
}

pub struct Temperature {
    pub celsius: f64,
    pub fahrenheit: f64,
}

pub struct SingleTemperature {
    pub value: f64,
}

pub struct SolarIntensity {
    pub percentage: f64,
}

pub struct Humidity {
    pub percentage: f64,
}

pub struct HeatingPower {
    pub percentage: f64,
}

pub struct ZoneState {
    pub name: String,
    pub id: i32,
    pub setting: ZoneStateSetting,
    pub heating_power: Option<HeatingPower>,
    pub sensor_data_points: ZoneStateSensorDataPoints,
    pub open_window: Option<ZoneStateOpenWindow>,
}

pub struct ZoneStateSetting {
    pub temperature: Option<SingleTemperature>,
}

pub struct ZoneStateOpenWindow {

}

pub struct ZoneStateSensorDataPoints {
    pub inside_temperature: Option<SingleTemperature>,
    pub humidity: Option<Humidity>,
}

pub struct DataPoint<T> {
    pub timestamp: String,
    pub value: T,
}

pub struct HistoryReport {
    pub name: String,
    pub inside_temperature: Vec<DataPoint<Temperature>>,
}