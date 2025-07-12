use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::time::{Duration, Instant};
use std::vec::Vec;

use chrono::DateTime;
use chrono::Utc;
use lazy_static::lazy_static;
use log::{error, info};
use reqwest;
use std::fs;

use super::error::AuthError;
use super::model::{HistoryReport, Weather, ZoneState};

use super::api::{
    AuthStartResponse, AuthTokensErrorResponse, AuthTokensResponse, MeApiResponse,
    WeatherApiResponse, ZoneDayReportApiResponse, ZonesApiResponse,
};

const AUTH_PENDING_MESSAGE: &str = "authorization_pending";
const TOKENS_FILE: &str = "tokens.json";

lazy_static! {
    // TODO: POST DEVICE - https://login.tado.com/oauth2/device
    static ref AUTH_START_URL: reqwest::Url = "https://login.tado.com/oauth2/device_authorize".parse().unwrap();
    static ref AUTH_TOKEN_URL: reqwest::Url = "https://login.tado.com/oauth2/token".parse().unwrap();
    pub static ref BASE_URL: reqwest::Url = "https://my.tado.com/api/v2/".parse().unwrap();
    pub static ref HOPS_URL: reqwest::Url = "https://hops.tado.com/".parse().unwrap();
}

pub struct Client {
    http_client: reqwest::Client,
    base_url: reqwest::Url,
    hops_url: reqwest::Url,

    // API Authentication information.
    client_id: String,
    tokens: AuthTokensResponse,
    tokens_refresh_by: Instant,

    home_id: i32,
}

impl Client {
    pub fn new(client_id: String) -> Client {
        Client::with_base_url(BASE_URL.clone(), HOPS_URL.clone(), client_id)
    }

    fn with_base_url(base_url: reqwest::Url, hops_url: reqwest::Url, client_id: String) -> Client {
        Client {
            http_client: reqwest::Client::new(),
            base_url,
            hops_url,
            client_id,
            tokens: AuthTokensResponse {
                access_token: String::default(),
                expires_in: 0,
                refresh_token: String::default(),
            },
            tokens_refresh_by: Instant::now(),
            home_id: 0,
        }
    }

    /// Authenticate to the Tado API service.
    ///
    /// The authentication processes uses the oauth2 device code grant flow as required by Tado
    /// <https://support.tado.com/en/articles/8565472-how-do-i-authenticate-to-access-the-rest-api>.
    ///
    /// To avoid manual intervention, the method also attempts to complete the login challenge
    /// on behalf of the user.
    pub async fn authenticate(&mut self) -> Result<(), AuthError> {
        self.load_tokens().expect("Loading tokens should not fail");

        if let Ok(()) = self.refresh_authentication().await {
            info!("Refreshed authentication tokens");

            return Ok(());
        }

        // Start device authentication flow.
        let start_params = [
            ("client_id", self.client_id.as_str()),
            ("scope", "offline_access"),
        ];
        let resp = self
            .http_client
            .post(AUTH_START_URL.clone())
            .form(&start_params)
            .send()
            .await?;
        let start = resp.json::<AuthStartResponse>().await?;
        info!(
            "Started device authentication flow with URL {}",
            start.verification_uri_complete
        );

        // TODO: run through login flow.

        // Wait for API tokens to be returned once the flow is complete.
        self.wait_for_tokens(start).await?;
        Ok(())
    }

    async fn get(&self, url: reqwest::Url) -> Result<reqwest::Response, reqwest::Error> {
        self.http_client
            .get(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.tokens.access_token),
            )
            .send()
            .await
    }

    async fn me(&self) -> Result<MeApiResponse, reqwest::Error> {
        let url = self.base_url.join("/api/v2/me").unwrap();
        let resp = self.get(url).await?;

        resp.json::<MeApiResponse>().await
    }

    async fn zones(&mut self) -> Result<Vec<ZonesApiResponse>, reqwest::Error> {
        let endpoint = format!("homes/{}/rooms", self.home_id);
        let url = self.hops_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        resp.json::<Vec<ZonesApiResponse>>().await
    }

    async fn weather(&self) -> Result<WeatherApiResponse, reqwest::Error> {
        let endpoint = format!("homes/{}/weather/", self.home_id);
        let url = self.base_url.join(&endpoint).unwrap();

        let resp = self.get(url).await?;

        resp.json::<WeatherApiResponse>().await
    }

    pub fn merge_history(
        &mut self,
        history: &mut HashMap<String, HistoryReport>,
        report: HistoryReport,
    ) {
        match history.get_mut(&report.name) {
            Some(r) => r.inside_temperature.extend(report.inside_temperature),
            None => {
                history.insert(report.name.clone(), report);
            }
        }
    }

    pub async fn history(&mut self) -> HashMap<String, HistoryReport> {
        // get last month
        let mut date = Utc::now();

        let mut history = HashMap::new();

        for zone in self.retrieve_zones().await {
            for _ in 1..30 {
                date -= chrono::Duration::days(1);
                info!("Retrieving history for {}", date.format("%Y-%m-%d"));

                let today = self.history_date(&zone, date).await;

                self.merge_history(&mut history, today)
            }
        }

        history
    }

    pub async fn history_date(&mut self, zone: &ZoneState, date: DateTime<Utc>) -> HistoryReport {
        let endpoint = format!("/api/v2/homes/{}/zones/{}/dayReport", self.home_id, zone.id);
        let mut url = self.base_url.join(&endpoint).unwrap();
        url.set_query(Some(format!("date={}", date.format("%Y-%m-%d")).as_str()));

        let inside_temperature = self
            .get(url)
            .await
            .expect("Unable to connect")
            .json::<ZoneDayReportApiResponse>()
            .await
            .expect("Unable to deserialize")
            .convert_inside_temperature();

        HistoryReport {
            name: zone.name.clone(),
            inside_temperature,
        }
    }

    /// Refresh the API access token if it expired.
    pub async fn refresh_authentication(&mut self) -> Result<(), AuthError> {
        if Instant::now() < self.tokens_refresh_by {
            return Ok(());
        }

        let refresh_params = [
            ("client_id", self.client_id.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", self.tokens.refresh_token.as_str()),
        ];
        let resp = self
            .http_client
            .post(AUTH_TOKEN_URL.clone())
            .form(&refresh_params)
            .send()
            .await?;

        let tokens = resp.json::<AuthTokensResponse>().await?;
        self.set_tokens(tokens)
            .expect("Unable to write auth tokens");

        info!("API access tokens refreshed");
        Ok(())
    }

    pub async fn retrieve_zones(&mut self) -> Vec<ZoneState> {
        // retrieve home details (only if we don't already have a home identifier)
        if self.home_id == 0 {
            let me_response = match self.me().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("unable to retrieve home identifier: {e}");
                    return Vec::new();
                }
            };

            self.home_id = me_response.homes.first().unwrap().id;
        }

        // retrieve home different zones
        let zones_response = match self.zones().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("unable to retrieve home zones: {e}");
                return Vec::new();
            }
        };

        let mut response = Vec::<ZoneState>::new();

        for zone in zones_response {
            info!("retrieving zone details for {}...", zone.name);

            response.push(zone.convert());
        }

        response
    }

    pub async fn retrieve_weather(&mut self) -> Option<Weather> {
        info!("retrieving weather details ...");

        // retrieve home details (only if we don't already have a home identifier)
        if self.home_id == 0 {
            let me_response = match self.me().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("unable to retrieve home identifier: {e}");
                    return None;
                }
            };

            self.home_id = me_response.homes.first().unwrap().id;
        }

        // retrieve weather state
        let weather_response = match self.weather().await {
            Ok(resp) => resp,
            Err(e) => {
                error!("unable to retrieve weather info: {e}");
                return None;
            }
        };

        Some(weather_response.convert())
    }

    /// Set the API access tokens to use and manage related metadata.
    fn set_tokens(&mut self, tokens: AuthTokensResponse) -> Result<(), Error> {
        // Reduce the tokens validity slightly to refresh before they expire.
        let expires_in = tokens.expires_in - 10;

        File::create(TOKENS_FILE)?.write_all(serde_json::to_string(&tokens.clone())?.as_bytes())?;

        self.tokens = tokens;
        self.tokens_refresh_by = Instant::now() + Duration::from_secs(expires_in);

        Ok(())
    }

    fn load_tokens(&mut self) -> Result<(), Error> {
        if let Some(json) = fs::read_to_string(TOKENS_FILE).ok() {
            // Ignore if file is not there
            self.tokens = serde_json::from_str::<AuthTokensResponse>(&json)?;
        }

        Ok(())
    }

    async fn wait_for_tokens(&mut self, start: AuthStartResponse) -> Result<(), AuthError> {
        let must_complete_by = Instant::now() + Duration::from_secs(start.expires_in);
        let token_params = [
            ("client_id", self.client_id.as_str()),
            ("device_code", &start.device_code),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ];
        while Instant::now() < must_complete_by {
            let resp = self
                .http_client
                .post(AUTH_TOKEN_URL.clone())
                .form(&token_params)
                .send()
                .await?;
            match resp.status() {
                reqwest::StatusCode::OK => {
                    let tokens = resp.json::<AuthTokensResponse>().await?;
                    self.set_tokens(tokens)
                        .expect("Unable to write auth tokens");
                    info!("Device authentication flow completed");
                    return Ok(());
                }
                reqwest::StatusCode::BAD_REQUEST => {
                    let error = resp
                        .error_for_status_ref()
                        .expect_err("must be error for BAD_REQUEST");
                    let failure = resp.json::<AuthTokensErrorResponse>().await?;
                    if failure.error != AUTH_PENDING_MESSAGE {
                        return Err(AuthError::from(error));
                    }
                }
                _ => {
                    let status = resp.status();
                    let url = resp.url().clone();
                    resp.error_for_status()?;
                    return Err(AuthError::UnexpectedStatus(status, url));
                }
            }
            info!("Device authentication flow still pending, will retry");
            tokio::time::sleep(Duration::from_secs(start.interval)).await;
        }
        Err(AuthError::Timeout)
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    use crate::tado::api::{
        WeatherOutsideTemperatureApiResponse, WeatherSolarIntensityApiResponse,
    };

    use rstest::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_new() {
        let client = Client::new("client_id".to_string());

        assert_eq!(client.client_id, "client_id");
        assert_eq!(client.base_url, *BASE_URL);
    }

    #[test]
    fn test_with_base_url() {
        let client = Client::with_base_url(
            "https://example.com".parse().unwrap(),
            "https://example.com".parse().unwrap(),
            "client_id".to_string(),
        );

        assert_eq!(client.client_id, "client_id");
        assert_eq!(client.base_url, "https://example.com".parse().unwrap());
    }

    #[rstest(response_str, expected,
        case(
            r#"
            {
                "solarIntensity": {
                  "type": "PERCENTAGE",
                  "percentage": 18.3,
                  "timestamp": "2022-09-03T17:43:41.088Z"
                },
                "outsideTemperature": {
                  "celsius": 21.53,
                  "fahrenheit": 70.75,
                  "timestamp": "2022-09-03T17:43:41.088Z",
                  "type": "TEMPERATURE",
                  "precision": { "celsius": 0.01, "fahrenheit": 0.01 }
                },
                "weatherState": {
                  "type": "WEATHER_STATE",
                  "value": "CLOUDY_PARTLY",
                  "timestamp": "2022-09-03T17:43:41.088Z"
                }
              }
            "#,
            WeatherApiResponse {
                solarIntensity: WeatherSolarIntensityApiResponse {
                    percentage: 18.3,
                },
                outsideTemperature: WeatherOutsideTemperatureApiResponse{
                    celsius: 21.53,
                    fahrenheit: 70.75
                },
            }
        )
    )]
    #[actix_rt::test]
    async fn test_weather(response_str: &str, expected: WeatherApiResponse) {
        /*
        GIVEN an OSM client
        WHEN calling the capabilities() function
        THEN returns the sets of capablities and policies
        */

        // GIVEN
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("homes/0/weather/"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(response_str, "application/json"))
            .mount(&mock_server)
            .await;

        let client = Client::with_base_url(
            mock_server.uri().parse().unwrap(),
            mock_server.uri().parse().unwrap(),
            "client_secret".to_string(),
        );

        // WHEN
        let actual = client.weather().await.unwrap();

        // THEN
        assert_eq!(actual, expected);
    }
}
