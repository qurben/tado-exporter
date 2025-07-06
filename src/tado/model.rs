use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct AuthStartResponse {
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
    pub verification_uri_complete: String,
}

#[derive(Deserialize, Debug)]
pub struct AuthTokensErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

#[derive(Deserialize, Debug)]
pub struct MeApiResponse {
    pub homes: Vec<HomesApiResponse>,
}

#[derive(Deserialize, Debug)]
pub struct HomesApiResponse {
    pub id: i32,
}

#[derive(Deserialize, Debug)]
pub struct ZonesApiResponse {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateApiResponse {
    pub setting: ZoneStateSettingApiResponse,
    // pub activityDataPoints: ZoneStateActivityDataPointsApiResponse,
    pub heatingPower: Option<ActivityDataPointsHeatingPowerApiResponse>,
    pub sensorDataPoints: ZoneStateSensorDataPointsApiResponse,
    pub openWindow: Option<ZoneStateOpenWindowApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct ZoneStateOpenWindowApiResponse {
    pub detectedTime: String, // RFC 3339 timestamp
    pub durationInSeconds: i32,
    pub expiry: String,
    pub remainingTimeInSeconds: i32,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSettingApiResponse {
    pub power: String,
    pub temperature: Option<ZoneStateSettingTemperatureApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ZoneStateSettingTemperatureApiResponse {
    pub value: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateActivityDataPointsApiResponse {
    pub heatingPower: Option<ActivityDataPointsHeatingPowerApiResponse>,
    pub acPower: Option<ActivityDataPointsAcPowerApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ActivityDataPointsHeatingPowerApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ActivityDataPointsAcPowerApiResponse {
    pub value: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneStateSensorDataPointsApiResponse {
    pub insideTemperature: Option<SensorDataPointsInsideTemperatureApiResponse>,
    pub humidity: Option<SensorDataPointsHumidityApiResponse>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SensorDataPointsInsideTemperatureApiResponse {
    pub value: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct SensorDataPointsHumidityApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct WeatherApiResponse {
    pub solarIntensity: WeatherSolarIntensityApiResponse,
    pub outsideTemperature: WeatherOutsideTemperatureApiResponse,
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct WeatherSolarIntensityApiResponse {
    pub percentage: f64,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct WeatherOutsideTemperatureApiResponse {
    pub fahrenheit: f64,
    pub celsius: f64,
}

pub struct ZoneStateResponse {
    pub name: String,
    pub id: i32,
    pub state_response: ZoneStateApiResponse,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct TimeInterval {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DataPoint<T> {
    pub timestamp: String,
    pub value: T,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct DataInterval {
    pub from: String,
    pub to: String,
    pub value: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(non_snake_case)]
#[serde(tag = "valueType")]
pub struct ZoneDayReportMeasuredDataMeasuringDeviceConnectedApiResponse {
    pub timeSeriesType: String,
    pub valueType: String,
    pub dataIntervals: Vec<DataInterval>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneDayReportMeasuredDataInsideTemperatureApiResponse {
    pub timeSeriesType: String,
    pub valueType: String,
    pub min: WeatherOutsideTemperatureApiResponse,
    pub max: WeatherOutsideTemperatureApiResponse,
    pub dataPoints: Vec<DataPoint<WeatherOutsideTemperatureApiResponse>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneDayReportMeasuredDataHumidityApiResponse {
    pub timeSeriesType: String,
    pub valueType: String,
    pub percentageUnit: String,
    pub min: f64,
    pub max: f64,
    pub dataPoints: Vec<DataPoint<f64>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneDayReportMeasuredDataApiResponse {
    pub measuringDeviceConnected: ZoneDayReportMeasuredDataMeasuringDeviceConnectedApiResponse,
    pub insideTemperature: ZoneDayReportMeasuredDataInsideTemperatureApiResponse,
    pub humidity: ZoneDayReportMeasuredDataHumidityApiResponse,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct ZoneDayReportApiResponse {
    pub zoneType: String,
    pub interval: TimeInterval,
    pub hoursInDay: i32,
    pub measuredData: ZoneDayReportMeasuredDataApiResponse,
    // pub stripes: Any,
    // pub settings: Any,
    // pub callForHeat: Any,
    // pub weather: Any,
}

pub struct HistoryReport {
    pub name: String,
    pub inside_temperature: Vec<DataPoint<WeatherOutsideTemperatureApiResponse>>,
}