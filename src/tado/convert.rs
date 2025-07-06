use super::{
    api::{
        ActivityDataPointsHeatingPowerApiResponse, SensorDataPointsHumidityApiResponse,
        SensorDataPointsInsideTemperatureApiResponse, WeatherApiResponse,
        WeatherOutsideTemperatureApiResponse, WeatherSolarIntensityApiResponse,
        ZoneDayReportApiResponse, ZoneStateOpenWindowApiResponse,
        ZoneStateSensorDataPointsApiResponse, ZoneStateSettingApiResponse,
        ZoneStateSettingTemperatureApiResponse, ZonesApiResponse,
    },
    model::{
        DataPoint, HeatingPower, Humidity, SingleTemperature, SolarIntensity, Temperature, Weather,
        ZoneState, ZoneStateOpenWindow, ZoneStateSensorDataPoints, ZoneStateSetting,
    },
};

impl ZonesApiResponse {
    pub fn convert(&self) -> ZoneState {
        ZoneState {
            name: self.name.clone(),
            id: self.id,
            setting: self.setting.convert(),
            heating_power: self.heatingPower.as_ref().map(|f| f.convert()),
            sensor_data_points: self.sensorDataPoints.convert(),
            open_window: self.openWindow.as_ref().map(|f| f.convert()),
        }
    }
}

impl ZoneStateOpenWindowApiResponse {
    pub fn convert(&self) -> ZoneStateOpenWindow {
        ZoneStateOpenWindow {}
    }
}

impl ZoneStateSettingApiResponse {
    pub fn convert(&self) -> ZoneStateSetting {
        ZoneStateSetting {
            temperature: self.temperature.as_ref().map(|f| f.convert()),
        }
    }
}

impl ZoneStateSettingTemperatureApiResponse {
    pub fn convert(&self) -> SingleTemperature {
        SingleTemperature { value: self.value }
    }
}

impl ActivityDataPointsHeatingPowerApiResponse {
    pub fn convert(&self) -> HeatingPower {
        HeatingPower {
            percentage: self.percentage,
        }
    }
}

impl ZoneStateSensorDataPointsApiResponse {
    pub fn convert(&self) -> ZoneStateSensorDataPoints {
        ZoneStateSensorDataPoints {
            inside_temperature: self.insideTemperature.as_ref().map(|f| f.convert()),
            humidity: self.humidity.as_ref().map(|f| f.convert()),
        }
    }
}

impl SensorDataPointsHumidityApiResponse {
    pub fn convert(&self) -> Humidity {
        Humidity {
            percentage: self.percentage,
        }
    }
}

impl SensorDataPointsInsideTemperatureApiResponse {
    pub fn convert(&self) -> SingleTemperature {
        SingleTemperature { value: self.value }
    }
}

impl WeatherOutsideTemperatureApiResponse {
    pub fn convert(&self) -> Temperature {
        Temperature {
            celsius: self.celsius,
            fahrenheit: self.fahrenheit,
        }
    }
}

impl ZoneDayReportApiResponse {
    pub fn convert_inside_temperature(&self) -> Vec<DataPoint<Temperature>> {
        self.measuredData
            .insideTemperature
            .dataPoints
            .iter()
            .map(|f| DataPoint {
                timestamp: f.timestamp.clone(),
                value: f.value.convert(),
            })
            .collect()
    }
}

impl WeatherApiResponse {
    pub fn convert(&self) -> Weather {
        Weather {
            outside_temperature: self.outsideTemperature.convert(),
            solar_intensity: self.solarIntensity.convert(),
        }
    }
}

impl WeatherSolarIntensityApiResponse {
    pub fn convert(&self) -> SolarIntensity {
        SolarIntensity {
            percentage: self.percentage,
        }
    }
}
